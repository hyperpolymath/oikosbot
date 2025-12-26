// SPDX-License-Identifier: MIT OR AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

= Oikos Bot
:toc: auto
:toclevels: 2

image:https://img.shields.io/badge/RSR-Bronze-cd7f32[RSR Bronze,link=https://github.com/hyperpolymath/rhodium-standard-repositories]
image:https://img.shields.io/badge/License-MIT%20OR%20AGPL--3.0--or--later-blue[License]
image:https://img.shields.io/github/v/release/hyperpolymath/oikos-bot?include_prereleases[GitHub Release]
image:https://github.com/hyperpolymath/oikos-bot/actions/workflows/ci.yml/badge.svg[CI,link=https://github.com/hyperpolymath/oikos-bot/actions/workflows/ci.yml]
image:https://api.securityscorecards.dev/projects/github.com/hyperpolymath/oikos-bot/badge[OpenSSF Scorecard,link=https://securityscorecards.dev/viewer/?uri=github.com/hyperpolymath/oikos-bot]

**Oikos Bot** — Ecological & Economic Code Analysis Platform

_οἶκος (oikos)_: the shared Greek root of both "ecology" and "economy" — the study and management of the household, whether natural or financial.

== Vision

Make ecological and economic thinking **first-class** in software development, just like security and testing.

[source]
----
                    EXISTING ECOSYSTEM
  Dependabot | CodeQL | Copilot | Renovate
       |         |        |          |
  +-------------------------------------------+
  |          SHARED CONTEXT LAYER             |
  |  Common prompts | Shared results | APIs   |
  +-------------------------------------------+
                      ^
  +-------------------------------------------+
  |              OIKOS BOT                    |
  |  Carbon Analysis | Pareto Optimization    |
  |  Policy Learning | Praxis Loop            |
  +-------------------------------------------+
----

== Features

=== Ecological Analysis
* **Carbon Intensity** — Based on SCI specification (ISO/IEC 21031:2024)
* **Energy Patterns** — Detect busy-waiting, inefficient I/O, resource waste
* **Sustainability Score** — Normalized 0-100 eco-friendliness rating

=== Economic Optimization
* **Pareto Optimality** — Multi-objective optimization analysis
* **Allocative Efficiency** — Resource utilization assessment
* **Technical Debt** — Economic modeling of code quality

=== Bot Modes
* **Consultant** — Answers questions, explains trade-offs
* **Advisor** — Proactive suggestions on PRs
* **Regulator** — Enforces policy compliance
* **Policy Developer** — Learns from practice, evolves rules

=== Integrations
* GitHub Actions & Apps
* GitLab CI/CD
* Copilot/Claude Code prompts
* SARIF output for code scanning
* OpenTelemetry metrics

== Architecture

Oikos Bot is a polyglot system using the best language for each task:

[cols="1,1,2"]
|===
|Component |Language |Purpose

|Code Analyzer |**Haskell** |AST analysis, carbon estimation
|Doc Analyzer |**OCaml** |NLP, semantic extraction
|Policy Engine |**Datalog + DeepProbLog** |Rule inference, ML
|Bot Integration |**ReScript + Deno** |GitHub/GitLab webhooks
|Orchestrator |**Rust** |High-performance coordination
|Databases |**ArangoDB + Virtuoso** |Graph + Semantic storage
|Math Proofs |**Echidna** |Formal verification
|===

See link:ARCHITECTURE.md[ARCHITECTURE.md] for detailed design.

== Quick Start

=== Using Guix

[source,bash]
----
# Enter development environment
guix shell -m guix/manifest.scm

# Or use channels
guix pull -C guix/channels.scm
guix shell oikos
----

=== Using Nix

[source,bash]
----
# Enter development environment
nix develop

# Or specific sub-shells
nix develop .#haskell
nix develop .#ocaml
nix develop .#bot
----

=== Using Containers (Vörðr/Podman)

[source,bash]
----
# Build images (uses Vörðr if available, falls back to Podman)
./containers/vordr-build.sh

# Run the stack
cd containers && podman-compose up -d

# View logs
podman-compose logs -f oikos
----

== Development

=== Prerequisites

Via Guix:
[source,bash]
----
guix shell -m guix/manifest.scm
----

Via Nix:
[source,bash]
----
nix develop
----

Or manually:
* Haskell (GHC 9.4+)
* OCaml (4.14+)
* Deno (1.40+)
* Python 3.11+
* Rust (latest stable)
* Souffle (Datalog)
* SWI-Prolog

