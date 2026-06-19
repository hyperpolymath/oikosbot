# SPDX-License-Identifier: MPL-2.0
# SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
#
# OikosBot project automation

haskell_dir := "analyzers/code-haskell"

all: check

check: haskell-build haskell-test

haskell-build:
    cd {{ haskell_dir }} && cabal build all

haskell-test:
    cd {{ haskell_dir }} && cabal test all

affine-check:
    ./bot-integration-affine/check.sh

shell-check:
    bash -n bot-integration-affine/check.sh

clean:
    cd {{ haskell_dir }} && cabal clean
