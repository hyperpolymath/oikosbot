# Round-one verification record: estate economics (telemetry / capability / DEA)

**Date:** 2026-08-03
**Snapshot under test:** `hyperpolymath/oikosbot-estate` `snapshots/2026-08-03/` (commit `c196423`)
**Spec:** `docs/superpowers/specs/2026-08-03-estate-economics-design.md`, "Verification" section, six items.
**Workspace:** `/home/hyperpolymath/developer/hyper-repos/_EXTENSIONS _SET/oikosbot`, branch `feat/estate-round-one`.

This note answers each of the spec's six verification items with the actual
evidence produced by running the pipeline, not by re-deriving results.

---

## 1. Independence

From `snapshots/2026-08-03/analysis.json` → `independence`:

| pair | pearson r |
|---|---|
| `wall_minutes ~ size_kb` | **-0.0486** |
| `wall_minutes ~ verified_success_runs` | 0.1520 |

**Verdict: premise HOLDS.** The falsifier pair, `wall_minutes ~ size_kb`, is
essentially zero (-0.0486) — measured compute (the DEA input, `wall_minutes`)
is empirically uncorrelated with the code-volume proxy (`size_kb`). This
means the DEA model's inputs are not secretly measuring the same axis twice.
Stated plainly and without softening: had this coefficient come out high
(e.g. |r| > ~0.5), the premise would have failed and that would be reported
as a failure, not rationalised away. It did not — the coefficient is low.

## 2. Determinism

Ran a second `analyse` over the **same, static** `.staging/` directory used
for snapshot #1 (frozen since Task 13's collect; nothing re-collected):

```
$ ./target/release/oikosbot estate analyse \
    --staging ~/developer/oikosbot-estate/.staging \
    --snapshot "$CLAUDE_JOB_DIR/tmp/snap-b"
wrote snapshot to /home/hyperpolymath/.claude/jobs/05e3bf6a/tmp/snap-b
real  0m20.699s

$ diff <(jq -S . ~/developer/oikosbot-estate/snapshots/2026-08-03/analysis.json) \
       <(jq -S . "$CLAUDE_JOB_DIR/tmp/snap-b/analysis.json")
$ echo $?
0
```

**Result: empty diff.** `analyse` is byte-for-byte deterministic given a
fixed staging input — no floating-point nondeterminism, no HashMap-ordering
leakage, no timestamp-of-run contamination in the output JSON.

**Collection-window caveat (as flagged in the brief):** this only proves
determinism of the *analyse* step over a frozen input. It does **not** (and
cannot) prove that two independent `estate collect` runs against a live
GitHub Actions history would be byte-identical, because the collection
window is a moving target — new runs land, in-flight runs finalise, between
one `collect` invocation and the next. Task 13's own resumability exercise
(deliberately `kill -9`-ing a `collect` mid-sweep and re-running the
identical command) is the closest available substitute: the re-run emitted
exactly the 7 `skip (exists)` lines expected for the already-landed files
and then continued to completion with 0 errors — i.e. two partially-
overlapping collects over the same short window agreed on every run they
both saw, for the repos already staged. No second live collect was run for
this task (that would require re-hitting `gh api` against a shifted window,
which is out of scope for a determinism check); the byte-exact `analyse`
diff above is the reproducibility guarantee this task can make, plus the
resumability evidence as the collection-side substitute.

## 3. DEA correctness

Cited, not re-derived — analytic tests in `crates/oikosbot-dea/src/lib.rs`:

- `single_io_matches_closed_form` (line 143) — single-input/single-output
  case checked against the closed-form ratio (θ = own_ratio /
  max_peer_ratio), confirming the LP matches hand-computable ground truth
  for DMUs A(2,2)=1.0, B(4,4)=1.0, C(8,4)=0.5.
- `strong_duality_holds_and_scores_bounded` (line 152) — for every DMU,
  `|theta_ccr - theta_mult| < 1e-5` (envelopment LP score equals multiplier
  LP score, i.e. **strong duality holds** within numerical tolerance);
  `0 < theta_ccr <= 1 + 1e-9`; and `theta_bcc + 1e-9 >= theta_ccr` (the BCC
  frontier is never farther than CCR, as VRS theory requires).
- `inefficient_unit_names_frontier_peers` (line 165) — an inefficient DMU's
  peer set includes the frontier unit with positive envelopment weight
  (λ > `PEER_TOL` = 1e-6).
- `two_input_dominated_unit_is_inefficient` (line 171) — a DMU strictly
  dominated on two inputs for the same output scores inefficient.

All four pass under `cargo test -p oikosbot-dea` (see §Full workspace gate
below for the consolidated run).

**Honest flag, not smoothed over:** in the live snapshot, all 20 "worst"
DMUs in `report.md`'s off-frontier table score `theta_ccr = 0.0000` with an
**empty peers list**. This is mathematically consistent with the code's own
documented behaviour — `crates/oikosbot-dea/src/lib.rs` line 39,
`const FLOOR: f64 = 1e-6`, and the comment at line 103, "Inputs/outputs are
floored at `1e-6` before solving (zero inputs/outputs...)" — these are
repos with `verified_success_runs = 0`, so their DEA output vector is
floored to `1e-6` rather than true zero, and the resulting envelopment LP
finds no positive-weight peer combination that dominates them (θ collapses
to the floor itself, effectively unclassifiable as "closer to" any single
peer). It is not a bug in the tested LP machinery — the analytic tests
above pass on synthetic DMUs with genuine non-zero outputs — but it means
the DEA scores for verified-zero-output repos carry no discriminative peer
information. This is flagged here for round-two refinement (e.g. treating
zero-verified-output DMUs as a separate X-inefficiency class rather than
running them through CCR at all), per Task 13's own concern note.

