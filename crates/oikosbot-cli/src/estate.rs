// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Estate-level telemetry, capability and DEA analysis CLI subcommand.
//!
//! `collect` pulls GitHub Actions run/repo/release telemetry into a
//! resumable per-repo staging directory; `analyse` reads that staging area,
//! writes a Parquet snapshot, and runs the pure [`run_analysis`] core into
//! `analysis.json`; `report` renders that snapshot as Markdown or JSON.

use anyhow::{Context, Result};
use clap::Subcommand;
use oikosbot_capability::{assess, CapabilityRow};
use oikosbot_dea::{dea, DeaScore, Dmu};
use oikosbot_telemetry::collect::{collect_releases, collect_runs, list_repos, GhCli, GhRunner};
use oikosbot_telemetry::derive::{confidence_of, derive_per_repo, Assumptions};
use oikosbot_telemetry::rows::{ReleaseRow, RepoRow, RunRow};
use oikosbot_telemetry::snapshot;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Subcommand)]
pub enum EstateCmd {
    /// Collect run/repo/release telemetry into a staging directory
    /// (resumable: an existing staging file for a repo/owner is skipped)
    Collect {
        /// GitHub owner (user or org) to sweep; may be repeated
        #[arg(long = "owner", required = true)]
        owners: Vec<String>,

        /// Staging directory for the raw per-repo/per-owner JSON files
        #[arg(long)]
        out: PathBuf,

        /// Maximum workflow runs to fetch per repository
        #[arg(long, default_value_t = 200)]
        max_runs: usize,
    },

    /// Analyse staged telemetry into a Parquet + JSON snapshot
    Analyse {
        /// Staging directory produced by `estate collect`
        #[arg(long)]
        staging: PathBuf,

        /// Output snapshot directory (created if missing)
        #[arg(long)]
        snapshot: PathBuf,
    },

    /// Render a report from an analysis snapshot
    Report {
        /// Snapshot directory produced by `estate analyse`
        #[arg(long)]
        snapshot: PathBuf,

        /// Output format (md, json)
        #[arg(long, default_value = "md")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}

/// Dispatch an already-parsed [`EstateCmd`].
pub fn run(cmd: EstateCmd) -> Result<()> {
    match cmd {
        EstateCmd::Collect {
            owners,
            out,
            max_runs,
        } => collect(&GhCli, &owners, &out, max_runs),
        EstateCmd::Analyse { staging, snapshot } => analyse(&staging, &snapshot),
        EstateCmd::Report {
            snapshot,
            format,
            output,
        } => report(&snapshot, &format, output.as_deref()),
    }
}

/// A serde-friendly mirror of [`oikosbot_telemetry::derive::DerivedRepo`],
/// which does not itself derive `Serialize`.
#[derive(Debug, Clone, serde::Serialize)]
pub struct DerivedOut {
    pub repo: String,
    pub wall_minutes: f64,
    pub energy_kwh: f64,
    pub carbon_g: f64,
    pub imputed_cost_usd: f64,
}

/// The full estate analysis: derived resource metrics, verified capability,
/// DEA efficiency scores, independence evidence and confidence labels.
#[derive(Debug, Clone, serde::Serialize)]
pub struct Analysis {
    pub derived: Vec<DerivedOut>,
    pub capability: Vec<CapabilityRow>,
    pub dea: Vec<DeaScore>,
    /// Pearson correlations over per-repo axes. Derived axes (energy,
    /// carbon, cost) are linear in wall_minutes BY CONSTRUCTION in round
    /// one and are therefore not reported as independence evidence; the
    /// falsifier pair is wall_minutes ~ size_kb (measured compute vs
    /// code-volume proxy).
    pub independence: Vec<(String, f64)>,
    pub confidence_counts: BTreeMap<String, String>,
}

/// Pearson product-moment correlation coefficient. `NaN` for `n < 2` or
/// when either series has zero variance.
pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n < 2.0 {
        return f64::NAN;
    }
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb): (f64, f64) = (
        a.iter().map(|x| (x - ma).powi(2)).sum(),
        b.iter().map(|y| (y - mb).powi(2)).sum(),
    );
    cov / (va.sqrt() * vb.sqrt())
}

