# CI evidence — oikosbot `main` @ c9637fb (#105), gathered 2026-09-25

Sandbox limitations when this was gathered: `gh run view --log-failed`, artifact
downloads and the `gh-actions-lock` extension binary all failed with EOF (blob
storage blocked). Code-scanning API returned 403. Evidence below comes from the
check-run **annotations** API and by re-running gate logic locally.

## Run IDs (push of #105)

| Workflow | Run | Result | Failing step |
|---|---|---|---|
| Workflow Security Linter | 36187443576 | failure | `Check SPDX headers` (exit 15) |
| Governance | 36187444802 | failure | `governance / Actions lockfile verify` → `Verify actions.lock (or SHA pins during the grace window)` |
| Hypatia Security Scan | 36187444852 | failure | `Block high and critical findings when requested` |
| CI, CodeQL, Pages, Mirror, Language Policy, Secret Scanner | — | success | — |

Earlier: #104 (dependabot bump) gave `startup_failure` on CI / Pages / Publish Image
because the lock didn't cover the bumped refs. #105 fixed that part.

## Exact annotations

- Linter: `Process completed with exit code 15.`
- Governance: `actions-lock gate: lockfile verification FAILED (exit 1). Regenerate with scripts/update-actions-lock.sh in the same PR as the uses: change.`
- Hypatia: `Hypatia found 1 current high or critical finding(s); see the SARIF artifact`

## Root causes and fixes (committed on `arena/01a0da74-oikosbot`)

### 1. SPDX linter: 15 of 15 workflows fail
`gh actions-lock` put `# This workflow is managed by gh actions-lock.` on line 1,
above the SPDX line, and a second copy below it. The linter checks `head -1` only.
**Fix:** SPDX goes back on line 1, followed by one marker line. Reproduced locally:
unfixed = exactly 15 errors (matching exit 15), fixed = pass.

### 2. Governance lockfile gate
- `codeql.yml` used `github/codeql-action/{init,analyze}@1c5b675…  # v3` (that SHA is
  actually v4.38.1, so the comment was wrong too), but `actions.lock` → `workflows` →
  `codeql.yml` still listed `github/codeql-action@b96794…` (v4.38.0). The verifier
  rejects a mismatch between a workflow's lock list and what it actually uses.
- Stale entries left behind by the dependabot bump:
  `codeql-action@v4.38.0`, `codeql-action@b96794…`, `docker/build-push-action@v7.3.0`,
  `docker/setup-buildx-action@v4.3.0`, `haskell-actions/setup@v2.12.0`.

**Fix:** `codeql.yml` now uses `@v4.38.1` (same ref as `ci.yml`, already locked).
The lock list is updated and the 5 stale entries are removed.

**Deliberately kept:** 8 entries that no workflow here reaches (`actions/checkout@3d3c42e…`,
`actions/cache@55cc834…`, `actions/upload-artifact@043fb46…`, `erlef/setup-beam@54075bc…`,
`ossf/scorecard-action`, `webfactory/ssh-agent`, `dtolnay/rust-toolchain`,
`editorconfig-checker`). These are the pinned actions *inside* the standards reusable
workflows. For example, `hypatia-scan-reusable.yml@da2c748` uses setup-beam@54075bc,
cache@55cc834 and upload-artifact@043fb46.

**Gate acceptance logic** (standards `scripts/update-actions-lock.sh` @ `874ffe58`, called
by `governance-reusable.yml@da2c748` line ~1286): it runs
`gh actions-lock --verify-local --json=valid,findings`. The lock is accepted if
`.valid == true`; advisory findings such as `sha-as-ref` are logged but don't fail it.
If `valid == false`, it is accepted only when **every** finding is `category == stale`
for a reusable-workflow dependency the workflow really references, and there is at
least one such finding. Anything else fails closed.

**Confidence:** high for the SPDX fix (reproduced exactly). Medium-high for the lock fix:
the authoritative verifier binary couldn't be downloaded here, so this is confirmed by
`tools/ci/lockcheck.sh` (a re-implementation) plus the gate's own logic. The first
merge to `main` is the real test.

### 3. Hypatia: 1 high/critical finding, **NOT diagnosed**
This has failed on every run since at least 2026-09-20, so it wasn't caused by the bumps.
We couldn't get the SARIF artifact or code-scanning alerts. Next steps: download the
`hypatia-scan-findings` artifact from run 36187444852, or look in the Security tab under
tool "Hypatia". Then either fix it or commit `.hypatia-baseline.json` +
`scripts/apply-baseline.sh` (schema `.machine_readable/hypatia-baseline.schema.json`
in standards). Committing a baseline makes the gate blocking against *unbaselined*
findings only.

## Re-run verification

    pip install pyyaml            # if missing
    bash tools/ci/linter-verify.sh   # runs workflow-linter.yml's run: steps verbatim + lockcheck
    bash tools/ci/lockcheck.sh       # actions.lock ↔ workflows consistency only

## Environmental facts
- **Only merges to `main` by the owner trigger Actions.** Pushes and PRs from agent
  branches don't produce runs, so CI verification means merging.
- There is no Rust, Haskell or Elixir toolchain in the sandbox. Rust/Haskell changes can
  only be verified in CI.
