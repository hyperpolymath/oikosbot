# Estate Economics Round One — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Land the Part 0 repairs, then build the read-only estate pipeline: telemetry collector → snapshot → derived metrics → DEA frontier → report.

**Architecture:** Three new library crates in the existing `oikosbot` Cargo workspace plus an `estate` CLI subcommand. Collection shells out to the authenticated `gh` CLI (no auth code, resumable). Snapshots are Parquet files committed to a new `oikosbot-estate` dataset repo. DEA is solved natively as paired envelopment/multiplier LPs on `good_lp`+HiGHS, giving θ, peer sets, and dual weights with a built-in strong-duality cross-check. Nothing blocks, nothing is enforced.

**Tech Stack:** Rust (existing workspace), `good_lp` (HiGHS backend, MIT), `parquet`/`parquet_derive`, `serde_json`, `anyhow`, `gh` CLI subprocess.

**Spec:** `docs/superpowers/specs/2026-08-03-estate-economics-design.md` (committed in Task 1 from `~/.claude/plans/oikosbot-sitrep-effervescent-crescent.md`).

## Global Constraints

- Workspace root: `/home/hyperpolymath/developer/hyper-repos/_EXTENSIONS _SET/oikosbot` — **path contains spaces; always quote**. Use `OIKOSBOT="/home/hyperpolymath/developer/hyper-repos/_EXTENSIONS _SET/oikosbot"` in every shell step.
- Crate naming: new crates are `oikosbot-*` (NOT `oikos-*` — that prefix belongs to the accounting DSL per `DISAMBIGUATION.adoc`). The dataset repo is `oikosbot-estate` for the same reason.
- Every new source file starts with `// SPDX-License-Identifier: MPL-2.0` (YAML/MD files: `# SPDX-License-Identifier: MPL-2.0` where the format allows comments).
- `good_lp` MUST be `default-features = false, features = ["highs"]` (the CBC default is EPL-licensed and stale).
- No Python anywhere. No new GitHub Actions in this round (estate Actions are dead pending the owner's lockfile fix; verification is local).
- `cargo fmt --all` and `cargo clippy --workspace --all-targets -- -D warnings` must pass before every commit.
- Confidence labels are load-bearing: money-from-declared-rate and energy are at best `Calibrated`; carbon is at most `Estimated`; only API-sourced quantities are `Measured`. A test asserts this (Task 10).
- Do not touch the uncommitted `.editorconfig`/`.gitignore` edits in the checkout — they are the owner's.
- CI cannot run (estate-wide `startup_failure`); PRs are created but verified by file inspection and local tests. Do not claim CI-green anywhere.

---

## Phase A — Part 0 repairs

### Task 1: Sync checkout and commit the design spec

**Files:**
- Create: `docs/superpowers/specs/2026-08-03-estate-economics-design.md` (copy of the approved plan file)
- Create: `docs/superpowers/plans/2026-08-03-estate-economics-round-one.md` (this file)

**Interfaces:**
- Consumes: approved design at `~/.claude/plans/oikosbot-sitrep-effervescent-crescent.md`
- Produces: branch `feat/estate-round-one` carrying the spec; local `main` in sync with `origin/main` at `7e73ddf` (#57)

- [ ] **Step 1: Sync main (leave the owner's dirty files alone)**

```bash
OIKOSBOT="/home/hyperpolymath/developer/hyper-repos/_EXTENSIONS _SET/oikosbot"
git -C "$OIKOSBOT" fetch origin
git -C "$OIKOSBOT" status -sb          # expect only .editorconfig/.gitignore dirty
git -C "$OIKOSBOT" merge --ff-only origin/main
git -C "$OIKOSBOT" log --oneline -1    # expect 7e73ddf (#57)
```

If `merge --ff-only` fails because the dirty files conflict, STOP and report — do not stash or discard the owner's edits.

- [ ] **Step 2: Create the round-one branch and commit spec + plan**

```bash
git -C "$OIKOSBOT" switch -c feat/estate-round-one origin/main
mkdir -p "$OIKOSBOT/docs/superpowers/specs" "$OIKOSBOT/docs/superpowers/plans"
cp ~/.claude/plans/oikosbot-sitrep-effervescent-crescent.md \
   "$OIKOSBOT/docs/superpowers/specs/2026-08-03-estate-economics-design.md"
git -C "$OIKOSBOT" add docs/superpowers
git -C "$OIKOSBOT" commit -m "docs: estate-economics design spec + round-one implementation plan"
```

### Task 2: Fix the two parse-dead workflows (illegal `timeout-minutes` on `uses:` jobs)

**Files:**
- Modify: `.github/workflows/mirror.yml` (remove one line)
- Modify: `.github/workflows/secret-scanner.yml` (remove one line)

**Interfaces:**
- Produces: PR `fix/uses-job-timeout` against `main`. Merge is the owner's call or post-verification (estate doctrine: Claude merges once verified — but CI cannot verify until the lockfile fix, so leave the PR open with that stated).

- [ ] **Step 1: Branch from origin/main**

```bash
git -C "$OIKOSBOT" switch -c fix/uses-job-timeout origin/main
```

- [ ] **Step 2: Remove the illegal key from both files**

In `.github/workflows/mirror.yml`, the job currently reads:

```yaml
jobs:
  mirror:
    uses: hyperpolymath/standards/.github/workflows/mirror-reusable.yml@d135b05bfc647d0c0fbfedc7e80f37ea50f49236
    timeout-minutes: 10
    secrets: inherit
```

Delete the `timeout-minutes: 10` line (a job whose body is `uses:` may not carry it; its presence invalidates the whole file — the same defect PR #56 fixed in `scorecard.yml`). Repeat for the `scan:` job in `.github/workflows/secret-scanner.yml`.

- [ ] **Step 3: Verify no workflow retains the defect**

```bash
for f in "$OIKOSBOT"/.github/workflows/*.yml; do
  awk '/^    uses:/{u=1} /^    timeout-minutes:/{if(u) print FILENAME": job-level timeout-minutes after uses:"} /^  [a-z]/{u=0}' "$f"
done
```

Expected: no output.

- [ ] **Step 4: Commit and open the PR**

```bash
git -C "$OIKOSBOT" add .github/workflows/mirror.yml .github/workflows/secret-scanner.yml
git -C "$OIKOSBOT" commit -m "fix(ci): drop illegal timeout-minutes on reusable-workflow jobs

A job whose body is 'uses:' may not carry timeout-minutes; the key
invalidates the entire file (conclusion 'failure' at 0s, run named by
file path). PR #56 fixed scorecard.yml and missed these two."
git -C "$OIKOSBOT" push -u origin fix/uses-job-timeout
gh pr create -R hyperpolymath/oikosbot --head fix/uses-job-timeout \
  --title "fix(ci): drop illegal timeout-minutes on uses: jobs (mirror, secret-scanner)" \
  --body "Same defect class as #56, which missed these two files. NOTE: repo-wide startup_failure (GitHub lockfile enforcement) means no checks will run on this PR; verified by file inspection. Merge after the owner's gh-actions-lock fix, or on inspection."
```

### Task 3: Un-gate the proven pilot (enaction-engine)

**Files:**
- Modify (in a fresh clone): `metadatastician/enaction-engine` → `.github/workflows/oikosbot.yml`

**Interfaces:**
- Produces: PR to `metadatastician/enaction-engine` removing `if: vars.OIKOSBOT_ENABLED == 'true'` and its stale justification comment.

- [ ] **Step 1: Clone into the job tmp dir**

```bash
TMP="$CLAUDE_JOB_DIR/tmp"
git clone --depth 1 git@github.com:metadatastician/enaction-engine.git "$TMP/enaction-engine"
cd "$TMP/enaction-engine" && git switch -c fix/ungate-oikosbot
```

- [ ] **Step 2: Remove the gate and the stale comment**

In `.github/workflows/oikosbot.yml`: delete the `if: vars.OIKOSBOT_ENABLED == 'true'` line and the multi-line comment block above it that claims the image "has never been published" (stale: publish-image went green 2026-07-28; the image is podman-verified and the job already uses the composite action, not the container pull the comment describes). Replace the comment with:

```yaml
    # Runs the published composite action (hyperpolymath/oikosbot@v0.1.0).
    # This repo is the proven pilot: 101 SARIF results ingested on PR #21.
```

- [ ] **Step 3: Commit and open the PR**

```bash
git add .github/workflows/oikosbot.yml
git commit -m "fix(ci): remove stale OIKOSBOT_ENABLED gate

The gate's justification (image never published) has been false since
2026-07-28: publish-image is green and the image is verified. With no
repo variable set, the gate silently disabled the one proven pilot."
git push -u origin fix/ungate-oikosbot
gh pr create -R metadatastician/enaction-engine --head fix/ungate-oikosbot \
  --title "fix(ci): remove stale OIKOSBOT_ENABLED gate on the oikosbot job" \
  --body "Premise ('image has never been published') stale since 2026-07-28. This repo is the proven pilot (101 results ingested, PR #21); the gate turned it off. NOTE: estate-wide startup_failure means checks will not run here until the lockfile fix."
```

### Task 4: Trace the two unresolvable consumer repos

**Files:**
- Create: `docs/superpowers/notes/2026-08-03-consumer-repo-trace.md` (on `feat/estate-round-one`)

**Interfaces:**
- Produces: written disposition for `hyperpolymath/boj-server-mk2` and `hyperpolymath/idaptik-ums` (renamed → new name; deleted; or transferred), so the consumer list is accurate for later sweeps.

- [ ] **Step 1: Probe both names and likely successors**

```bash
for r in hyperpolymath/boj-server-mk2 metadatastician/boj-server-mk2 hyperpolymath/boj-server metadatastician/boj-server \
         hyperpolymath/idaptik-ums metadatastician/idaptik-ums metadatastician/canonical-ums hyperpolymath/canonical-ums; do
  echo "== $r"; gh repo view "$r" --json name,owner,isArchived,url -q '"\(.owner.login)/\(.name) archived=\(.isArchived)"' 2>&1
done
# Renames follow redirects on the API; also check:
gh api repos/hyperpolymath/boj-server-mk2 -q .full_name 2>&1
gh api repos/hyperpolymath/idaptik-ums   -q .full_name 2>&1
```

- [ ] **Step 2: Search the owners' repo lists for near-names**

```bash
gh repo list hyperpolymath  --limit 400 --json name -q '.[].name' | grep -iE 'boj|ums|idaptik'
gh repo list metadatastician --limit 400 --json name -q '.[].name' | grep -iE 'boj|ums|idaptik'
```

- [ ] **Step 3: Record findings and commit**

Write the disposition table (old name → found-at / not-found, evidence commands) to `docs/superpowers/notes/2026-08-03-consumer-repo-trace.md`, then:

```bash
git -C "$OIKOSBOT" switch feat/estate-round-one
git -C "$OIKOSBOT" add docs/superpowers/notes
git -C "$OIKOSBOT" commit -m "docs: trace unresolvable oikosbot consumer repos"
```

### Task 5: Make the Eclexia builtin backend loud, and annotate the over-promising config

**Files:**
- Modify: `crates/oikosbot-eclexia/src/lib.rs` (the `evaluate_builtin` path)
- Modify: `config/oikos.yaml` (comment annotations only)
- Test: `crates/oikosbot-eclexia/src/lib.rs` (unit test in-module)

**Interfaces:**
- Consumes: existing `evaluate_policies(dir, results)` flow; existing `evaluate_builtin` dispatch-by-file-stem.
- Produces: a `::warning::` on stderr every time the builtin path is used, naming the file and stating that its contents were not parsed; a pub fn `builtin_warning(policy_stem: &str) -> String` so the text is testable.

- [ ] **Step 1: Write the failing test**

In `crates/oikosbot-eclexia/src/lib.rs` tests module:

```rust
#[test]
fn builtin_warning_names_file_and_admits_not_parsing() {
    let w = builtin_warning("energy_threshold");
    assert!(w.contains("energy_threshold.ecl"));
    assert!(w.contains("NOT parsed"));
    assert!(w.starts_with("::warning::"));
}
```

- [ ] **Step 2: Run it to make sure it fails**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-eclexia builtin_warning -- --nocapture
```

Expected: FAIL (`builtin_warning` not found).

- [ ] **Step 3: Implement**

```rust
/// The builtin backend matches policies by FILE STEM and applies hardcoded
/// thresholds; the .ecl contents are never read. Until the eclexia parser is
/// wired (round two), every builtin evaluation must say so out loud.
pub fn builtin_warning(policy_stem: &str) -> String {
    format!(
        "::warning::policy {policy_stem}.ecl evaluated by BUILTIN fallback: \
         file contents NOT parsed; hardcoded thresholds applied (may disagree \
         with the .ecl text). Install `eclexia` or await the native backend."
    )
}
```

In `evaluate_builtin`, before returning its decision, add:

```rust
eprintln!("{}", builtin_warning(stem));
```

- [ ] **Step 4: Run test, fmt, clippy**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-eclexia && cargo fmt --all && cargo clippy -p oikosbot-eclexia --all-targets -- -D warnings
```

Expected: PASS, clean.

- [ ] **Step 5: Annotate `config/oikos.yaml`**

Above each block the loader parses-and-discards (`integrations:`, `ai_assistants:`, `praxis:`, `reporting:`, `notifications:`, `logging:`, `databases:`, `weights:`, `analysis.carbon:`, `analysis.energy:`, `analysis.complexity:`, `thresholds.eco_standard/eco_excellence`), insert:

```yaml
# ASPIRATIONAL — parsed and DISCARDED by the current loader (config.rs).
# Documented for the target design; nothing below this line takes effect yet.
```

- [ ] **Step 6: Commit and open the honesty PR**

```bash
git -C "$OIKOSBOT" switch -c fix/honesty-punchlist origin/main
git -C "$OIKOSBOT" add crates/oikosbot-eclexia config/oikos.yaml
git -C "$OIKOSBOT" commit -m "fix: loud builtin-policy warning; mark aspirational config blocks

The eclexia builtin backend dispatches on file stem and never reads the
.ecl contents; say so on every use instead of silently applying
thresholds that contradict the files. config/oikos.yaml declared
integrations that do not exist; they are now labelled ASPIRATIONAL."
git -C "$OIKOSBOT" push -u origin fix/honesty-punchlist
gh pr create -R hyperpolymath/oikosbot --head fix/honesty-punchlist \
  --title "fix: loud builtin-policy warning; label aspirational config" \
  --body "Part 0 items 4 and 6 of the estate-economics design. No behaviour change beyond stderr. CI dead estate-wide; verified by local cargo test."
```

---

## Phase B — Round one pipeline

All Phase B work happens on `feat/estate-round-one`. Run `git -C "$OIKOSBOT" switch feat/estate-round-one` before each task.

### Task 6: Scaffold the three crates

**Files:**
- Create: `crates/oikosbot-telemetry/Cargo.toml`, `crates/oikosbot-telemetry/src/lib.rs`
- Create: `crates/oikosbot-capability/Cargo.toml`, `crates/oikosbot-capability/src/lib.rs`
- Create: `crates/oikosbot-dea/Cargo.toml`, `crates/oikosbot-dea/src/lib.rs`
- Modify: root `Cargo.toml` ONLY if `members` is an explicit list (check first; if it globs `crates/*`, no change)

**Interfaces:**
- Produces: three empty compiling library crates in the workspace.

- [ ] **Step 1: Check workspace membership style**

```bash
grep -A5 '^\[workspace\]' "$OIKOSBOT/Cargo.toml"
```

If `members` lists crates explicitly, the three new paths must be appended; if it uses `"crates/*"`, skip.

- [ ] **Step 2: Create the crates (mirror an existing crate's edition/metadata)**

Each `Cargo.toml` copies the `[package]` metadata style of `crates/oikosbot-pareto/Cargo.toml` (same edition, license MPL-2.0, version 0.1.0). Dependencies:

`oikosbot-telemetry`: `serde = { version = "1", features = ["derive"] }`, `serde_json = "1"`, `anyhow = "1"`, `parquet = "56"`, `parquet_derive = "56"`, and `oikosbot-metrics = { path = "../oikosbot-metrics" }`. (Run `cargo add parquet parquet_derive` to resolve the current compatible pair rather than trusting "56"; they must be the same version.)

`oikosbot-capability`: `serde`, `serde_json`, `anyhow`, `oikosbot-telemetry = { path = "../oikosbot-telemetry" }`.

`oikosbot-dea`: `anyhow`, `good_lp = { version = "1.15", default-features = false, features = ["highs"] }`.

Each `src/lib.rs` starts as:

```rust
// SPDX-License-Identifier: MPL-2.0
//! (one-line purpose)
```

- [ ] **Step 3: Verify the workspace builds and commit**

```bash
cd "$OIKOSBOT" && cargo check --workspace
git add crates/oikosbot-telemetry crates/oikosbot-capability crates/oikosbot-dea Cargo.toml Cargo.lock
git commit -m "chore: scaffold oikosbot-telemetry, oikosbot-capability, oikosbot-dea"
```

### Task 7: Telemetry row types + Parquet snapshot round-trip

**Files:**
- Create: `crates/oikosbot-telemetry/src/rows.rs`
- Create: `crates/oikosbot-telemetry/src/snapshot.rs`
- Modify: `crates/oikosbot-telemetry/src/lib.rs` (`pub mod rows; pub mod snapshot;`)

**Interfaces:**
- Produces (relied on by Tasks 8–13):

```rust
pub struct RunRow { pub repo: String, pub run_id: i64, pub workflow_name: String,
    pub workflow_path: String, pub event: String, pub conclusion: String,
    pub started_at: String, pub updated_at: String, pub duration_s: i64 }
pub struct RepoRow { pub repo: String, pub visibility: String, pub archived: bool,
    pub pushed_at: String, pub size_kb: i64 }
pub struct ReleaseRow { pub repo: String, pub tag: String, pub published_at: String }
// snapshot module:
pub fn write_runs(path: &Path, rows: &[RunRow]) -> anyhow::Result<()>
pub fn read_runs(path: &Path) -> anyhow::Result<Vec<RunRow>>   // same pairs for RepoRow, ReleaseRow
```

- [ ] **Step 1: Write the failing round-trip test**

In `snapshot.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::rows::RunRow;
    #[test]
    fn parquet_roundtrip_preserves_rows() {
        let rows = vec![RunRow {
            repo: "o/r".into(), run_id: 1, workflow_name: "CI".into(),
            workflow_path: ".github/workflows/ci.yml".into(), event: "push".into(),
            conclusion: "success".into(), started_at: "2026-08-01T00:00:00Z".into(),
            updated_at: "2026-08-01T00:05:00Z".into(), duration_s: 300 }];
        let dir = std::env::temp_dir().join("oikos-snap-test");
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("runs.parquet");
        write_runs(&p, &rows).unwrap();
        let back = read_runs(&p).unwrap();
        assert_eq!(back.len(), 1);
        assert_eq!(back[0].repo, "o/r");
        assert_eq!(back[0].duration_s, 300);
    }
}
```

- [ ] **Step 2: Run it to make sure it fails**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-telemetry parquet_roundtrip
```

Expected: FAIL (types/functions not defined).

- [ ] **Step 3: Implement rows and snapshot**

`rows.rs`: the three structs above, each `#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize, parquet_derive::ParquetRecordWriter)]`.

`snapshot.rs` — write via `parquet_derive`, read via the stable `RowAccessor` API with column indices matching struct field order:

```rust
use anyhow::{Context, Result};
use parquet::file::properties::WriterProperties;
use parquet::file::reader::{FileReader, SerializedFileReader};
use parquet::file::writer::SerializedFileWriter;
use parquet::record::{RecordWriter, RowAccessor};
use std::{fs::File, path::Path, sync::Arc};
use crate::rows::RunRow;

pub fn write_runs(path: &Path, rows: &[RunRow]) -> Result<()> {
    let schema = rows.as_slice().schema().context("derive schema")?;
    let file = File::create(path)?;
    let mut w = SerializedFileWriter::new(file, schema, Arc::new(WriterProperties::builder().build()))?;
    let mut rg = w.next_row_group()?;
    rows.as_slice().write_to_row_group(&mut rg)?;
    rg.close()?;
    w.close()?;
    Ok(())
}

pub fn read_runs(path: &Path) -> Result<Vec<RunRow>> {
    let reader = SerializedFileReader::new(File::open(path)?)?;
    let mut out = Vec::new();
    for row in reader.get_row_iter(None)? {
        let r = row?;
        out.push(RunRow {
            repo: r.get_string(0)?.clone(),
            run_id: r.get_long(1)?,
            workflow_name: r.get_string(2)?.clone(),
            workflow_path: r.get_string(3)?.clone(),
            event: r.get_string(4)?.clone(),
            conclusion: r.get_string(5)?.clone(),
            started_at: r.get_string(6)?.clone(),
            updated_at: r.get_string(7)?.clone(),
            duration_s: r.get_long(8)?,
        });
    }
    Ok(out)
}
```

Repeat `write_/read_` pairs for `RepoRow` (bool via `r.get_bool`, size via `get_long`) and `ReleaseRow`. If the `parquet` version's derive/RowAccessor API differs from the above (it moves between majors), adapt to the installed version — the round-trip test is the contract, not this exact code.

- [ ] **Step 4: Run tests, fmt, clippy; commit**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-telemetry && cargo fmt --all && cargo clippy -p oikosbot-telemetry --all-targets -- -D warnings
git add crates/oikosbot-telemetry && git commit -m "feat(telemetry): row types and parquet snapshot round-trip"
```

### Task 8: Collector over the `gh` CLI (resumable, capped)

**Files:**
- Create: `crates/oikosbot-telemetry/src/collect.rs`
- Modify: `crates/oikosbot-telemetry/src/lib.rs` (`pub mod collect;`)

**Interfaces:**
- Produces:

```rust
pub trait GhRunner { fn api(&self, path: &str) -> anyhow::Result<serde_json::Value>; }
pub struct GhCli;                      // shells out: gh api <path>
pub fn list_repos(gh: &dyn GhRunner, owner: &str) -> anyhow::Result<Vec<RepoRow>>
pub fn collect_runs(gh: &dyn GhRunner, repo: &str, max_runs: usize) -> anyhow::Result<Vec<RunRow>>
pub fn collect_releases(gh: &dyn GhRunner, repo: &str) -> anyhow::Result<Vec<ReleaseRow>>
```

Resumability lives one level up (Task 12): the CLI writes one staging JSON per repo and skips repos whose file already exists.

- [ ] **Step 1: Write the failing tests with a fake runner**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    struct Fake(serde_json::Value);
    impl GhRunner for Fake {
        fn api(&self, _path: &str) -> anyhow::Result<serde_json::Value> { Ok(self.0.clone()) }
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
        let fake = Fake(serde_json::json!([{ "full_name": "o/r", "visibility": "public",
            "archived": false, "pushed_at": "2026-08-01T00:00:00Z", "size": 906 }]));
        let repos = list_repos(&fake, "o").unwrap();
        assert_eq!(repos[0].size_kb, 906);
    }
}
```

- [ ] **Step 2: Run to verify failure**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-telemetry collect_
```

Expected: FAIL.

- [ ] **Step 3: Implement**

```rust
use anyhow::{bail, Context, Result};
use crate::rows::{ReleaseRow, RepoRow, RunRow};

pub trait GhRunner { fn api(&self, path: &str) -> Result<serde_json::Value>; }

pub struct GhCli;
impl GhRunner for GhCli {
    fn api(&self, path: &str) -> Result<serde_json::Value> {
        let out = std::process::Command::new("gh").args(["api", path]).output()
            .context("spawn gh")?;
        if !out.status.success() {
            bail!("gh api {path}: {}", String::from_utf8_lossy(&out.stderr));
        }
        Ok(serde_json::from_slice(&out.stdout)?)
    }
}

fn iso_to_epoch(s: &str) -> i64 {
    // Rows keep ISO strings; duration needs arithmetic. RFC3339 UTC only.
    // Minimal parse: "YYYY-MM-DDTHH:MM:SSZ" — days-since-epoch via civil algorithm.
    let b = s.as_bytes();
    let num = |a: usize, z: usize| -> i64 { s[a..z].parse().unwrap_or(0) };
    if b.len() < 20 { return 0; }
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

pub fn collect_runs(gh: &dyn GhRunner, repo: &str, max_runs: usize) -> Result<Vec<RunRow>> {
    let mut rows = Vec::new();
    let mut page = 1;
    while rows.len() < max_runs {
        let v = gh.api(&format!("repos/{repo}/actions/runs?per_page=100&page={page}"))?;
        let runs = v["workflow_runs"].as_array().cloned().unwrap_or_default();
        if runs.is_empty() { break; }
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
            if rows.len() >= max_runs { break; }
        }
        page += 1;
    }
    Ok(rows)
}

pub fn list_repos(gh: &dyn GhRunner, owner: &str) -> Result<Vec<RepoRow>> {
    let mut out = Vec::new();
    for page in 1..=10 {
        let v = gh.api(&format!("users/{owner}/repos?per_page=100&page={page}&type=owner"))?;
        let arr = v.as_array().cloned().unwrap_or_default();
        if arr.is_empty() { break; }
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

pub fn collect_releases(gh: &dyn GhRunner, repo: &str) -> Result<Vec<ReleaseRow>> {
    let v = gh.api(&format!("repos/{repo}/releases?per_page=100"))?;
    Ok(v.as_array().cloned().unwrap_or_default().iter().map(|r| ReleaseRow {
        repo: repo.to_string(),
        tag: r["tag_name"].as_str().unwrap_or("").to_string(),
        published_at: r["published_at"].as_str().unwrap_or("").to_string(),
    }).collect())
}
```

Add a unit test for `iso_to_epoch` against a known pair: `iso_to_epoch("2026-08-01T00:05:00Z") - iso_to_epoch("2026-08-01T00:00:00Z") == 300`, and `iso_to_epoch("1970-01-01T00:00:00Z") == 0`.

- [ ] **Step 4: Tests, lint, commit**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-telemetry && cargo fmt --all && cargo clippy -p oikosbot-telemetry --all-targets -- -D warnings
git add crates/oikosbot-telemetry && git commit -m "feat(telemetry): gh-cli collector for runs, repos, releases"
```

### Task 9: Derived metrics with honest confidence labels

**Files:**
- Create: `crates/oikosbot-telemetry/src/derive.rs`
- Modify: `crates/oikosbot-telemetry/src/lib.rs` (`pub mod derive;`)

**Interfaces:**
- Consumes: `RunRow`, `oikosbot_metrics::Confidence`.
- Produces:

```rust
pub struct Assumptions { pub power_w: f64, pub pue: f64,
    pub grid_gco2_per_kwh: f64, pub usd_per_minute: f64 }
impl Default for Assumptions { /* documented constants, sources in comments */ }
pub struct DerivedRepo { pub repo: String, pub wall_minutes: f64,
    pub energy_kwh: f64, pub carbon_g: f64, pub imputed_cost_usd: f64 }
pub fn derive_per_repo(runs: &[RunRow], a: &Assumptions) -> Vec<DerivedRepo>
pub fn confidence_of(metric: &str) -> oikosbot_metrics::Confidence
```

- [ ] **Step 1: Write the failing tests — including the honesty assertions from the spec**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oikosbot_metrics::Confidence;
    fn run(repo: &str, dur: i64) -> crate::rows::RunRow { /* build a RunRow with duration_s = dur, conclusion "success" */ }
    #[test]
    fn derives_minutes_energy_carbon_cost() {
        let a = Assumptions::default();
        let d = derive_per_repo(&[run("o/r", 600)], &a);
        assert!((d[0].wall_minutes - 10.0).abs() < 1e-9);
        let expect_kwh = 10.0 / 60.0 * a.power_w / 1000.0 * a.pue;
        assert!((d[0].energy_kwh - expect_kwh).abs() < 1e-12);
        assert!((d[0].carbon_g - expect_kwh * a.grid_gco2_per_kwh).abs() < 1e-9);
    }
    #[test]
    fn confidence_ladder_is_honest() {
        assert_eq!(confidence_of("wall_minutes"), Confidence::Measured);
        assert_eq!(confidence_of("energy_kwh"), Confidence::Calibrated);
        assert_eq!(confidence_of("carbon_g"), Confidence::Estimated);   // NEVER better
        assert_eq!(confidence_of("imputed_cost_usd"), Confidence::Calibrated);
    }
}
```

- [ ] **Step 2: Verify failure** — `cargo test -p oikosbot-telemetry derive` → FAIL.

- [ ] **Step 3: Implement**

```rust
use crate::rows::RunRow;
use oikosbot_metrics::Confidence;
use std::collections::BTreeMap;

pub struct Assumptions {
    /// Active power attributable to a 4-vCPU share of an AMD EPYC 7763 host
    /// (Azure Standard_D4ads_v5, GitHub-hosted ubuntu-latest). 280 W TDP ×
    /// vhost ratio 0.03125 (4/128 threads, per Eco-CI) ≈ 8.75 W at full load;
    /// declared average utilisation 0.5 → 4.4 W, rounded conservatively up.
    pub power_w: f64,
    /// Azure fleet PUE per Cloud Carbon Footprint published constants.
    pub pue: f64,
    /// DECLARED grid intensity, gCO2e/kWh. GitHub does not expose runner
    /// region; this is an assumption, which is why carbon is Estimated.
    /// 475 = global average, matching the existing carbon.rs constant.
    pub grid_gco2_per_kwh: f64,
    /// Imputed price of a hosted-runner minute (USD, linux 2-core list price).
    /// VERIFY against the live GitHub pricing docs before publishing money
    /// figures — a ~40% cut took effect 2026-01-01.
    pub usd_per_minute: f64,
}
impl Default for Assumptions {
    fn default() -> Self {
        Self { power_w: 5.0, pue: 1.185, grid_gco2_per_kwh: 475.0, usd_per_minute: 0.008 }
    }
}

pub struct DerivedRepo {
    pub repo: String, pub wall_minutes: f64, pub energy_kwh: f64,
    pub carbon_g: f64, pub imputed_cost_usd: f64,
}

pub fn derive_per_repo(runs: &[RunRow], a: &Assumptions) -> Vec<DerivedRepo> {
    let mut minutes: BTreeMap<String, f64> = BTreeMap::new();
    for r in runs {
        *minutes.entry(r.repo.clone()).or_default() += r.duration_s as f64 / 60.0;
    }
    minutes.into_iter().map(|(repo, wall_minutes)| {
        let energy_kwh = wall_minutes / 60.0 * a.power_w / 1000.0 * a.pue;
        DerivedRepo {
            carbon_g: energy_kwh * a.grid_gco2_per_kwh,
            imputed_cost_usd: wall_minutes * a.usd_per_minute,
            repo, wall_minutes, energy_kwh,
        }
    }).collect()
}

/// The ladder is the honesty contract: only API-sourced quantities are
/// Measured; coefficients over measurements are Calibrated; anything
/// resting on a declared assumption (grid region) is Estimated.
pub fn confidence_of(metric: &str) -> Confidence {
    match metric {
        "wall_minutes" => Confidence::Measured,
        "energy_kwh" | "imputed_cost_usd" => Confidence::Calibrated,
        "carbon_g" => Confidence::Estimated,
        _ => Confidence::Unknown,
    }
}
```

(If `Confidence` lacks `PartialEq`, add `#[derive(PartialEq, Eq)]` to it in `oikosbot-metrics` — check first; the pareto crate already compares confidences via `confidence_rank`, so it likely has what's needed.)

- [ ] **Step 4: Tests, lint, commit**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-telemetry && cargo fmt --all && cargo clippy -p oikosbot-telemetry --all-targets -- -D warnings
git add crates/oikosbot-telemetry && git commit -m "feat(telemetry): derived energy/carbon/cost with honest confidence ladder"
```

### Task 10: Capability metrics (the output side)

**Files:**
- Create: `crates/oikosbot-capability/src/lib.rs` (replace scaffold)

**Interfaces:**
- Consumes: `oikosbot_telemetry::rows::{RunRow, ReleaseRow}`.
- Produces:

```rust
pub struct CapabilityRow {
    pub repo: String,
    pub runs_total: u64,
    pub startup_failures: u64,
    pub parse_dead_runs: u64,          // run name == workflow file path
    pub workflows_seen: u64,
    pub workflows_fallible: u64,       // ≥1 historical failure
    pub infallible_gate_candidates: Vec<String>, // ≥min_n successes, 0 failures
    pub verified_success_runs: u64,    // successes on FALLIBLE workflows only
    pub releases: u64,
}
pub fn assess(runs: &[RunRow], releases: &[ReleaseRow], min_n: u64) -> Vec<CapabilityRow>
```

- [ ] **Step 1: Write the failing tests — ground truth from the estate's documented pathologies**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use oikosbot_telemetry::rows::RunRow;
    fn run(repo: &str, wf: &str, path: &str, concl: &str) -> RunRow {
        RunRow { repo: repo.into(), run_id: 0, workflow_name: wf.into(),
            workflow_path: path.into(), event: "push".into(), conclusion: concl.into(),
            started_at: String::new(), updated_at: String::new(), duration_s: 60 }
    }
    #[test]
    fn startup_failures_and_parse_dead_are_counted() {
        // echidna pattern: repo-total startup_failure
        let runs: Vec<_> = (0..3).map(|_| run("o/dead", "CI", "x", "startup_failure")).collect();
        // parse-rejected pattern: run named by its own file path
        let mut all = runs;
        all.push(run("o/dead", ".github/workflows/m.yml", ".github/workflows/m.yml", "failure"));
        let c = assess(&all, &[], 5);
        assert_eq!(c[0].startup_failures, 3);
        assert_eq!(c[0].parse_dead_runs, 1);
    }
    #[test]
    fn gate_that_never_failed_is_flagged_and_excluded_from_verified_output() {
        // fake-gate pattern: N successes, zero failures → candidate;
        // its successes must NOT count as verified output.
        let mut all: Vec<_> = (0..6).map(|_| run("o/fake", "Gate", "g.yml", "success")).collect();
        all.push(run("o/fake", "Real", "r.yml", "failure"));
        all.push(run("o/fake", "Real", "r.yml", "success"));
        let c = assess(&all, &[], 5);
        assert_eq!(c[0].infallible_gate_candidates, vec!["Gate".to_string()]);
        assert_eq!(c[0].workflows_fallible, 1);
        assert_eq!(c[0].verified_success_runs, 1); // only Real's success counts
    }
}
```

- [ ] **Step 2: Verify failure** — `cargo test -p oikosbot-capability` → FAIL.

- [ ] **Step 3: Implement**

```rust
// SPDX-License-Identifier: MPL-2.0
//! Verified capability: output metrics derived from run history alone.
//! A gate with N successes and zero failures in its whole history cannot
//! be shown to be able to fail; its "green" is not evidence, so its
//! successes are excluded from verified output. This is the telemetry
//! operationalisation of the estate's fake-gate pathology.
use oikosbot_telemetry::rows::{ReleaseRow, RunRow};
use std::collections::BTreeMap;

pub struct CapabilityRow { /* fields exactly as the interface block above */ }

pub fn assess(runs: &[RunRow], releases: &[ReleaseRow], min_n: u64) -> Vec<CapabilityRow> {
    let mut by_repo: BTreeMap<&str, Vec<&RunRow>> = BTreeMap::new();
    for r in runs { by_repo.entry(&r.repo).or_default().push(r); }
    let mut rel_count: BTreeMap<&str, u64> = BTreeMap::new();
    for r in releases { *rel_count.entry(&r.repo).or_default() += 1; }

    by_repo.into_iter().map(|(repo, rs)| {
        let mut wf: BTreeMap<&str, (u64, u64)> = BTreeMap::new(); // (successes, failures)
        let mut startup = 0u64; let mut parse_dead = 0u64;
        for r in &rs {
            if r.conclusion == "startup_failure" { startup += 1; }
            if r.workflow_name == r.workflow_path && !r.workflow_path.is_empty() { parse_dead += 1; }
            let e = wf.entry(&r.workflow_name).or_default();
            match r.conclusion.as_str() {
                "success" => e.0 += 1,
                "failure" | "startup_failure" | "timed_out" => e.1 += 1,
                _ => {}
            }
        }
        let infallible: Vec<String> = wf.iter()
            .filter(|(_, (s, f))| *f == 0 && *s >= min_n)
            .map(|(n, _)| n.to_string()).collect();
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
    }).collect()
}
```

- [ ] **Step 4: Tests, lint, commit**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-capability && cargo fmt --all && cargo clippy -p oikosbot-capability --all-targets -- -D warnings
git add crates/oikosbot-capability && git commit -m "feat(capability): verified-capability metrics from run history"
```

### Task 11: DEA — CCR/BCC with paired envelopment/multiplier LPs

**Files:**
- Create: `crates/oikosbot-dea/src/lib.rs` (replace scaffold)

**Interfaces:**
- Produces:

```rust
pub struct Dmu { pub name: String, pub inputs: Vec<f64>, pub outputs: Vec<f64> }
pub struct DeaScore {
    pub name: String,
    pub theta_ccr: f64,          // envelopment objective
    pub theta_mult: f64,         // multiplier objective (strong-duality check)
    pub theta_bcc: f64,
    pub peers: Vec<(String, f64)>,       // λ_j > 1e-6
    pub input_weights: Vec<f64>,         // v_i from multiplier form
    pub output_weights: Vec<f64>,        // u_r from multiplier form
}
pub fn dea(dmus: &[Dmu]) -> anyhow::Result<Vec<DeaScore>>
```

Inputs/outputs are floored at `1e-6` inside `dea()` (zero inputs/outputs are common in the estate — dead repos — and break LP feasibility).

- [ ] **Step 1: Write the failing analytic tests**

Single-input single-output CRS has a closed form: θ_j = (y_j/x_j) / max_k(y_k/x_k).

```rust
#[cfg(test)]
mod tests {
    use super::*;
    fn dmu(n: &str, x: f64, y: f64) -> Dmu {
        Dmu { name: n.into(), inputs: vec![x], outputs: vec![y] }
    }
    #[test]
    fn single_io_matches_closed_form() {
        // A(2,2)=1.0, B(4,4)=1.0, C(8,4)=0.5
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("B", 4.0, 4.0), dmu("C", 8.0, 4.0)]).unwrap();
        let get = |n: &str| r.iter().find(|s| s.name == n).unwrap();
        assert!((get("A").theta_ccr - 1.0).abs() < 1e-6);
        assert!((get("B").theta_ccr - 1.0).abs() < 1e-6);
        assert!((get("C").theta_ccr - 0.5).abs() < 1e-6);
    }
    #[test]
    fn strong_duality_holds_and_scores_bounded() {
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("B", 4.0, 4.0), dmu("C", 8.0, 4.0)]).unwrap();
        for s in &r {
            assert!((s.theta_ccr - s.theta_mult).abs() < 1e-5, "duality gap for {}", s.name);
            assert!(s.theta_ccr > 0.0 && s.theta_ccr <= 1.0 + 1e-9);
            assert!(s.theta_bcc + 1e-9 >= s.theta_ccr); // BCC frontier is closer
        }
    }
    #[test]
    fn inefficient_unit_names_frontier_peers() {
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("C", 8.0, 4.0)]).unwrap();
        let c = r.iter().find(|s| s.name == "C").unwrap();
        assert!(c.peers.iter().any(|(n, w)| n == "A" && *w > 0.0));
    }
    #[test]
    fn two_input_dominated_unit_is_inefficient() {
        // D uses strictly more of both inputs than E for the same output.
        let e = Dmu { name: "E".into(), inputs: vec![1.0, 1.0], outputs: vec![1.0] };
        let d = Dmu { name: "D".into(), inputs: vec![2.0, 2.0], outputs: vec![1.0] };
        let r = dea(&[e, d]).unwrap();
        let d = r.iter().find(|s| s.name == "D").unwrap();
        assert!((d.theta_ccr - 0.5).abs() < 1e-6);
    }
}
```

- [ ] **Step 2: Verify failure** — `cargo test -p oikosbot-dea` → FAIL.

- [ ] **Step 3: Implement**

```rust
// SPDX-License-Identifier: MPL-2.0
//! Data Envelopment Analysis: input-oriented CCR and BCC, solved as plain
//! LPs. Envelopment form yields θ and the peer reference set; the multiplier
//! (dual) form yields the virtual input/output weights — the shadow prices.
//! Solving BOTH and asserting the objectives agree is a built-in
//! strong-duality verification (spec §Verification 3).
use anyhow::Result;
use good_lp::{constraint, variable, Expression, ProblemVariables, Solution, SolverModel};