/// Pure analysis core: derive resource metrics, assess verified capability,
/// score DEA efficiency (inputs: wall_minutes, size_kb; outputs:
/// verified_success_runs, releases), and compute independence evidence.
///
/// Deterministic: all repo-keyed aggregation goes through `BTreeMap`
/// (directly, or via `derive_per_repo`/`assess`, which do the same), so
/// iteration order — and therefore `analysis.json` — never depends on
/// input row order.
///
/// A repo present in `repos` with zero rows in `runs` never appears in
/// `derived` (which is keyed off `runs`) and is therefore silently absent
/// from `dea` too — DEA only scores DMUs actually derived from telemetry.
pub fn run_analysis(
    runs: &[RunRow],
    repos: &[RepoRow],
    releases: &[ReleaseRow],
    assumptions: &Assumptions,
) -> Result<Analysis> {
    let derived = derive_per_repo(runs, assumptions);
    let capability = assess(runs, releases, 5);

    let size_of = |repo: &str| -> f64 {
        repos
            .iter()
            .find(|r| r.repo == repo)
            .map(|r| r.size_kb as f64)
            .unwrap_or(0.0)
    };
    let cap_of = |repo: &str| capability.iter().find(|c| c.repo == repo);

    let dmus: Vec<Dmu> = derived
        .iter()
        .map(|d| Dmu {
            name: d.repo.clone(),
            inputs: vec![d.wall_minutes, size_of(&d.repo)],
            outputs: vec![
                cap_of(&d.repo)
                    .map(|c| c.verified_success_runs as f64)
                    .unwrap_or(0.0),
                cap_of(&d.repo).map(|c| c.releases as f64).unwrap_or(0.0),
            ],
        })
        .collect();

    let dea_scores = dea(&dmus).with_context(|| {
        format!(
            "DEA over {} DMUs derived from this snapshot's telemetry \
             (one infeasible LP aborts the whole batch)",
            dmus.len()
        )
    })?;

    let mins: Vec<f64> = derived.iter().map(|d| d.wall_minutes).collect();
    let sizes: Vec<f64> = derived.iter().map(|d| size_of(&d.repo)).collect();
    let outs: Vec<f64> = derived
        .iter()
        .map(|d| {
            cap_of(&d.repo)
                .map(|c| c.verified_success_runs as f64)
                .unwrap_or(0.0)
        })
        .collect();
    let independence = vec![
        ("wall_minutes~size_kb".to_string(), pearson(&mins, &sizes)),
        (
            "wall_minutes~verified_success_runs".to_string(),
            pearson(&mins, &outs),
        ),
    ];

    let confidence_counts = ["wall_minutes", "energy_kwh", "carbon_g", "imputed_cost_usd"]
        .iter()
        .map(|m| (m.to_string(), format!("{:?}", confidence_of(m))))
        .collect();

    let derived_out = derived
        .into_iter()
        .map(|d| DerivedOut {
            repo: d.repo,
            wall_minutes: d.wall_minutes,
            energy_kwh: d.energy_kwh,
            carbon_g: d.carbon_g,
            imputed_cost_usd: d.imputed_cost_usd,
        })
        .collect();

    Ok(Analysis {
        derived: derived_out,
        capability,
        dea: dea_scores,
        independence,
        confidence_counts,
    })
}

/// The `repo` part of a `owner/repo` full name (GitHub repo names cannot
/// themselves contain `/`, so this is unambiguous — unlike collapsing the
/// whole full name with a single separator, which lets `owner "a-b" + repo
/// "c"` and `owner "a" + repo "b-c"` collide on the same flattened string).
fn repo_name(full_name: &str) -> &str {
    full_name.rsplit_once('/').map_or(full_name, |(_, n)| n)
}

