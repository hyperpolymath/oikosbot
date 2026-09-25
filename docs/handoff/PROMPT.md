# Handoff prompt — oikosbot

Copy everything between the marker lines into a new chat on `hyperpolymath/oikosbot`.

----- BEGIN PROMPT -----

You're continuing work on `hyperpolymath/oikosbot`. A previous session left a
handoff package **committed in the repo** at `docs/handoff/`. Read these first:

- `docs/handoff/CI-EVIDENCE.md`: run IDs, exact annotations, root causes, and the
  estate lockfile gate's acceptance logic
- `docs/handoff/open-issues.txt`: all 7 open issue bodies, verbatim. These are the
  acceptance criteria.
- `docs/handoff/standards-ref/`: DEED/K9 canon frozen at `hyperpolymath/standards@2479cf76`.
  The standards Python tools are stored as `*.py.txt` because repo policy blocks new
  `.py` files. To run them, copy both into a temp dir and rename them to `.py`
  (`a2ml_to_deed` imports `deed_lint`).
- `tools/ci/linter-verify.sh`, `tools/ci/lockcheck.sh`: local re-verification.

## Hard facts (don't rediscover them)
1. **Only the owner's merges to `main` trigger Actions.** Agent pushes and PRs produce
   no runs. "Verified in CI" means after the owner merges.
2. **There is no Rust, Haskell or Elixir toolchain in the sandbox.** Rust/Haskell changes
   can only be verified in CI, so keep them small and reviewable. Say this plainly in PRs.
3. In the prior sandbox, Actions logs, artifacts and the `gh-actions-lock` binary
   download all failed (EOF). Use the check-run **annotations** API instead:
   `gh api repos/hyperpolymath/oikosbot/check-runs/<job_id>/annotations`.
4. Repo language policy blocks new `.py`, `.rb`, `.pl`, `.java` and `.kt` files. Put
   Python inside bash heredocs, the way `workflow-linter.yml` does.

## Step 0: CI fixes (done on branch `arena/01a0da74-oikosbot`; confirm they merged)
- SPDX: `gh actions-lock` had pushed the SPDX line off line 1 in all 15 workflows.
- actions.lock: `codeql.yml` now uses `codeql-action@v4.38.1`. Its lock list is
  corrected, and 5 superseded entries were pruned. Do **not** prune the 8 "unreachable"
  entries; they cover the standards reusable workflows.
- Check with: `gh run list --branch main -L 15`. Workflow Security Linter and
  Governance should be green. If Governance is still red, read its annotations. The
  authoritative verifier is `gh actions-lock --verify-local --json=valid,findings`.
- **Still open: Hypatia** (1 high/critical finding, red since at least 2026-09-20,
  not diagnosed). Get the `hypatia-scan-findings` artifact or the Security tab entry,
  then fix it or commit a validated `.hypatia-baseline.json` + `scripts/apply-baseline.sh`.
  Never baseline a real vulnerability just to go green. Report what the finding is.

## #12: taxonomy reconciliation, delivered as the DEED/K9 migration
**Owner ruling (2026-09-25): the DEED migration REPLACES the six `.a2ml` descriptiles
(`.machine_readable/descriptiles/{META,ECOSYSTEM,AGENTIC,NEUROSYM,PLAYBOOK,STATE}.a2ml`).
Don't land it alongside them.** Remove the `.a2ml` files in the same change that
introduces the `.deed`. Update every reference (grep for the filenames and for
`descriptiles/`), including `0-AI-MANIFEST.a2ml`, the Justfile, docs and policies.
Note that the issue body cites the stale path `.machine_readable/6a2/`.

Also settle the vocabulary: the #12 proposal makes NEUROSYM canonical (`wish`,
`externalities`, add `preventive`), and `policies/finding_taxonomy.ecl` already encodes
it. Confirm with the owner before flipping the direction.

What was measured with `standards-ref/.../a2ml_to_deed.py` (standards@2479cf76),
`--canonical-name oikosbot --beholding-chora '#u5"estate/chora"'`:
- `agentic`: **converts cleanly** (output must be named `<stem>_chora.deed`, e.g.
  `oikosbot_chora.deed`, or the post-condition lint fails).
