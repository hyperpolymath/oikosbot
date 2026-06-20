# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# OikosBot project automation

haskell_dir := "analyzers/code-haskell"

all: check

check: rust-build rust-test haskell-build haskell-test

# Rust analysis workspace. The default workspace excludes crates/oikosbot-fleet
# (the optional gitbot-fleet bridge) so OikosBot builds standalone.
rust-build:
    cargo build --workspace

rust-test:
    cargo test --workspace

rust-fmt-check:
    cargo fmt --check

haskell-build:
    cd {{ haskell_dir }} && cabal build all

haskell-test:
    cd {{ haskell_dir }} && cabal test all

affine-check:
    ./bot-integration-affine/check.sh

shell-check:
    bash -n bot-integration-affine/check.sh

clean:
    cargo clean
    cd {{ haskell_dir }} && cabal clean