/// Serialise `items` to pretty JSON and write to `path`. Errors are
/// returned rather than logged so callers can decide whether a given
/// staging write is fatal (the per-owner repo list, which nothing
/// downstream can proceed without) or recoverable (a single repo's runs
/// or releases, where one bad write must not abort the whole sweep).
fn stage_json<T: serde::Serialize>(path: &Path, items: &[T]) -> Result<()> {
    let json = serde_json::to_string_pretty(items)?;
    fs::write(path, json).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Sweep `owners` via `gh`, writing staging JSON nested under one directory
/// per owner (`<out>/<owner>/repos-<owner>.json`, `<out>/<owner>/runs-<repo
/// name>.json`, `<out>/<owner>/releases-<repo name>.json`). Nesting by owner
/// — rather than flattening `owner/repo` into a single filename — is what
/// keeps staging files unambiguous: two different (owner, repo) pairs can
/// share a flattened string (e.g. owner "a-b" + repo "c" vs. owner "a" +
/// repo "b-c") but never share a (directory, filename) pair, since a repo
/// name alone cannot contain `/`. Resumable: any staging file that already
/// exists is skipped. Per-repo/per-owner errors — including a local
/// serialise/write failure while staging a repo's runs or releases — are
/// logged to stderr and do not abort the sweep; one bad repo must not kill
/// a 400-repo run. The exception is the per-owner repo-list write itself
/// (see `stage_json`'s doc comment): that one is fatal, since nothing
/// downstream can proceed for the owner without it.
fn collect(gh: &dyn GhRunner, owners: &[String], out: &Path, max_runs: usize) -> Result<()> {
    fs::create_dir_all(out).with_context(|| format!("create staging dir {}", out.display()))?;

    for owner in owners {
        let owner_dir = out.join(owner);
        fs::create_dir_all(&owner_dir)
            .with_context(|| format!("create owner staging dir {}", owner_dir.display()))?;

        let repos_path = owner_dir.join(format!("repos-{owner}.json"));
        let repos: Vec<RepoRow> = if repos_path.exists() {
            eprintln!("skip (exists): {}", repos_path.display());
            let text = fs::read_to_string(&repos_path)
                .with_context(|| format!("read {}", repos_path.display()))?;
            serde_json::from_str(&text)
                .with_context(|| format!("parse {}", repos_path.display()))?
        } else {
            match list_repos(gh, owner) {
                Ok(repos) => {
                    // Fatal (unlike the per-repo writes below): nothing
                    // downstream can proceed for this owner without its
                    // repo list, so a local write/serialise failure here
                    // aborts the owner via `?` rather than being logged
                    // and skipped.
                    stage_json(&repos_path, &repos)?;
                    eprintln!(
                        "collected {} repos for owner {} -> {}",
                        repos.len(),
                        owner,
                        repos_path.display()
                    );
                    repos
                }
                Err(e) => {
                    eprintln!("error: list_repos({owner}): {e:#}");
                    continue;
                }
            }
        };

        for repo in &repos {
            if repo.archived {
                eprintln!("skip archived: {}", repo.repo);
                continue;
            }
            let name = repo_name(&repo.repo);

            let runs_path = owner_dir.join(format!("runs-{name}.json"));
            if runs_path.exists() {
                eprintln!("skip (exists): {}", runs_path.display());
            } else {
                match collect_runs(gh, &repo.repo, max_runs) {
                    Ok(runs) => match stage_json(&runs_path, &runs) {
                        Ok(()) => eprintln!(
                            "collected {} runs for {} -> {}",
                            runs.len(),
                            repo.repo,
                            runs_path.display()
                        ),
                        Err(e) => eprintln!("error: stage runs for {}: {e:#}", repo.repo),
                    },
                    Err(e) => eprintln!("error: collect_runs({}): {e:#}", repo.repo),
                }
            }

            let releases_path = owner_dir.join(format!("releases-{name}.json"));
            if releases_path.exists() {
                eprintln!("skip (exists): {}", releases_path.display());
            } else {
                match collect_releases(gh, &repo.repo) {
                    Ok(releases) => match stage_json(&releases_path, &releases) {
                        Ok(()) => eprintln!(
                            "collected {} releases for {} -> {}",
                            releases.len(),
                            repo.repo,
                            releases_path.display()
                        ),
                        Err(e) => eprintln!("error: stage releases for {}: {e:#}", repo.repo),
                    },
                    Err(e) => eprintln!("error: collect_releases({}): {e:#}", repo.repo),
                }
            }
        }
    }

    Ok(())
}

/// Load and concatenate every staging JSON file under `dir` (recursively —
/// staging files are nested one level down per owner) whose name starts
/// with `prefix` and ends with `.json`, sorted by full path first so the
/// concatenation order (and hence any downstream unaggregated output) is
/// deterministic across runs.
fn load_json_prefixed<T: serde::de::DeserializeOwned>(dir: &Path, prefix: &str) -> Result<Vec<T>> {
    let mut paths: Vec<PathBuf> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.into_path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with(prefix) && n.ends_with(".json"))
        })
        .collect();
    paths.sort();

    let mut out = Vec::new();
    for p in paths {
        let text = fs::read_to_string(&p).with_context(|| format!("read {}", p.display()))?;
        let rows: Vec<T> =
            serde_json::from_str(&text).with_context(|| format!("parse {}", p.display()))?;
        out.extend(rows);
    }
    Ok(out)
}