- `neurosym`: **FAIL-CLOSED: unknown section(s) ['finding-taxonomy']**. The family-4
  mapping (`mappings/agentic-neurosym-playbook-to-repo-deed.adoc`) has no row for
  `[finding-taxonomy]`. That blocks #12: it needs a **standards-side extension** to the
  family-4 mapping and converter, or an explicit **owner ruling** on where the taxonomy
  lives in the deed. Don't hand-roll an unmapped clause; the canon treats silent
  re-homing as the failure mode.
- `playbook`: **FAIL-CLOSED: unknown section(s) ['datastore', 'local-dev', 'overview']**.
  Same class of blocker (a second one).
- `meta-ecosystem`: must go through `full DIR`. `full` fails closed because there is
  **no `CLADE.a2ml`** (family 1 is a prerequisite). `clade` mode on META rejects its
  sections. `meta-ecosystem-to-repo-deed.adoc` §"open questions" also leaves open whether
  the `[maintenance-axes]` triple stays in META. That is exactly #12's question.
- `state-scan STATE.a2ml`: `[critical-next-actions]` (9 rows) and `[maintenance-status]`
  are journal content and are tombstoned under option B. **Ruling pending: standards#843.**
- The canon's ordering (`mappings/README.adoc`): the estate wave (6) comes **after
  families 1–5 are green in CI**. Check standards' `deed-conformance.yml` status before
  migrating. If 1–5 aren't green, stop and report.
- Contractiles (`.machine_readable/contractiles/*.a2ml`) → K9 (`*.k9.ncl`) is a separate
  family. See `standards-ref/1-formats/k9/SPEC.adoc` and standards'
  `.machine_readable/contractiles/*/*.k9.ncl` for the target layout. It isn't covered by
  the replace ruling, so ask before touching it.

Deliverable: a standards-side issue or PR for the family-4 `[finding-taxonomy]` and
PLAYBOOK gaps (or the owner's ruling), then the oikosbot replacement PR once it's unblocked.

## #18: taxonomy tags through Rust `Finding`
Depends on #12's vocabulary decision. Add `intent` (derived from confidence),
`maintenance` and `locus` to the types in `crates/oikosbot-metrics`, populate them in
`crates/oikosbot-analysis`, and emit them in `crates/oikosbot-sarif` (SARIF `properties`)
and PR comments. Use enums, not strings. Can only be verified in CI.

## #48: wire calibration in
Follow the issue exactly, including its **Caution**: map `patterns.rs` detections onto
`OperationKind`, use `calibration::estimate_operation()` for recognised patterns, keep
`Estimated` for everything else, and propagate `ResourceRange`. Add the three falsifier
tests. Include before/after numbers in the PR. Since nothing compiles here, derive them
by reasoning and label them that way, or ask the owner to run it. Never relabel
estimates as Calibrated.

## #16 / #17: docs
#16: `docs/README.adoc` index first (quick win), per-crate READMEs,
`analyzers/code-haskell/README`, a CI runbook (use `CI-EVIDENCE.md`), and a `just`
target reference taken from the actual Justfile. #17: end-user guide. Define `BOT_MODE`
from the code, not by guessing. Document the SARIF shape from `crates/oikosbot-sarif`.
`DEPLOY.adoc` stays gated on AffineScript operational parity. AsciiDoc, matching the repo.

## #81: whole test/benchmark backlog
Work P0 → P1 → benchmarks, following the issue's Acceptance rules. Every analyzer rule
gets a paired silent/firing fixture. Validate SARIF against the 2.1.0 schema. Seeded
property tests. Split the work into several reviewable PRs and tick the issue's
checkboxes as each one merges.

## #82: close and re-file
It's a conditional policy with an unresolvable blocker (no canonical Hexadeca authority
exists; see the issue's last section), so it isn't actionable as a task. **Confirm with
the owner, then** close it with a comment linking the re-files: (a) a policy doc in-repo
(e.g. `docs/policies/interface-gating.adoc`) recording the Idris2/Zig/SNIF/Hexadeca
trigger conditions, and (b) an issue, standards-side if the owner agrees, to locate or
publish the Hexadeca authority. Keep #81 cross-referenced.

## Working rules
Commit only to the session branch you're given. Open PRs from it. Ask the owner when
the canon is silent rather than inventing structure.

----- END PROMPT -----