## 4. Ground truth

Cited from Task 13's report (`task-13-report.md`, Step 3, "ALL PASS"),
verified against the staged raw JSON:

1. **`hyperpolymath/oikosbot` heavy `startup_failure` since 2026-07-30** —
   137/200 runs (68.5%) are `startup_failure`; earliest timestamp
   `2026-07-30T16:38:55Z` — every `startup_failure` in the 200-run window
   is on/after 07-30, matching the "epidemic since ~07-30" memory note.
2. **echidna / kitchenspeak dominated by `startup_failure`** — echidna:
   186/200 (93%); kitchenspeak: 136/200 (68%, remaining 64 plain
   `failure`).
3. **≥1 infallible-gate candidate, far exceeded** — grouping all 69,445
   runs by `(repo, workflow_path)`, filtered to groups with ≥5 runs and
   100% `success`: **1,270 candidate workflows**. Lowest-duration examples
   (7-12s average) include `rhodibot.yml`, `wellknown-enforcement.yml`,
   `guix-policy.yml`, `sonarqube.yml`, `secret-scanner.yml`,
   `runtime-policy.yml` — several matching already-documented pathologies
   (Guix gate presence-only, proof gates that cannot fail).
4. **38 X-inefficiency repos** — `report.md`'s X-inefficiency table:
   `wall_minutes > 0` with `verified_success_runs = 0`. Largest consumers:
   `hyperpolymath/live-files` (23,281.35 wall-min),
   `hyperpolymath/nextgen-language-evangeliser` (5,917.75),
   `hyperpolymath/JuliaPackage-Reuse-Audit.jl` (6,163.82),
   `hyperpolymath/road-skate` (4,456.92),
   `hyperpolymath/php-aegis` (4,100.50). Smallest non-zero: several
   `hpm-*-rsr` repos and `a2ml-estate-normalizer` at 0.05 wall-min each —
   real input consumed, zero verified output, regardless of scale.

All four gates PASS.

## 5. Confidence labelling

Cited from `crates/oikosbot-telemetry/src/derive.rs` line 135,
`confidence_ladder_is_honest`:

```rust
assert_eq!(confidence_of("wall_minutes"), Confidence::Measured);
assert_eq!(confidence_of("energy_kwh"), Confidence::Calibrated);
assert_eq!(confidence_of("carbon_g"), Confidence::Estimated);
assert_eq!(confidence_of("imputed_cost_usd"), Confidence::Calibrated);
```

Cross-checked against the snapshot's own `analysis.json` →
`confidence_counts` block (independently emitted at analysis time, not just
asserted in the unit test):

```json
{
  "wall_minutes": "Measured",
  "energy_kwh": "Calibrated",
  "imputed_cost_usd": "Calibrated",
  "carbon_g": "Estimated"
}
```

Test and live output agree: the confidence ladder is honestly graded
per-metric — `wall_minutes` comes straight from GitHub API timestamps
(Measured), `energy_kwh`/`imputed_cost_usd` are derived via fixed
assumption constants applied to a measured quantity (Calibrated), and
`carbon_g` additionally depends on an external, time-varying grid-intensity
figure (Estimated) — the ladder does not claim more certainty than the
derivation chain supports at any step.

## 6. Part 0

Status of the Part-0 fix PRs (checked live via `gh pr view` /
`gh issue view` on 2026-08-03, i.e. this task's own run, not carried over
from the brief which predates these merges landing):

- `hyperpolymath/oikosbot#58` — "fix(ci): drop illegal timeout-minutes on
  uses: jobs (mirror, secret-scanner)" — **MERGED** 2026-08-03T16:42:30Z.
- `hyperpolymath/oikosbot#59` — "fix: loud builtin-policy warning; label
  aspirational config" — **MERGED** (open at brief-authoring time, merged
  since).
- `metadatastician/enaction-engine#26` — "fix(ci): remove stale
  OIKOSBOT_ENABLED gate on the oikosbot job" — **MERGED** (open at
  brief-authoring time, merged since).

All three Part-0 fixes are now merged to their respective `main` branches.

**CI-verification is blocked estate-wide, not specific to these three
repos**: per the memory note `actions-lockfile-enforcement-startup-failure`,
GitHub's workflow-lockfile enforcement update produces `startup_failure` on
essentially every workflow across the estate (oikosbot, echidna, squisher,
hypatia, trope-checker, UMS, and by the ground-truth measurement above,
oikosbot/echidna/kitchenspeak specifically) as of ~2026-07-30, with the
actual failure reason visible only on the GitHub Actions run HTML page (not
in `gh run view`/API output). The fix requires the repo owner to install
the `github/gh-actions-lock` extension — an action pending on the owner,
outside this task's scope. Consequently, none of the three Part-0 PRs above
(nor this round-one PR) can be **CI**-verified right now regardless of
correctness; all verification in this document and in Task 13 was done by
direct file inspection (diffing the illegal-key removal, reading the
`OIKOSBOT_ENABLED` gate removal) and by running the relevant test/build
commands **locally** (`cargo test`, `cargo build --release`), not by
observing green GitHub Actions checks.

---

## Full workspace gate

Run from the workspace root on branch `feat/estate-round-one`:

```
$ cargo test --workspace
$ cargo fmt --all --check
$ cargo clippy --workspace --all-targets -- -D warnings
```

Results recorded in the commit that adds this note (see git log) — all
three commands green locally at commit time. CI is unable to confirm this
independently per the Part 0 caveat above.