/// Read every staging JSON in `staging`, write the three Parquet snapshots
/// plus `analysis.json` into `snapshot` (created if missing).
fn analyse(staging: &Path, snapshot_dir: &Path) -> Result<()> {
    let runs: Vec<RunRow> = load_json_prefixed(staging, "runs-")?;
    let repos: Vec<RepoRow> = load_json_prefixed(staging, "repos-")?;
    let releases: Vec<ReleaseRow> = load_json_prefixed(staging, "releases-")?;

    fs::create_dir_all(snapshot_dir)
        .with_context(|| format!("create snapshot dir {}", snapshot_dir.display()))?;

    snapshot::write_runs(&snapshot_dir.join("runs.parquet"), &runs)
        .with_context(|| format!("write runs.parquet into {}", snapshot_dir.display()))?;
    snapshot::write_repos(&snapshot_dir.join("repos.parquet"), &repos)
        .with_context(|| format!("write repos.parquet into {}", snapshot_dir.display()))?;
    snapshot::write_releases(&snapshot_dir.join("releases.parquet"), &releases)
        .with_context(|| format!("write releases.parquet into {}", snapshot_dir.display()))?;

    let assumptions = Assumptions::default();
    let analysis = run_analysis(&runs, &repos, &releases, &assumptions).with_context(|| {
        format!(
            "run_analysis for snapshot {} (staging: {})",
            snapshot_dir.display(),
            staging.display()
        )
    })?;

    let analysis_path = snapshot_dir.join("analysis.json");
    let json = serde_json::to_string_pretty(&analysis)?;
    fs::write(&analysis_path, json)
        .with_context(|| format!("write {}", analysis_path.display()))?;
    eprintln!("wrote snapshot to {}", snapshot_dir.display());

    Ok(())
}

const FRONTIER_EPS: f64 = 1e-6;

/// Render `analysis.json` from `snapshot_dir` as `format` ("md" or "json"),
/// to `output` if given, else stdout.
fn report(snapshot_dir: &Path, format: &str, output: Option<&Path>) -> Result<()> {
    let analysis_path = snapshot_dir.join("analysis.json");
    let text = fs::read_to_string(&analysis_path)
        .with_context(|| format!("read {}", analysis_path.display()))?;

    let rendered = match format {
        "json" => text,
        "md" | "markdown" => {
            let v: serde_json::Value = serde_json::from_str(&text)
                .with_context(|| format!("parse {}", analysis_path.display()))?;
            render_markdown(&v)
        }
        other => anyhow::bail!("unsupported format: {other} (expected md or json)"),
    };

    match output {
        Some(path) => {
            fs::write(path, &rendered).with_context(|| format!("write {}", path.display()))?;
            eprintln!("Output written to: {}", path.display());
        }
        None => println!("{rendered}"),
    }

    Ok(())
}