pub struct Dmu { pub name: String, pub inputs: Vec<f64>, pub outputs: Vec<f64> }
pub struct DeaScore { /* fields exactly as the interface block */ }

const FLOOR: f64 = 1e-6;
const PEER_TOL: f64 = 1e-6;

fn floored(d: &Dmu) -> (Vec<f64>, Vec<f64>) {
    (d.inputs.iter().map(|v| v.max(FLOOR)).collect(),
     d.outputs.iter().map(|v| v.max(FLOOR)).collect())
}

/// Envelopment form. vrs=false → CCR; vrs=true adds Σλ=1 → BCC.
fn envelopment(o: usize, xs: &[Vec<f64>], ys: &[Vec<f64>], vrs: bool) -> Result<(f64, Vec<f64>)> {
    let n = xs.len();
    let mut pb = ProblemVariables::new();
    let theta = pb.add(variable().min(0.0));
    let lam: Vec<_> = (0..n).map(|_| pb.add(variable().min(0.0))).collect();
    let mut model = pb.minimise(theta).using(good_lp::solvers::highs::highs);
    for i in 0..xs[o].len() {
        let lhs: Expression = lam.iter().enumerate().map(|(j, l)| *l * xs[j][i]).sum();
        model = model.with(constraint!(lhs <= theta * xs[o][i]));
    }
    for r in 0..ys[o].len() {
        let lhs: Expression = lam.iter().enumerate().map(|(j, l)| *l * ys[j][r]).sum();
        model = model.with(constraint!(lhs >= ys[o][r]));
    }
    if vrs {
        let s: Expression = lam.iter().map(|l| Expression::from(*l)).sum();
        model = model.with(constraint!(s == 1.0));
    }
    let sol = model.solve()?;
    Ok((sol.value(theta), lam.iter().map(|l| sol.value(*l)).collect()))
}

