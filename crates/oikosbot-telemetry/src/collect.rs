// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
//! Collector over the `gh` CLI: resumable, capped fetch of runs, repos, and
//! releases for the estate economics pipeline.
//!
//! Resumability lives one level up (Task 12): the CLI writes one staging
//! JSON per repo and skips repos whose file already exists.

use crate::rows::{ReleaseRow, RepoRow, RunRow};
use anyhow::{bail, Context, Result};

/// Abstraction over `gh api <path>` so the collector can be tested without
/// shelling out or hitting the network.
pub trait GhRunner {
    fn api(&self, path: &str) -> Result<serde_json::Value>;
}

/// Real runner: shells out to the `gh` CLI.
pub struct GhCli;

impl GhRunner for GhCli {
    fn api(&self, path: &str) -> Result<serde_json::Value> {
        let out = std::process::Command::new("gh")
            .args(["api", path])
            .output()
            .context("spawn gh")?;
        if !out.status.success() {
            bail!("gh api {path}: {}", String::from_utf8_lossy(&out.stderr));
        }
        Ok(serde_json::from_slice(&out.stdout)?)
    }
}

/// Parse an RFC3339 UTC timestamp of the exact form `YYYY-MM-DDTHH:MM:SSZ`
/// into seconds since the Unix epoch, via the civil-days algorithm.
fn iso_to_epoch(s: &str) -> i64 {
    let b = s.as_bytes();
    let num = |a: usize, z: usize| -> i64 { s[a..z].parse().unwrap_or(0) };
    if b.len() < 20 {
        return 0;
    }
    let (y, m, d) = (num(0, 4), num(5, 7), num(8, 10));
    let (hh, mm, ss) = (num(11, 13), num(14, 16), num(17, 19));
    let yy = if m <= 2 { y - 1 } else { y };
    let era = yy.div_euclid(400);
    let yoe = yy - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    days * 86_400 + hh * 3600 + mm * 60 + ss
}

/// Fetch up to `max_runs` workflow runs for `repo`, paginating 100 at a time.
pub fn collect_runs(gh: &dyn GhRunner, repo: &str, max_runs: usize) -> Result<Vec<RunRow>> {
    let mut rows = Vec::new();
    let mut page = 1;
    while rows.len() < max_runs {
        let v = gh.api(&format!(
            "repos/{repo}/actions/runs?per_page=100&page={page}"
        ))?;
        let runs = v["workflow_runs"].as_array().cloned().unwrap_or_default();
        if runs.is_empty() {
            break;
        }
        for r in &runs {
            let started = r["run_started_at"].as_str().unwrap_or("").to_string();
            let updated = r["updated_at"].as_str().unwrap_or("").to_string();
            rows.push(RunRow {
                repo: repo.to_string(),
                run_id: r["id"].as_i64().unwrap_or(0),
                workflow_name: r["name"].as_str().unwrap_or("").to_string(),
                workflow_path: r["path"].as_str().unwrap_or("").to_string(),
                event: r["event"].as_str().unwrap_or("").to_string(),
                conclusion: r["conclusion"].as_str().unwrap_or("").to_string(),
                duration_s: (iso_to_epoch(&updated) - iso_to_epoch(&started)).max(0),
                started_at: started,
                updated_at: updated,
            });
            if rows.len() >= max_runs {
                break;
            }
        }
        page += 1;
    }
    Ok(rows)
}

/// List up to 1000 repos owned by `owner` (10 pages of 100 — `gh api`'s
/// per-page cap), via `users/{owner}/repos?type=owner`.
pub fn list_repos(gh: &dyn GhRunner, owner: &str) -> Result<Vec<RepoRow>> {
    let mut out = Vec::new();
    for page in 1..=10 {
        let v = gh.api(&format!(
            "users/{owner}/repos?per_page=100&page={page}&type=owner"
        ))?;
        let arr = v.as_array().cloned().unwrap_or_default();
        if arr.is_empty() {
            break;
        }
        for r in &arr {
            out.push(RepoRow {
                repo: r["full_name"].as_str().unwrap_or("").to_string(),
                visibility: r["visibility"].as_str().unwrap_or("").to_string(),
                archived: r["archived"].as_bool().unwrap_or(false),
                pushed_at: r["pushed_at"].as_str().unwrap_or("").to_string(),
                size_kb: r["size"].as_i64().unwrap_or(0),
            });
        }
    }
    Ok(out)
}

/// Fetch up to 100 releases for `repo`.
pub fn collect_releases(gh: &dyn GhRunner, repo: &str) -> Result<Vec<ReleaseRow>> {
    let v = gh.api(&format!("repos/{repo}/releases?per_page=100"))?;
    Ok(v.as_array()
        .cloned()
        .unwrap_or_default()
        .iter()
        .map(|r| ReleaseRow {
            repo: repo.to_string(),
            tag: r["tag_name"].as_str().unwrap_or("").to_string(),
            published_at: r["published_at"].as_str().unwrap_or("").to_string(),
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Fake(serde_json::Value);
    impl GhRunner for Fake {
        fn api(&self, _path: &str) -> anyhow::Result<serde_json::Value> {
            Ok(self.0.clone())
        }
    }

    #[test]
    fn collect_runs_maps_fields_and_duration() {
        let fake = Fake(serde_json::json!({"workflow_runs":[{
            "id": 42, "name": "CI", "path": ".github/workflows/ci.yml",
            "event": "push", "conclusion": "startup_failure",
            "run_started_at": "2026-08-01T00:00:00Z", "updated_at": "2026-08-01T00:00:00Z"}]}));
        let rows = collect_runs(&fake, "o/r", 50).unwrap();
        assert_eq!(rows[0].conclusion, "startup_failure");
        assert_eq!(rows[0].duration_s, 0);
        assert_eq!(rows[0].workflow_path, ".github/workflows/ci.yml");
    }

    #[test]
    fn list_repos_maps_size_and_visibility() {
        let fake = Fake(
            serde_json::json!([{ "full_name": "o/r", "visibility": "public",
            "archived": false, "pushed_at": "2026-08-01T00:00:00Z", "size": 906 }]),
        );
        let repos = list_repos(&fake, "o").unwrap();
        assert_eq!(repos[0].size_kb, 906);
    }

    #[test]
    fn iso_to_epoch_known_pairs() {
        assert_eq!(iso_to_epoch("1970-01-01T00:00:00Z"), 0);
        assert_eq!(
            iso_to_epoch("2026-08-01T00:05:00Z") - iso_to_epoch("2026-08-01T00:00:00Z"),
            300
        );
        // Month-boundary guard: 23:59 on 07-31 -> 00:01 on 08-01 is 120s, not
        // a huge/negative jump, proving the civil-days arithmetic carries the
        // month/day rollover correctly.
        assert_eq!(
            iso_to_epoch("2026-08-01T00:01:00Z") - iso_to_epoch("2026-07-31T23:59:00Z"),
            120
        );
    }
}
