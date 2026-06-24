<!--
SPDX-License-Identifier: CC-BY-SA-4.0
SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# `oikosbot-fleet` — optional gitbot-fleet bridge

This crate is the **only** part of OikosBot that knows about
[`hyperpolymath/gitbot-fleet`](https://github.com/hyperpolymath/gitbot-fleet).
It converts OikosBot's `AnalysisResult`s into the fleet's shared-context
`Finding`s so OikosBot can run as a fleet member.

## Why it is excluded from the default workspace

OikosBot and gitbot-fleet are **separate projects** (see
[`../../DISAMBIGUATION.adoc`](../../DISAMBIGUATION.adoc)). To keep that boundary
honest, the root `Cargo.toml` lists this crate under `exclude`, **not**
`members`. As a result:

- `cargo build` / `cargo test` at the repo root build OikosBot with **zero**
  dependency on gitbot-fleet.
- This bridge is compiled only when you ask for it explicitly.

## Building / running the bridge

The bridge has a `path` dependency on `gitbot-fleet/shared-context`, so check
out gitbot-fleet as a **sibling** of this repository:

```text
parent/
├── oikosbot/        # this repo
└── gitbot-fleet/    # hyperpolymath/gitbot-fleet
```

Then:

```bash
cargo build --manifest-path crates/oikosbot-fleet/Cargo.toml
cargo run   --manifest-path crates/oikosbot-fleet/Cargo.toml -- <repo-path> [--context ctx.json]
```

## Fleet identity

The bridge publishes under its own **`BotId::Oikosbot`** identity, so the fleet
distinguishes OikosBot from the separate, reserved `sustainabot` slot. This
requires a `gitbot-shared-context` that provides the `Oikosbot` variant (added in
gitbot-fleet alongside this change); build with a sibling gitbot-fleet that has
it.
