#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
# Black-box smoke contract. Not SARIF schema validation or provenance verification.
# 0 = contract passed, 1 = wrong observed outcome, 2 = no check/tool failure.
set -Eeuo pipefail
trap 'echo "no check performed: container contract infrastructure failed" >&2; exit 2' ERR
for tool in docker jq; do
  command -v "$tool" >/dev/null || { echo "missing $tool" >&2; exit 2; }
done
image=${1:?usage: test-container-contract.sh IMAGE}
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/clean" "$work/firing"
printf 'fn clean() -> u32 { 1 }\n' > "$work/clean/source.rs"
cat > "$work/firing/source.rs" <<'RS'
fn work() {
    for i in 0..8 { for j in 0..8 { for k in 0..8 { let _ = i + j + k; } } }
}
RS
fail() { echo "wrong outcome: $*" >&2; exit 1; }
version=$(docker run --rm --network none "$image" --version)
[[ $version == 'oikosbot '* ]] || fail "missing version"
uid=$(docker run --rm --network none --entrypoint /usr/bin/id "$image" -u)
[[ $uid =~ ^[0-9]+$ && $uid != 0 ]] || fail "default user is root or unknown"
for fixture in clean firing; do
  docker run --rm --network none --read-only --cap-drop ALL \
    --security-opt no-new-privileges --user "$(id -u):$(id -g)" \
    -v "$work:/work" -w /work "$image" report "/work/$fixture" \
    --format sarif --output "/work/$fixture.sarif"
done
jq -e '.version == "2.1.0" and (.runs[0].results | length == 0)' "$work/clean.sarif" >/dev/null || fail "clean fixture alerted"
jq -e '.runs[0].results | any(.ruleId == "oikosbot/nested-loops")' "$work/firing.sarif" >/dev/null || fail "known rule did not fire"
echo "container smoke contract passed (not a release certification)"
