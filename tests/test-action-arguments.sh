#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
# Exercise the actual action shell block with a recorder, NOT an image/E2E test.
set -euo pipefail
cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.."
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
# The action has one final, eight-space-indented run block. Refuse drift.
[[ $(grep -c '^      run: |$' action.yml) == 1 ]] || exit 2
sed -n '/^      run: |$/,$p' action.yml | tail -n +2 | sed 's/^        //' > "$work/action.sh"
bash -n "$work/action.sh"
mkdir "$work/bin"
cat > "$work/bin/docker" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$@" > "$RECORDED_ARGS"
SH
chmod +x "$work/bin/docker"
export PATH="$work/bin:$PATH" RECORDED_ARGS="$work/args"
export GITHUB_WORKSPACE="$work/workspace" MODE=report TARGET='source with spaces' BASE=''
export FORMAT=sarif OUTFILE=results.sarif CONFIG='' THRESHOLD='' PR_BODY='' DO_CHECK=false IMAGE=fixture-image
bash "$work/action.sh"
grep -Fx 'source with spaces' "$work/args" >/dev/null
MODE=compare BASE=base FORMAT=sarif bash "$work/action.sh"
grep -Fx json "$work/args" >/dev/null
grep -Fx -- --output "$work/args" >/dev/null
expect_rejected() {
  rm -f "$work/args"
  local status=0
  env "$@" bash "$work/action.sh" > "$work/rejection" 2>&1 || status=$?
  [[ $status == 2 && ! -e "$work/args" ]] || { cat "$work/rejection"; exit 1; }
}
expect_rejected MODE=invalid
expect_rejected MODE=compare BASE=''
expect_rejected MODE=compare BASE=base THRESHOLD=50
expect_rejected DO_CHECK=typo
printf 'action argument controls passed (no container was executed)\n'
