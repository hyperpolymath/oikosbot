<!--
SPDX-License-Identifier: CC-BY-SA-4.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
## Machine-Readable Artefacts

The following files in `.machine_readable/6a2/` contain structured project metadata (a2ml format; the prior `.scm` Guile-Scheme files were retired estate-wide):

- `STATE.a2ml` - Current project state and progress
- `META.a2ml` - Architecture decisions and development practices
- `ECOSYSTEM.a2ml` - Position in the ecosystem and related projects
- `AGENTIC.a2ml` - AI agent interaction patterns
- `NEUROSYM.a2ml` - Neurosymbolic integration config
- `PLAYBOOK.a2ml` - Operational runbook

---

# CLAUDE.md - AI Assistant Instructions

## Language Policy (Hyperpolymath Standard)

> **Disambiguation (READ FIRST — see [`DISAMBIGUATION.adoc`](../DISAMBIGUATION.adoc)):** this repo is **OikosBot** and **only** OikosBot. Do not conflate it with two separate, similarly-named projects:
> - **oikos** — the stock-flow-consistent economics *DSL* (crates `oikos-syntax`/`oikos-parser`/`oikos-desugar`/`oikos-check`). It lives in its **own** repo `hyperpolymath/oikos-economics-accounting-dsl`, **not here**. Do **not** add `oikos-*` DSL crates to this repo.
> - **sustainabot** — a *reserved member slot* of `hyperpolymath/gitbot-fleet` (`bots/sustainabot/`). **Not here.** A misfiled copy of OikosBot once lived there; it was moved into this repo and renamed `sustainabot-*` → `oikosbot-*`. Do **not** reintroduce the `sustainabot` name here. (The `crates/oikosbot-fleet` bridge now publishes under its own `BotId::Oikosbot` identity — distinct from the fleet's reserved `BotId::Sustainabot` slot.)
>
> **Policy refresh 2026-06-19**: This repo has two surfaces, both OikosBot. (1) The **Rust analysis workspace** at the repo root — crates `oikosbot-*` (`oikosbot-cli`/`-analysis`/`-metrics`/`-sarif`/`-eclexia`, plus the optional, workspace-excluded `oikosbot-fleet` bridge). The default `cargo` workspace builds **standalone** with no dependency on gitbot-fleet. (2) **OikosBot's webhook bot** at `bot-integration-affine/` is **AffineScript** as of the 2026-05-28 legacy-shutoff (oikos#41) — the previous ReScript implementation at `bot-integration/` was removed in the same PR. Write no new ReScript or TypeScript here. MPL-1.0 / MPL-1.0-or-later are banned; rewrite to MPL-2.0 wherever encountered (DR-010 supersedes DR-002).

### ALLOWED Languages & Tools

| Language/Tool | Use Case | Notes |
|---------------|----------|-------|
| **Rust** | OikosBot analysis workspace (`crates/oikosbot-*`) | Repo root; performance-critical. NOT the `oikos-*` DSL (separate repo). |
| **AffineScript** (`.affine`) | OikosBot (`bot-integration-affine/`) | Affine types, dependent types, row polymorphism, extensible effects |
| **Haskell** | OikosBot analyser backend (`analyzers/`) | Existing surface; pure analysis |
| **Deno** | Runtime & package management | Replaces Node/npm/bun |
| **Tauri 2.0+** | Mobile apps (iOS/Android) | Rust backend + web UI |
| **Dioxus** | Mobile apps (native UI) | Pure Rust, React-like |
| **Gleam** | Backend services | Runs on BEAM or compiles to JS |
| **Bash/POSIX Shell** | Scripts, automation | Keep minimal |
| **Nickel** | Configuration language | For complex configs |
| **A2ML** | State/meta files | `.machine_readable/6a2/*.a2ml` |
| **Julia** | Batch scripts, data processing | Per RSR |
| **OCaml** | AffineScript compiler upstream | Not in this repo (lives in `hyperpolymath/affinescript`) |
| **Ada** | Safety-critical systems | Where required |

### BANNED - Do Not Use

| Banned | Replacement |
|--------|-------------|
| TypeScript | AffineScript (for OikosBot) / Rust (for DSL) |
| **ReScript** (new files) | **AffineScript** — the legacy `bot-integration/` ReScript was removed 2026-05-28 (oikos#41) |
| JavaScript (new files) | AffineScript |
| Node.js | Deno |
| npm | Deno |
| Bun | Deno |
| pnpm/yarn | Deno |
| Go | Rust |
| Python | Julia/Rust/AffineScript |
| Java/Kotlin | Rust/Tauri/Dioxus |
| Swift | Tauri/Dioxus |
| React Native | Tauri/Dioxus |
| Flutter/Dart | Tauri/Dioxus |

### Mobile Development

**No exceptions for Kotlin/Swift** - use Rust-first approach:

1. **Tauri 2.0+** - Web UI (AffineScript) + Rust backend, MIT/Apache-2.0
2. **Dioxus** - Pure Rust native UI, MIT/Apache-2.0

Both are FOSS with independent governance (no Big Tech).

### Enforcement Rules

1. **No new TypeScript files** - Write new code in AffineScript
2. **No new ReScript files** - Legacy `bot-integration/` ReScript was retired 2026-05-28 (oikos#41); the AS port lives at `bot-integration-affine/`
3. **No package.json - use deno.json deps** - Use deno.json imports
4. **No node_modules in production** - Deno caches deps automatically
5. **No Go code** - Use Rust instead
6. **No Python anywhere** - Use Julia for data/batch, Rust for systems
7. **No Kotlin/Swift for mobile** - Use Tauri 2.0+ or Dioxus
8. **MPL-1.0 / MPL-1.0-or-later are non-conforming** - Rewrite to MPL-2.0 (DR-010)

### Package Management

- **Primary**: Guix (guix.scm)
- **Fallback**: Nix (flake.nix)
- **JS deps**: Deno (deno.json imports)

### Security Requirements

- No MD5/SHA1 for security (use SHA256+)
- HTTPS only (no HTTP URLs)
- No hardcoded secrets
- SHA-pinned dependencies
- SPDX license headers on all files

## Sustainable-Development Guidance

Folded from the former `prompts/claude-code-instructions.md`. OikosBot's whole
purpose is an ecological & economic review lens; apply the same lens when writing
or refactoring code here — these are also the principles OikosBot evaluates in the
repositories it monitors.

### 1. Carbon Awareness (weight 40%)

- Prefer lower algorithmic complexity (O(n) over O(n²), O(log n) over O(n)).
- Minimise allocations and CPU cycles; batch I/O; cache strategically.
- Sleep efficiently — event-driven, not busy-wait/polling.
- Ask: "Will this run millions of times in production?"

### 2. Economic Efficiency (weight 30%)

- Pareto optimality: don't improve one axis at another's expense without saying so.
- Allocative efficiency: put effort where it creates the most value.
- Track and minimise technical debt; weigh opportunity cost.
- Ask: "Does this abstraction justify its complexity?"

### 3. Quality (weight 30%)

- Keep cyclomatic complexity reasonable (< ~10 per function).
- Minimise coupling; cover critical paths with tests.
- Document non-obvious trade-offs.

### Trade-off documentation

When making a significant trade-off, record: the competing objectives, the
decision taken, why it is Pareto-optimal for this context, and the rough metric
impact (carbon / performance / complexity).

### Anti-patterns to avoid

Busy-waiting, N+1 queries, unbounded caches (use LRU + size limits), polling
where webhooks/SSE/event-driven would do, and premature optimisation (profile
first, then optimise hotspots).