/// Multiplier form: max u·y_o  s.t. v·x_o = 1,  u·y_j − v·x_j ≤ 0 ∀j,  u,v ≥ 0.
fn multiplier(o: usize, xs: &[Vec<f64>], ys: &[Vec<f64>]) -> Result<(f64, Vec<f64>, Vec<f64>)> {
    let (m, s) = (xs[o].len(), ys[o].len());
    let mut pb = ProblemVariables::new();
    let v: Vec<_> = (0..m).map(|_| pb.add(variable().min(0.0))).collect();
    let u: Vec<_> = (0..s).map(|_| pb.add(variable().min(0.0))).collect();
    let obj: Expression = u.iter().enumerate().map(|(r, uu)| *uu * ys[o][r]).sum();
    let mut model = pb.maximise(obj.clone()).using(good_lp::solvers::highs::highs);
    let norm: Expression = v.iter().enumerate().map(|(i, vv)| *vv * xs[o][i]).sum();
    model = model.with(constraint!(norm == 1.0));
    for j in 0..xs.len() {
        let uy: Expression = u.iter().enumerate().map(|(r, uu)| *uu * ys[j][r]).sum();
        let vx: Expression = v.iter().enumerate().map(|(i, vv)| *vv * xs[j][i]).sum();
        model = model.with(constraint!(uy - vx <= 0.0));
    }
    let sol = model.solve()?;
    Ok((sol.eval(&obj),
        v.iter().map(|x| sol.value(*x)).collect(),
        u.iter().map(|x| sol.value(*x)).collect()))
}

