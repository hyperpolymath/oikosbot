// SPDX-License-Identifier: MPL-2.0
//! Verified capability: output metrics derived from run history alone.
//! A gate with N successes and zero failures in its whole history cannot
//! be shown to be able to fail; its "green" is not evidence, so its
//! successes are excluded from verified output. This is the telemetry
//! operationalisation of the estate's fake-gate pathology.
use oikosbot_telemetry::rows::{ReleaseRow, RunRow};
use std::collections::BTreeMap;

#[derive(Debug, Clone, serde::Serialize)]
pub struct CapabilityRow {
    pub repo: String,
    pub runs_total: u64,
    pub startup_failures: u64,
    pub parse_dead_runs: u64,
    pub workflows_seen: u64,
    pub workflows_fallible: u64,
    /// Workflow **paths** (not names) with N successes and zero failures in
    /// their whole history. Grouping by path rather than the free-text
    /// `workflow_name` matters: two different files can share a name (e.g.
    /// two workflows both titled "CI"), and grouping by name would merge
    /// their run counts, potentially hiding a fake-gate candidate behind a
    /// same-named failing workflow. Paths are unique per file, so this is
    /// also more precise for humans tracking down the offending file.
    pub infallible_gate_candidates: Vec<String>,
    pub verified_success_runs: u64,
    pub releases: u64,
}

pub fn assess(runs: &[RunRow], releases: &[ReleaseRow], min_n: u64) -> Vec<CapabilityRow> {
    let mut by_repo: BTreeMap<&str, Vec<&RunRow>> = BTreeMap::new();
    for r in runs {
        by_repo.entry(&r.repo).or_default().push(r);
    }
    let mut rel_count: BTreeMap<&str, u64> = BTreeMap::new();
    for r in releases {
        *rel_count.entry(&r.repo).or_default() += 1;
    }

    by_repo
        .into_iter()
        .map(|(repo, rs)| {
            // Keyed by workflow_path (unique per file), not workflow_name
            // (free-text, non-unique — see the doc comment on
            // `infallible_gate_candidates`).
            let mut wf: BTreeMap<&str, (u64, u64)> = BTreeMap::new(); // (successes, failures)
            let mut startup = 0u64;
            let mut parse_dead = 0u64;
            for r in &rs {
                if r.conclusion == "startup_failure" {
                    startup += 1;
                }
                if r.workflow_name == r.workflow_path && !r.workflow_path.is_empty() {
                    parse_dead += 1;
                }
                let e = wf.entry(&r.workflow_path).or_default();
                match r.conclusion.as_str() {
                    "success" => e.0 += 1,
                    "failure" | "startup_failure" | "timed_out" => e.1 += 1,
                    _ => {} // in-progress runs have conclusion == ""; neither success nor failure
                }
            }
            let infallible: Vec<String> = wf
                .iter()
                .filter(|(_, (s, f))| *f == 0 && *s >= min_n)
                .map(|(n, _)| n.to_string())
                .collect();
            let fallible = wf.values().filter(|(_, f)| *f > 0).count() as u64;
            let verified: u64 = wf.values().filter(|(_, f)| *f > 0).map(|(s, _)| s).sum();
            CapabilityRow {
                repo: repo.to_string(),
                runs_total: rs.len() as u64,
                startup_failures: startup,
                parse_dead_runs: parse_dead,
                workflows_seen: wf.len() as u64,
                workflows_fallible: fallible,
                infallible_gate_candidates: infallible,
                verified_success_runs: verified,
                releases: *rel_count.get(repo).unwrap_or(&0),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use oikosbot_telemetry::rows::RunRow;

    fn run(repo: &str, wf: &str, path: &str, concl: &str) -> RunRow {
        RunRow {
            repo: repo.into(),
            run_id: 0,
            workflow_name: wf.into(),
            workflow_path: path.into(),
            event: "push".into(),
            conclusion: concl.into(),
            started_at: String::new(),
            updated_at: String::new(),
            duration_s: 60,
        }
    }

    #[test]
    fn startup_failures_and_parse_dead_are_counted() {
        // echidna pattern: repo-total startup_failure
        let runs: Vec<_> = (0..3)
            .map(|_| run("o/dead", "CI", "x", "startup_failure"))
            .collect();
        // parse-rejected pattern: run named by its own file path
        let mut all = runs;
        all.push(run(
            "o/dead",
            ".github/workflows/m.yml",
            ".github/workflows/m.yml",
            "failure",
        ));
        let c = assess(&all, &[], 5);
        assert_eq!(c[0].startup_failures, 3);
        assert_eq!(c[0].parse_dead_runs, 1);
    }

    #[test]
    fn gate_that_never_failed_is_flagged_and_excluded_from_verified_output() {
        // fake-gate pattern: N successes, zero failures → candidate;
        // its successes must NOT count as verified output.
        let mut all: Vec<_> = (0..6)
            .map(|_| run("o/fake", "Gate", "g.yml", "success"))
            .collect();
        all.push(run("o/fake", "Real", "r.yml", "failure"));
        all.push(run("o/fake", "Real", "r.yml", "success"));
        let c = assess(&all, &[], 5);
        assert_eq!(c[0].infallible_gate_candidates, vec!["g.yml".to_string()]);
        assert_eq!(c[0].workflows_fallible, 1);
        assert_eq!(c[0].verified_success_runs, 1); // only Real's success counts
    }

    #[test]
    fn same_workflow_name_different_paths_are_not_merged() {
        // Regression: two workflows named "CI" but living at different
        // paths must be assessed independently. Grouping by name (the old
        // bug) would merge a.yml's 6 clean successes with b.yml's 1
        // success + 1 failure into a single "CI" entry with a failure,
        // hiding a.yml's fake-gate status entirely.
        let mut all: Vec<_> = (0..6)
            .map(|_| run("o/collide", "CI", "a.yml", "success"))
            .collect();
        all.push(run("o/collide", "CI", "b.yml", "success"));
        all.push(run("o/collide", "CI", "b.yml", "failure"));
        let c = assess(&all, &[], 5);
        assert_eq!(c[0].infallible_gate_candidates, vec!["a.yml".to_string()]);
        assert_eq!(c[0].verified_success_runs, 1); // only b.yml's success counts
    }
}