=== Build

[source,bash]
----
# Haskell analyzer
cd analyzers/code-haskell && cabal build

# OCaml analyzer
cd analyzers/docs-ocaml && dune build

# ReScript bot (compiles to JS for Deno)
cd bot-integration
npm install rescript --save-dev  # Only for compiler
npx rescript build

# Run with Deno (no npm runtime!)
deno run --allow-net --allow-env --allow-read src/Oikos.res.js
----

=== Container Build

[source,bash]
----
# Build all images
./containers/vordr-build.sh

# Or individual images
podman build -t oikos:latest -f containers/Containerfile .
podman build -t oikos-policy:latest -f containers/Containerfile.policy .
----

== Configuration

[source,yaml]
----
# config/oikos.yaml
mode: advisor
thresholds:
  eco_minimum: 50
  eco_standard: 70
  eco_excellence: 85
weights:
  ecological: 0.4
  economic: 0.3
  quality: 0.3
integrations:
  github: true
  gitlab: true
  copilot_prompts: true
databases:
  arangodb: "http://localhost:8529"
  virtuoso: "http://localhost:8890/sparql"
----

== Metrics

=== Health Index Formula

----
HealthIndex = 0.4 × EcoScore + 0.3 × EconScore + 0.3 × QualityScore
----

=== Ecological Score (SCI-based)

Based on the https://sci.greensoftware.foundation/[Software Carbon Intensity] specification (ISO/IEC 21031:2024).

=== Economic Score (Pareto-based)

----
EconScore = w₁ × ParetoDistance + w₂ × AllocationEfficiency + w₃ × (100 - DebtRatio)
----

== Policy Engine

Oikos Bot uses a hybrid reasoning system:

**Datalog** — Deterministic rules:
[source,prolog]
----
needs_refactor(E, "eco_improvement", "high") :-
    eco_hotspot(E, _),
    dominated(E).
----

**DeepProbLog** — Probabilistic inference with neural networks:
[source,prolog]
----
nn(carbon_estimator, [CodeFeatures], P) ::
    high_carbon(Code) :- P > 0.7.
----

The **Praxis Loop** continuously learns from outcomes:
----
Theory → Practice → Observation → Theory Update
----

== Project Structure

[source]
----
oikos-bot/
├── analyzers/
│   ├── code-haskell/       # Haskell code analyzer
│   └── docs-ocaml/         # OCaml documentation analyzer
├── bot-integration/        # ReScript + Deno webhooks
│   ├── src/                # ReScript source
│   ├── bindings/           # Deno API bindings
│   └── deno.json           # Deno configuration
├── policy-engine/
│   ├── datalog/            # Souffle rules
│   ├── deepproblog/        # Probabilistic rules
│   └── python/             # Python integration
├── databases/
│   ├── arangodb/           # Graph DB schema
│   └── virtuoso/           # RDF ontology
├── containers/
│   ├── Containerfile       # Main container
│   ├── compose.yaml        # Podman compose
│   └── vordr-build.sh      # Build script (Svalinn/Podman)
├── guix/
│   ├── channels.scm        # Guix channels
│   ├── manifest.scm        # Development manifest
│   └── oikos.scm           # Package definition
├── nix/
│   └── flake.nix           # Detailed Nix flake
├── flake.nix               # Root Nix flake
├── prompts/                # AI assistant prompts
└── config/                 # Configuration files
----

== References

* https://greensoftware.foundation/[Green Software Foundation]
* https://sci.greensoftware.foundation/[Software Carbon Intensity (SCI) Spec]
* https://github.com/ML-KULeuven/deepproblog[DeepProbLog]
* https://patrickkarsh.medium.com/pareto-efficiency-a-guide-for-software-engineers-3de566e58b75[Pareto Efficiency in Software]
* https://github.com/Green-Software-Foundation/awesome-green-software[Awesome Green Software]

== License

This project is dual-licensed under **MIT** or **AGPL-3.0-or-later** (your choice).

We also encourage (but do not require) adherence to the principles of the https://github.com/hyperpolymath/palimpsest-license[Palimpsest License] — a framework for consent-based digital interaction.

See link:LICENSE.txt[LICENSE.txt] for full details.

'''

_Part of the https://github.com/hyperpolymath[hyperpolymath] ecosystem. Politically autonomous software for ecologically and economically conscious development._