pub fn dea(dmus: &[Dmu]) -> Result<Vec<DeaScore>> {
    let xs: Vec<Vec<f64>> = dmus.iter().map(|d| floored(d).0).collect();
    let ys: Vec<Vec<f64>> = dmus.iter().map(|d| floored(d).1).collect();
    (0..dmus.len()).map(|o| {
        let (theta_ccr, lam) = envelopment(o, &xs, &ys, false)?;
        let (theta_bcc, _) = envelopment(o, &xs, &ys, true)?;
        let (theta_mult, vw, uw) = multiplier(o, &xs, &ys)?;
        let peers = lam.iter().enumerate()
            .filter(|(j, l)| **l > PEER_TOL && *j != o)
            .map(|(j, l)| (dmus[j].name.clone(), *l)).collect();
        Ok(DeaScore { name: dmus[o].name.clone(), theta_ccr, theta_mult, theta_bcc,
                      peers, input_weights: vw, output_weights: uw })
    }).collect()
}
```

(If the installed `good_lp` exposes the HiGHS solver under a different path than `good_lp::solvers::highs::highs`, use what `cargo doc` shows — the solver import is the only unstable point; the LP formulations are the contract.)

- [ ] **Step 4: Tests, lint, commit**

```bash
cd "$OIKOSBOT" && cargo test -p oikosbot-dea && cargo fmt --all && cargo clippy -p oikosbot-dea --all-targets -- -D warnings
git add crates/oikosbot-dea && git commit -m "feat(dea): CCR/BCC via paired envelopment+multiplier LPs on HiGHS"
```

### Task 12: `estate` CLI subcommand (collect / analyse / report)

**Files:**
- Create: `crates/oikosbot-cli/src/estate.rs`
- Modify: `crates/oikosbot-cli/src/main.rs` (add `Estate` variant to the clap enum; `mod estate;`)
- Modify: `crates/oikosbot-cli/Cargo.toml` (add path deps on the three new crates)

**Interfaces:**
- Consumes: everything produced in Tasks 7–11.
- Produces CLI:

```
oikosbot estate collect --owner hyperpolymath --owner metadatastician \
    --out <staging-dir> [--max-runs 200]
