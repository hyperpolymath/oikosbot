# Eco-Bot

**Ecological & Economic Code Analysis Platform**

Eco-Bot is an intelligent code analysis platform that acts as a **consultant, advisor, regulator, and policy developer** for software repositories. It complements existing tools like Dependabot, CodeQL, and Copilot by adding a dedicated **ecological and economic lens** to code analysis.

## Vision

Make ecological and economic thinking **first-class** in software development, just like security and testing.

```
                    EXISTING ECOSYSTEM
  Dependabot | CodeQL | Copilot | Renovate
       |         |        |          |
  +-------------------------------------------+
  |          SHARED CONTEXT LAYER             |
  |  Common prompts | Shared results | APIs   |
  +-------------------------------------------+
                      ^
  +-------------------------------------------+
  |                ECO-BOT                    |
  |  Carbon Analysis | Pareto Optimization    |
  |  Policy Learning | Praxis Loop            |
  +-------------------------------------------+
```

## Features

### Ecological Analysis
- **Carbon Intensity** - Based on SCI specification (ISO/IEC 21031:2024)
- **Energy Patterns** - Detect busy-waiting, inefficient I/O, resource waste
- **Sustainability Score** - Normalized 0-100 eco-friendliness rating

### Economic Optimization
- **Pareto Optimality** - Multi-objective optimization analysis
- **Allocative Efficiency** - Resource utilization assessment
- **Technical Debt** - Economic modeling of code quality

### Bot Modes
- **Consultant** - Answers questions, explains trade-offs
- **Advisor** - Proactive suggestions on PRs
- **Regulator** - Enforces policy compliance
- **Policy Developer** - Learns from practice, evolves rules

### Integrations
- GitHub Actions & Apps
- GitLab CI/CD
- Copilot/Claude Code prompts
- SARIF output for code scanning
- OpenTelemetry metrics

## Architecture

Eco-Bot is a polyglot system using the best language for each task:

| Component | Language | Purpose |
|-----------|----------|---------|
| Code Analyzer | **Haskell** | AST analysis, carbon estimation |
| Doc Analyzer | **OCaml** | NLP, semantic extraction |
| Policy Engine | **Datalog + DeepProbLog** | Rule inference, ML |
| Bot Integration | **ReScript + Deno** | GitHub/GitLab webhooks |
| Orchestrator | **Rust** | High-performance coordination |
| Databases | **ArangoDB + Virtuoso** | Graph + Semantic storage |
| Math Proofs | **Echidna** | Formal verification |

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed design.

## Technology Stack

### Package Management
- **Guix** - Primary package manager and reproducible builds
- **Nix** - Shared development environments via flakes
- **No npm** - Zero JavaScript runtime dependencies

### Containerization
- **nerdctl** - containerd-native CLI (no Docker daemon)
- **buildkit** - Efficient multi-stage builds
- **/cerro-torre** - Base image from hyperpolymath

### Languages (No TypeScript/JavaScript at runtime)
- **Haskell** - Pure functional code analysis
- **OCaml** - Documentation processing
- **ReScript** - Compiles to clean JS for Deno
- **Python** - Policy engine and ML
- **Rust** - Orchestration layer

## Quick Start

### Using Guix

```bash
# Enter development environment
guix shell -m guix/manifest.scm

# Or use channels
guix pull -C guix/channels.scm
guix shell eco-bot
```

### Using Nix

```bash
# Enter development environment
nix develop

# Or specific sub-shells
nix develop .#haskell
nix develop .#ocaml
nix develop .#bot
```

### Using Containers (nerdctl)

```bash
# Build images
./containers/nerdctl-build.sh

# Run the stack
cd containers && nerdctl compose up -d

# View logs
nerdctl compose logs -f eco-bot
```

## Development

### Prerequisites

Via Guix:
```bash
guix shell -m guix/manifest.scm
```

Via Nix:
```bash
nix develop
```

Or manually:
- Haskell (GHC 9.4+)
- OCaml (4.14+)
- Deno (1.40+)
- Python 3.11+
- Rust (latest stable)
- Souffle (Datalog)
- SWI-Prolog

### Build

```bash
# Haskell analyzer
cd analyzers/code-haskell && cabal build

# OCaml analyzer
cd analyzers/docs-ocaml && dune build

# ReScript bot (compiles to JS for Deno)
cd bot-integration
npm install rescript --save-dev  # Only for compiler
npx rescript build

# Run with Deno (no npm runtime!)
deno run --allow-net --allow-env --allow-read src/Main.res.js
```

### Container Build

```bash
# Ensure /cerro-torre base image is available
nerdctl pull /cerro-torre

# Build all images
./containers/nerdctl-build.sh

# Or individual images
nerdctl build -t eco-bot:latest -f containers/Containerfile .
nerdctl build -t eco-bot-policy:latest -f containers/Containerfile.policy .
```

## Configuration

```yaml
# config/eco-bot.yaml
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
```

## Metrics

### Health Index Formula

```
HealthIndex = 0.4 x EcoScore + 0.3 x EconScore + 0.3 x QualityScore
```

### Ecological Score (SCI-based)

Based on the [Software Carbon Intensity](https://sci.greensoftware.foundation/) specification (ISO/IEC 21031:2024).

### Economic Score (Pareto-based)

```
EconScore = w1 x ParetoDistance + w2 x AllocationEfficiency + w3 x (100-DebtRatio)
```

## Policy Engine

Eco-Bot uses a hybrid reasoning system:

**Datalog** - Deterministic rules:
```prolog
needs_refactor(E, "eco_improvement", "high") :-
    eco_hotspot(E, _),
    dominated(E).
```

**DeepProbLog** - Probabilistic inference with neural networks:
```prolog
nn(carbon_estimator, [CodeFeatures], P) ::
    high_carbon(Code) :- P > 0.7.
```

The **Praxis Loop** continuously learns from outcomes:
```
Theory -> Practice -> Observation -> Theory Update
```

## Database Architecture

**Twin database design** for different query patterns:

| Database | Type | Use Case |
|----------|------|----------|
| ArangoDB | Graph + Document | Dependencies, history, metrics |
| Virtuoso | RDF Triple Store | Semantic knowledge, ontologies |

Both sync bidirectionally and feed into [Echidna](https://gitlab.com/hyperpolymath/echidna) for formal verification.

## Project Structure

```
eco-bot/
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
│   ├── Containerfile       # Main container (cerro-torre based)
│   ├── compose.yaml        # nerdctl compose
│   └── nerdctl-build.sh    # Build script
├── guix/
│   ├── channels.scm        # Guix channels
│   ├── manifest.scm        # Development manifest
│   └── eco-bot.scm         # Package definition
├── nix/
│   └── flake.nix           # Detailed Nix flake
├── flake.nix               # Root Nix flake
├── prompts/                # AI assistant prompts
└── config/                 # Configuration files
```

## References

- [Green Software Foundation](https://greensoftware.foundation/)
- [Software Carbon Intensity (SCI) Spec](https://sci.greensoftware.foundation/)
- [DeepProbLog](https://github.com/ML-KULeuven/deepproblog)
- [Pareto Efficiency in Software](https://patrickkarsh.medium.com/pareto-efficiency-a-guide-for-software-engineers-3de566e58b75)
- [Awesome Green Software](https://github.com/Green-Software-Foundation/awesome-green-software)

## License

Apache 2.0 - See [LICENSE](./LICENSE)

---

*Making software development ecologically and economically conscious, one PR at a time.*
