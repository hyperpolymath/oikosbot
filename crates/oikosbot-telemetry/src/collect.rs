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
fn iso_to_epoch(s: &str) -> Result<i64> {
    let b = s.as_bytes();
    anyhow::ensure!(
        b.len() == 20
            && b.iter().enumerate().all(|(i, c)| match i {
                4 | 7 => *c == b'-',
                10 => *c == b'T',
                13 | 16 => *c == b':',
                19 => *c == b'Z',
                _ => c.is_ascii_digit(),
            }),
        "invalid UTC timestamp {s:?}"
    );
    // ASCII validation above makes these byte boundaries safe.
    let num = |a: usize, z: usize| -> Result<i64> { Ok(s[a..z].parse()?) };
    let (y, m, d) = (num(0, 4)?, num(5, 7)?, num(8, 10)?);
    let (hh, mm, ss) = (num(11, 13)?, num(14, 16)?, num(17, 19)?);
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days_in_month = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    anyhow::ensure!(
        y >= 1970 && d >= 1 && d <= days_in_month && hh < 24 && mm < 60 && ss < 60,
        "out-of-range UTC timestamp {s:?}"
    );
    let yy = if m <= 2 { y - 1 } else { y };
    let era = yy.div_euclid(400);
    let yoe = yy - era * 400;
    let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Ok(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Fetch up to `max_runs` workflow runs for `repo`, paginating 100 at a time.
pub fn collect_runs(gh: &dyn GhRunner, repo: &str, max_runs: usize) -> Result<Vec<RunRow>> {
    let mut rows = Vec::new();
    let mut page = 1;
    while rows.len() < max_runs {
        let v = gh.api(&format!(
            "repos/{repo}/actions/runs?per_page=100&page={page}"
        ))?;
        let runs = v["workflow_runs"]
            .as_array()
            .context("invalid runs response: missing workflow_runs array")?;
        for r in runs {
            let id = r["id"]
                .as_i64()
                .filter(|id| *id > 0)
                .context("invalid run id")?;
            anyhow::ensure!(
                !rows.iter().any(|row: &RunRow| row.run_id == id),
                "duplicate run {id} across pages: collection is not a stable snapshot"
            );
            let started = r["run_started_at"].as_str().unwrap_or("").to_string();
            let updated = r["updated_at"].as_str().unwrap_or("").to_string();
            // `run_started_at` is nullable in GitHub's API (queued/early-state
            // runs); a null/absent value maps to "" above. Never subtract
            // through an empty timestamp — iso_to_epoch("") is epoch zero,
            // which would otherwise inflate duration_s by ~1970 years'
            // worth of seconds. Treat either side missing as duration 0.
            let duration_s = if started.is_empty() || updated.is_empty() {
                0
            } else {
                (iso_to_epoch(&updated)? - iso_to_epoch(&started)?).max(0)
            };
            rows.push(RunRow {
                repo: repo.to_string(),
                run_id: id,
                workflow_name: r["name"].as_str().unwrap_or("").to_string(),
                workflow_path: r["path"].as_str().unwrap_or("").to_string(),
                event: r["event"].as_str().unwrap_or("").to_string(),
                // `conclusion` is null for in-progress runs and maps to ""
                // here; downstream counting must treat "" as neither
                // success nor failure, not as a distinct outcome bucket.
                conclusion: r["conclusion"].as_str().unwrap_or("").to_string(),
                duration_s,
                started_at: started,
                updated_at: updated,
            });
            if rows.len() >= max_runs {
                break;
            }
        }
        if runs.len() < 100 {
            if let Some(total) = v.get("total_count") {
                let total = total.as_u64().context("invalid workflow run total_count")?;
                anyhow::ensure!(
                    rows.len() as u64 >= total.min(max_runs as u64),
                    "truncated run history: short page contradicts total_count"
                );
            }
            break;
        }
        page += 1;
    }
    Ok(rows)
}

/// List up to 1000 repos owned by `owner` (10 pages of 100 — `gh api`'s
/// per-page cap), via `users/{owner}/repos?type=owner`.
pub fn list_repos(gh: &dyn GhRunner, owner: &str) -> Result<Vec<RepoRow>> {
    let mut out = Vec::new();
    // The extra page distinguishes exactly 1000 repos from truncation.
    for page in 1..=11 {
        let v = gh.api(&format!(
            "users/{owner}/repos?per_page=100&page={page}&type=owner"
        ))?;
        let arr = v
            .as_array()
            .context("invalid repository response: expected array")?;
        if arr.is_empty() {
            break;
        }
        anyhow::ensure!(
            page <= 10,
            "estate exceeds 1000 repository cap; refusing truncated inventory"
        );
        for r in arr {
            let name = r["full_name"]
                .as_str()
                .filter(|s| s.contains('/'))
                .context("repository is missing owner/name identity")?;
            anyhow::ensure!(
                !out.iter().any(|row: &RepoRow| row.repo == name),
                "duplicate repository {name}: unstable pagination"
            );
            out.push(RepoRow {
                repo: name.to_string(),
                visibility: r["visibility"].as_str().unwrap_or("").to_string(),
                archived: r["archived"].as_bool().unwrap_or(false),
                pushed_at: r["pushed_at"].as_str().unwrap_or("").to_string(),
                size_kb: r["size"].as_i64().unwrap_or(0),
            });
        }
        if arr.len() < 100 {
            break;
        }
    }
    Ok(out)
}

/// Fetch public release history. A safety cap is an error, never a complete
/// empty or truncated history. Draft releases do not count as publication.
pub fn collect_releases(gh: &dyn GhRunner, repo: &str) -> Result<Vec<ReleaseRow>> {
    let mut rows = Vec::new();
    for page in 1..=101 {
        let v = gh.api(&format!("repos/{repo}/releases?per_page=100&page={page}"))?;
        let releases = v
            .as_array()
            .context("invalid releases response: expected array")?;
        anyhow::ensure!(
            page <= 100 || releases.is_empty(),
            "release history exceeds collection cap"
        );
        for r in releases {
            if r["draft"].as_bool() == Some(true) {
                continue;
            }
            let tag = r["tag_name"]
                .as_str()
                .filter(|s| !s.is_empty())
                .context("missing release tag")?;
            anyhow::ensure!(
                !rows.iter().any(|row: &ReleaseRow| row.tag == tag),
                "duplicate release tag {tag}"
            );
            rows.push(ReleaseRow {
                repo: repo.to_string(),
                tag: tag.to_string(),
                published_at: r["published_at"].as_str().unwrap_or("").to_string(),
            });
        }
        if releases.len() < 100 {
            return Ok(rows);
        }
    }
    bail!("release pagination did not terminate")
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
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].conclusion, "startup_failure");
        assert_eq!(rows[0].duration_s, 0);
        assert_eq!(rows[0].workflow_path, ".github/workflows/ci.yml");
    }

    #[test]
    fn collect_runs_zeroes_duration_when_started_at_is_null() {
        let fake = Fake(serde_json::json!({"workflow_runs":[{
            "id": 43, "name": "CI", "path": ".github/workflows/ci.yml",
            "event": "push", "conclusion": null,
            "run_started_at": null, "updated_at": "2026-08-01T00:05:00Z"}]}));
        let rows = collect_runs(&fake, "o/r", 50).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(
            rows[0].duration_s, 0,
            "null run_started_at must not be treated as epoch zero"
        );
        assert_eq!(rows[0].conclusion, "");
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
        assert_eq!(iso_to_epoch("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            iso_to_epoch("2026-08-01T00:05:00Z").unwrap()
                - iso_to_epoch("2026-08-01T00:00:00Z").unwrap(),
            300
        );
        // Month-boundary guard: 23:59 on 07-31 -> 00:01 on 08-01 is 120s, not
        // a huge/negative jump, proving the civil-days arithmetic carries the
        // month/day rollover correctly.
        assert_eq!(
            iso_to_epoch("2026-08-01T00:01:00Z").unwrap()
                - iso_to_epoch("2026-07-31T23:59:00Z").unwrap(),
            120
        );
    }
}