oikosbot estate analyse --staging <staging-dir> --snapshot <snapshot-dir>
oikosbot estate report  --snapshot <snapshot-dir> [--format md|json] [-o file]
```

`collect` writes one `<staging>/runs-<owner>-<repo>.json` (serde_json of `Vec<RunRow>`) plus `repos-<owner>.json` and `releases-<owner>-<repo>.json` per repo, **skipping any file that already exists** (resumability). `analyse` reads staging, writes `runs.parquet`/`repos.parquet`/`releases.parquet` + `analysis.json` (derived + capability + DEA + independence) into the snapshot dir. `report` renders `analysis.json`.

- [ ] **Step 1: Write the failing test for the analyse core (pure function)**

In `estate.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn analysis_produces_dea_and_independence_for_two_repos() {
        use oikosbot_telemetry::rows::*;
        let mk = |repo: &str, dur, concl: &str| RunRow { repo: repo.into(), run_id: 0,
            workflow_name: "CI".into(), workflow_path: "ci.yml".into(), event: "push".into(),
            conclusion: concl.into(), started_at: String::new(), updated_at: String::new(),
            duration_s: dur };
        let runs = vec![mk("o/a", 600, "success"), mk("o/a", 600, "failure"),
                        mk("o/b", 6000, "success"), mk("o/b", 600, "failure")];
        let repos = vec![
            RepoRow { repo: "o/a".into(), visibility: "public".into(), archived: false,
                      pushed_at: String::new(), size_kb: 100 },
            RepoRow { repo: "o/b".into(), visibility: "public".into(), archived: false,
                      pushed_at: String::new(), size_kb: 5000 }];
        let a = run_analysis(&runs, &repos, &[], &Default::default()).unwrap();
        assert_eq!(a.dea.len(), 2);
        assert!(a.dea.iter().all(|s| s.theta_ccr > 0.0 && s.theta_ccr <= 1.0 + 1e-9));
        assert!(a.independence.iter().any(|(pair, _)| pair == "wall_minutes~size_kb"));
    }
}
```

- [ ] **Step 2: Verify failure** — `cargo test -p oikosbot-cli analysis_produces` → FAIL.

- [ ] **Step 3: Implement `estate.rs`**

Core pieces (clap wiring mirrors the existing subcommand style in `main.rs`):

```rust
// SPDX-License-Identifier: MPL-2.0
use anyhow::Result;
use oikosbot_capability::{assess, CapabilityRow};
use oikosbot_dea::{dea, DeaScore, Dmu};
use oikosbot_telemetry::derive::{derive_per_repo, Assumptions, confidence_of};
use oikosbot_telemetry::rows::{ReleaseRow, RepoRow, RunRow};

