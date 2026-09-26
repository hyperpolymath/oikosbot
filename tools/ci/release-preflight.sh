#!/usr/bin/env bash
# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell
# Local evidence collection, not permission to release. See docs/LAUNCH-READINESS.adoc.
set -Eeuo pipefail
for tool in cargo rustc docker jq; do
  command -v "$tool" >/dev/null || { echo "no check performed: missing $tool" >&2; exit 2; }
done
cargo fmt --check
cargo test --workspace --locked
cargo test --workspace --all-features --locked
cargo clippy --workspace --locked
bash tests/test-eclexia-adapter.sh
docker build -f Containerfile -t oikosbot:preflight .
bash tests/test-container-contract.sh oikosbot:preflight
echo 'Local evidence collected. Release remains subject to the launch-readiness review.'