#[cfg(test)]
mod boundary_tests {
    use super::*;
    use std::cell::RefCell;
    use std::collections::VecDeque;

    struct Pages(RefCell<VecDeque<Result<serde_json::Value>>>);
    impl GhRunner for Pages {
        fn api(&self, _path: &str) -> Result<serde_json::Value> {
            self.0
                .borrow_mut()
                .pop_front()
                .expect("unexpected extra request")
        }
    }
    fn pages(values: Vec<Result<serde_json::Value>>) -> Pages {
        Pages(RefCell::new(values.into()))
    }
    fn runs(start: i64, count: usize) -> serde_json::Value {
        serde_json::json!({"workflow_runs": (0..count).map(|i| serde_json::json!({
            "id": start + i as i64, "name": "CI", "conclusion": "success",
            "run_started_at": "2026-08-01T00:00:00Z", "updated_at": "2026-08-01T00:01:00Z"
        })).collect::<Vec<_>>()})
    }

    #[test]
    fn two_pages_are_collected_once_and_short_page_terminates() {
        let gh = pages(vec![Ok(runs(1, 100)), Ok(runs(101, 1))]);
        let rows = collect_runs(&gh, "o/r", 200).unwrap();
        assert_eq!(rows.len(), 101);
        assert_eq!(rows[100].run_id, 101);
    }

    #[test]
    fn malformed_rate_limited_and_duplicate_pages_cannot_become_empty_success() {
        let gh = pages(vec![Ok(serde_json::json!({"message":"rate limited"}))]);
        assert!(collect_runs(&gh, "o/r", 200).is_err());
        let gh = pages(vec![Ok(runs(1, 100)), Err(anyhow::anyhow!("HTTP 429"))]);
        assert!(collect_runs(&gh, "o/r", 200).is_err());
        let gh = pages(vec![Ok(runs(1, 100)), Ok(runs(1, 1))]);
        assert!(collect_runs(&gh, "o/r", 200).is_err());
        let gh = pages(vec![Ok(serde_json::json!({}))]);
        assert!(list_repos(&gh, "o").is_err());
        let gh = pages(vec![Ok(serde_json::json!({}))]);
        assert!(collect_releases(&gh, "o/r").is_err());
    }

    #[test]
    fn short_page_cannot_claim_a_larger_declared_history() {
        let mut page = runs(1, 1);
        page["total_count"] = serde_json::json!(10);
        assert!(collect_runs(&pages(vec![Ok(page)]), "o/r", 200).is_err());
    }

    #[test]
    fn invalid_timestamps_are_errors_not_epoch_zero_or_utf8_panics() {
        for timestamp in [
            "",
            "💥💥💥💥💥",
            "2026-02-30T00:00:00Z",
            "2026-08-01T99:00:00Z",
        ] {
            assert!(iso_to_epoch(timestamp).is_err());
        }
        assert!(iso_to_epoch("2024-02-29T00:00:00Z").is_ok());
    }

    #[test]
    fn repository_inventory_cap_is_not_silent_truncation() {
        let mut responses = Vec::new();
        for page in 0..11 {
            responses.push(Ok(serde_json::json!((0..100)
                .map(|i| serde_json::json!({
                    "full_name": format!("o/r{}", page * 100 + i)
                }))
                .collect::<Vec<_>>())));
        }
        assert!(list_repos(&pages(responses), "o")
            .unwrap_err()
            .to_string()
            .contains("cap"));
    }
}