#[derive(serde::Serialize)]
pub struct Analysis {
    pub derived: Vec<DerivedOut>,
    pub capability: Vec<CapabilityOut>,
    pub dea: Vec<DeaOut>,
    /// Pearson correlations over per-repo axes. Derived axes (energy, carbon,
    /// cost) are linear in wall_minutes BY CONSTRUCTION in round one and are
    /// therefore not reported as independence evidence; the falsifier pair is
    /// wall_minutes ~ size_kb (measured compute vs code-volume proxy).
    pub independence: Vec<(String, f64)>,
    pub confidence_counts: std::collections::BTreeMap<String, String>,
}
// DerivedOut / CapabilityOut / DeaOut are flat serde mirrors of the crate
// types (define them here; DeaScore etc. do not derive Serialize).

pub fn pearson(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len() as f64;
    if n < 2.0 { return f64::NAN; }
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let cov: f64 = a.iter().zip(b).map(|(x, y)| (x - ma) * (y - mb)).sum();
    let (va, vb): (f64, f64) = (a.iter().map(|x| (x - ma).powi(2)).sum(),
                                b.iter().map(|y| (y - mb).powi(2)).sum());
    cov / (va.sqrt() * vb.sqrt())
}

pub fn run_analysis(runs: &[RunRow], repos: &[RepoRow], releases: &[ReleaseRow],
                    assumptions: &Assumptions) -> Result<Analysis> {
    let derived = derive_per_repo(runs, assumptions);
    let capability = assess(runs, releases, 5);
    // DEA: inputs [wall_minutes, size_kb]; outputs [verified_success_runs, releases].
    let size_of = |repo: &str| repos.iter().find(|r| r.repo == repo)
        .map(|r| r.size_kb as f64).unwrap_or(0.0);
    let cap_of = |repo: &str| capability.iter().find(|c| c.repo == repo);
    let dmus: Vec<Dmu> = derived.iter().map(|d| Dmu {
        name: d.repo.clone(),
        inputs: vec![d.wall_minutes, size_of(&d.repo)],
        outputs: vec![
            cap_of(&d.repo).map(|c| c.verified_success_runs as f64).unwrap_or(0.0),
            cap_of(&d.repo).map(|c| c.releases as f64).unwrap_or(0.0),
        ],
    }).collect();
    let dea_scores = dea(&dmus)?;
    let mins: Vec<f64> = derived.iter().map(|d| d.wall_minutes).collect();
    let sizes: Vec<f64> = derived.iter().map(|d| size_of(&d.repo)).collect();
    let outs: Vec<f64> = derived.iter()
        .map(|d| cap_of(&d.repo).map(|c| c.verified_success_runs as f64).unwrap_or(0.0)).collect();
    let independence = vec![
        ("wall_minutes~size_kb".to_string(), pearson(&mins, &sizes)),
        ("wall_minutes~verified_success_runs".to_string(), pearson(&mins, &outs)),
    ];
    let confidence_counts = ["wall_minutes", "energy_kwh", "carbon_g", "imputed_cost_usd"]
        .iter().map(|m| (m.to_string(), format!("{:?}", confidence_of(m)))).collect();
    Ok(Analysis { /* map crate types into the *Out mirrors */ })
}
```

`collect` (in the same file) loops `list_repos` per owner, then per non-archived repo calls `collect_runs`/`collect_releases`, writing each result to its staging JSON and printing progress; a pre-existing staging file is skipped with a log line (this is the resumability contract). `analyse` loads staging JSONs, writes the three Parquet files via `snapshot::write_*`, runs `run_analysis`, writes `analysis.json`. `report` renders markdown: frontier table (θ=1 repos), worst-20 off-frontier with their peers, X-inefficiency list — the design's headline — as repos with `wall_minutes > 0 && verified_success_runs == 0` under a heading that states plainly: *"X-inefficiency here is our own framing (Leibenstein has no software-engineering literature): real input consumed, zero verified output produced."* End with the independence pairs and the confidence counts, Infracost-style.

- [ ] **Step 4: Wire clap** — add to the existing `Commands` enum in `main.rs`:

```rust
/// Estate-level telemetry, capability and DEA analysis (read-only)
Estate {
    #[command(subcommand)]
    cmd: estate::EstateCmd,
},
```

with `EstateCmd { Collect{..}, Analyse{..}, Report{..} }` defined in `estate.rs` carrying the flags from the interface block.

- [ ] **Step 5: Tests, lint, commit**

```bash
cd "$OIKOSBOT" && cargo test --workspace && cargo fmt --all && cargo clippy --workspace --all-targets -- -D warnings
git add crates/oikosbot-cli && git commit -m "feat(cli): estate collect/analyse/report subcommands"
```

### Task 13: Create the `oikosbot-estate` dataset repo and take snapshot #1

**Files:**
- Create (new repo, outside the workspace): `~/developer/oikosbot-estate/README.adoc`, `SCHEMA.adoc`, `snapshots/<date>/…`

**Interfaces:**
- Consumes: the `oikosbot estate` CLI from Task 12.
- Produces: `hyperpolymath/oikosbot-estate` on GitHub with the first committed snapshot.

- [ ] **Step 1: Initialise the repo**

```bash
mkdir -p ~/developer/oikosbot-estate/snapshots && cd ~/developer/oikosbot-estate
git init -b main
cat > README.adoc <<'EOF'
// SPDX-License-Identifier: MPL-2.0
= oikosbot-estate
Versioned telemetry snapshots of the estate, produced by `oikosbot estate`.
Data repo only: no code, no CI. One directory per collection run under
`snapshots/`, containing runs/repos/releases Parquet plus analysis.json.
History is git history; dynamic-efficiency analysis reads consecutive
snapshots. Named `oikosbot-estate` (not `oikos-*`, which is the accounting
DSL's prefix — see oikosbot's DISAMBIGUATION.adoc).
EOF
```

`SCHEMA.adoc`: one table per Parquet file listing column name/type/meaning, copied from the `rows.rs` doc comments, plus the ledger note: flows (minutes, energy, carbon, cost per snapshot period) and stocks (cumulative totals) reconcile per the SFC discipline — the reconciliation check itself lands with multi-snapshot support in round two, and this file says so explicitly.

- [ ] **Step 2: Run the first collection (small scope first, then full)**

```bash
cd "$OIKOSBOT"
cargo run -p oikosbot-cli -- estate collect --owner hyperpolymath --out ~/developer/oikosbot-estate/.staging --max-runs 200
cargo run -p oikosbot-cli -- estate collect --owner metadatastician --out ~/developer/oikosbot-estate/.staging --max-runs 200
DATE=$(date -u +%F)
cargo run -p oikosbot-cli -- estate analyse --staging ~/developer/oikosbot-estate/.staging \
  --snapshot ~/developer/oikosbot-estate/snapshots/$DATE