fn render_markdown(v: &serde_json::Value) -> String {
    let mut out = String::new();
    out.push_str("# Estate Economics Report\n\n");

    let wall_minutes_by_repo: BTreeMap<String, f64> = v["derived"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| {
            (
                d["repo"].as_str().unwrap_or_default().to_string(),
                d["wall_minutes"].as_f64().unwrap_or(0.0),
            )
        })
        .collect();
    let verified_by_repo: BTreeMap<String, u64> = v["capability"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|c| {
            (
                c["repo"].as_str().unwrap_or_default().to_string(),
                c["verified_success_runs"].as_u64().unwrap_or(0),
            )
        })
        .collect();

    let mut dea_rows: Vec<(&str, f64, &serde_json::Value)> = v["dea"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|d| {
            (
                d["name"].as_str().unwrap_or_default(),
                d["theta_ccr"].as_f64().unwrap_or(0.0),
                d,
            )
        })
        .collect();
    dea_rows.sort_by(|a, b| a.0.cmp(b.0));

    // Frontier: theta_ccr >= 1.0 - eps
    out.push_str("## Frontier (theta_ccr = 1.0)\n\n");
    out.push_str("| repo | theta_ccr |\n|---|---|\n");
    for (name, theta, _) in dea_rows.iter().filter(|(_, t, _)| *t >= 1.0 - FRONTIER_EPS) {
        out.push_str(&format!("| {name} | {theta:.4} |\n"));
    }
    out.push('\n');

    // Worst-20 off-frontier, with peers
    let mut off_frontier: Vec<&(&str, f64, &serde_json::Value)> = dea_rows
        .iter()
        .filter(|(_, t, _)| *t < 1.0 - FRONTIER_EPS)
        .collect();
    off_frontier.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
    out.push_str("## Worst 20 off-frontier (with peers)\n\n");
    out.push_str("| repo | theta_ccr | peers |\n|---|---|---|\n");
    for (name, theta, d) in off_frontier.into_iter().take(20) {
        let peers: Vec<String> = d["peers"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| {
                let pn = p.get(0).and_then(|x| x.as_str()).unwrap_or_default();
                let pw = p.get(1).and_then(|x| x.as_f64()).unwrap_or(0.0);
                format!("{pn} ({pw:.3})")
            })
            .collect();
        out.push_str(&format!("| {name} | {theta:.4} | {} |\n", peers.join(", ")));
    }
    out.push('\n');

    // X-inefficiency: wall_minutes > 0 && verified_success_runs == 0
    out.push_str("## X-inefficiency\n\n");
    out.push_str(
        "X-inefficiency here is our own framing (Leibenstein has no \
         software-engineering literature): real input consumed, zero \
         verified output produced.\n\n",
    );
    out.push_str("| repo | wall_minutes |\n|---|---|\n");
    for (repo, &wall_minutes) in &wall_minutes_by_repo {
        let verified = verified_by_repo.get(repo).copied().unwrap_or(0);
        if wall_minutes > 0.0 && verified == 0 {
            out.push_str(&format!("| {repo} | {wall_minutes:.2} |\n"));
        }
    }
    out.push('\n');

    // Independence pairs
    out.push_str("## Independence\n\n");
    out.push_str("| pair | pearson r |\n|---|---|\n");
    for pair in v["independence"].as_array().into_iter().flatten() {
        let name = pair.get(0).and_then(|x| x.as_str()).unwrap_or_default();
        let r = pair.get(1).and_then(|x| x.as_f64()).unwrap_or(f64::NAN);
        out.push_str(&format!("| {name} | {r:.4} |\n"));
    }
    out.push('\n');

    // Confidence counts
    out.push_str("## Confidence\n\n");
    out.push_str("| metric | confidence |\n|---|---|\n");
    if let Some(obj) = v["confidence_counts"].as_object() {
        let mut entries: Vec<(&String, &serde_json::Value)> = obj.iter().collect();
        entries.sort_by(|a, b| a.0.cmp(b.0));
        for (metric, level) in entries {
            out.push_str(&format!(
                "| {metric} | {} |\n",
                level.as_str().unwrap_or_default()
            ));
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analysis_produces_dea_and_independence_for_two_repos() {
        use oikosbot_telemetry::rows::*;
        let mk = |repo: &str, dur, concl: &str| RunRow {
            repo: repo.into(),
            run_id: 0,
            workflow_name: "CI".into(),
            workflow_path: "ci.yml".into(),
            event: "push".into(),
            conclusion: concl.into(),
            started_at: String::new(),
            updated_at: String::new(),
            duration_s: dur,
        };
        let runs = vec![
            mk("o/a", 600, "success"),
            mk("o/a", 600, "failure"),
            mk("o/b", 6000, "success"),
            mk("o/b", 600, "failure"),
        ];
        let repos = vec![
            RepoRow {
                repo: "o/a".into(),
                visibility: "public".into(),
                archived: false,
                pushed_at: String::new(),
                size_kb: 100,
            },
            RepoRow {
                repo: "o/b".into(),
                visibility: "public".into(),
                archived: false,
                pushed_at: String::new(),
                size_kb: 5000,
            },
        ];
        let a = run_analysis(&runs, &repos, &[], &Default::default()).unwrap();
        assert_eq!(a.dea.len(), 2);
        assert!(a
            .dea
            .iter()
            .all(|s| s.theta_ccr > 0.0 && s.theta_ccr <= 1.0 + 1e-9));
        assert!(a
            .independence
            .iter()
            .any(|(pair, _)| pair == "wall_minutes~size_kb"));
    }

    #[test]
    fn pearson_perfect_positive_correlation_is_one() {
        let a = [1.0, 2.0, 3.0, 4.0];
        let b = [2.0, 4.0, 6.0, 8.0];
        assert!((pearson(&a, &b) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn pearson_needs_at_least_two_points() {
        assert!(pearson(&[1.0], &[1.0]).is_nan());
    }

    #[test]
    fn repo_name_strips_owner_prefix() {
        assert_eq!(repo_name("owner/repo"), "repo");
        assert_eq!(repo_name("norepo"), "norepo");
    }

    struct Panicky;
    impl GhRunner for Panicky {
        fn api(&self, _path: &str) -> Result<serde_json::Value> {
            panic!("gh api should not be called when staging files already exist");
        }
    }

    #[test]
    fn collect_skips_existing_staging_files() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join("o")).unwrap();
        fs::write(dir.path().join("o").join("repos-o.json"), "[]").unwrap();
        collect(&Panicky, &["o".to_string()], dir.path(), 200).unwrap();
    }

    /// Regression for the filename-collision bug: flattening `owner/repo`
    /// into a single string with `/` -> `-` let owner "a-b" + repo "c" and
    /// owner "a" + repo "b-c" both produce "runs-a-b-c.json", silently
    /// merging two distinct repos' staging files. Nesting under a per-owner
    /// directory keeps them apart because a bare repo name cannot contain
    /// `/`.
    struct TwoOwnersFake;
    impl GhRunner for TwoOwnersFake {
        fn api(&self, path: &str) -> Result<serde_json::Value> {
            if path.starts_with("users/a-b/repos") {
                Ok(serde_json::json!([{
                    "full_name": "a-b/c", "visibility": "public",
                    "archived": false, "pushed_at": "", "size": 1
                }]))
            } else if path.starts_with("users/a/repos") {
                Ok(serde_json::json!([{
                    "full_name": "a/b-c", "visibility": "public",
                    "archived": false, "pushed_at": "", "size": 1
                }]))
            } else if path.contains("/actions/runs") {
                Ok(serde_json::json!({"workflow_runs": []}))
            } else {
                Ok(serde_json::json!([]))
            }
        }
    }

    #[test]
    fn collect_avoids_owner_repo_filename_collision() {
        let dir = tempfile::tempdir().unwrap();
        collect(
            &TwoOwnersFake,
            &["a-b".to_string(), "a".to_string()],
            dir.path(),
            5,
        )
        .unwrap();

        // Old flat scheme would have written BOTH of these to the single
        // path "runs-a-b-c.json"; nested-by-owner keeps them distinct.
        let p1 = dir.path().join("a-b").join("runs-c.json");
        let p2 = dir.path().join("a").join("runs-b-c.json");
        assert!(p1.exists(), "missing {}", p1.display());
        assert!(p2.exists(), "missing {}", p2.display());
        assert_ne!(p1, p2);

        let r1: Vec<oikosbot_telemetry::rows::RunRow> =
            serde_json::from_str(&fs::read_to_string(&p1).unwrap()).unwrap();
        let r2: Vec<oikosbot_telemetry::rows::RunRow> =
            serde_json::from_str(&fs::read_to_string(&p2).unwrap()).unwrap();
        assert!(r1.is_empty() && r2.is_empty()); // fake returns no runs; distinctness is the point

        // A second sweep over both owners now finds every staging file
        // already present and must not call `gh` at all — proving resume
        // correctly recognises BOTH nested files rather than confusing them
        // with a single flattened one.
        collect(
            &Panicky,
            &["a-b".to_string(), "a".to_string()],
            dir.path(),
            5,
        )
        .unwrap();
    }
}