cargo run -p oikosbot-cli -- estate report --snapshot ~/developer/oikosbot-estate/snapshots/$DATE \
  --format md -o ~/developer/oikosbot-estate/snapshots/$DATE/report.md
```

(Interrupt/re-run `collect` at least once mid-way to exercise resumability; the second run must skip already-collected repos.)

- [ ] **Step 3: Verify against ground truth before committing**

The report MUST show, or the pipeline is wrong (spec §Verification 4):
- `hyperpolymath/oikosbot` with heavy `startup_failure` counts since 07-30;
- known-dead repos (echidna, kitchenspeak) dominated by startup failures;
- at least one infallible-gate candidate somewhere in the estate (documented pathology).

Also confirm `.staging/` is NOT committed (add to `.gitignore`).

- [ ] **Step 4: Create the GitHub repo and push**

```bash
cd ~/developer/oikosbot-estate
echo ".staging/" > .gitignore
git add -A && git commit -m "snapshot: first estate telemetry collection ($DATE)"
gh repo create hyperpolymath/oikosbot-estate --public --source . --push \
  --description "Estate telemetry snapshots for OikosBot's economics pipeline (data only)"
```

### Task 14: Reproducibility + end-to-end verification, PR

**Files:**
- Create: `docs/superpowers/notes/2026-08-03-round-one-verification.md` (results record)

**Interfaces:**
- Consumes: snapshot #1; the full workspace.
- Produces: the round-one PR with the verification record; the spec's six verification items each answered with evidence.

- [ ] **Step 1: Determinism of analyse**

```bash
cargo run -p oikosbot-cli -- estate analyse --staging ~/developer/oikosbot-estate/.staging --snapshot /tmp/snap-b
diff <(jq -S . ~/developer/oikosbot-estate/snapshots/$DATE/analysis.json) <(jq -S . /tmp/snap-b/analysis.json)
```

Expected: empty diff. (Collection reproducibility over a shifting live window cannot be byte-exact; record instead that two collects within minutes agree on all closed runs — check a sample repo's run set.)

- [ ] **Step 2: Record the six verification answers**

Write `2026-08-03-round-one-verification.md` answering each spec item with the actual evidence: (1) the independence coefficients from `analysis.json` — state plainly whether `wall_minutes~size_kb` is low (premise holds) or high (premise fails; say so, do not soften); (2) the determinism diff; (3) the DEA analytic tests + duality-gap assertion; (4) the ground-truth detections from Task 13 step 3; (5) the confidence-ladder test; (6) Part 0 status (PRs open, CI-unverifiable, why).

- [ ] **Step 3: Full workspace gate and PR**

```bash
cd "$OIKOSBOT" && cargo test --workspace && cargo fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings
git add docs/superpowers/notes && git commit -m "docs: round-one verification record"
git push -u origin feat/estate-round-one
gh pr create -R hyperpolymath/oikosbot --head feat/estate-round-one \
  --title "feat: estate economics round one — telemetry, capability, DEA (read-only)" \
  --body "Implements the approved design (docs/superpowers/specs/2026-08-03-estate-economics-design.md): oikosbot-telemetry / oikosbot-capability / oikosbot-dea crates + 'oikosbot estate' CLI. Read-only; nothing blocks. First snapshot at hyperpolymath/oikosbot-estate. Verification record in docs/superpowers/notes/. CI dead estate-wide (lockfile enforcement); verified locally: cargo test --workspace green."
```

---

## Self-Review (done at write time)

- **Spec coverage:** Part 0 items 1–6 → Tasks 2,3,4,5(×2),1; collector/capability/DEA/CLI/dataset-repo/verification → Tasks 7–14. Deferred per spec: SFC reconciliation check (needs ≥2 snapshots — noted in SCHEMA.adoc), static capability signals, PR decoration, Eclexia join, enforcement.
- **Known API risk, stated:** exact `parquet_derive`/`good_lp` item paths move between versions; the tests are the contract and Steps say adapt to the installed version.
- **Type consistency:** `RunRow`/`RepoRow`/`ReleaseRow` field lists identical in Tasks 7, 8, 10, 12; `Confidence` variants match `oikosbot-metrics`; DEA field names consistent between Task 11 and 12.
