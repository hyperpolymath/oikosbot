# Eco-Bot 🌱

**Ecological & Economic Code Analysis Platform**

Eco-Bot is an intelligent code analysis platform that acts as a **consultant, advisor, regulator, and policy developer** for software repositories. It complements existing tools like Dependabot, CodeQL, and Copilot by adding a dedicated **ecological and economic lens** to code analysis.

## Vision

Make ecological and economic thinking **first-class** in software development, just like security and testing.

```
┌─────────────────────────────────────────────────────────────────┐
│                    EXISTING ECOSYSTEM                            │
│  Dependabot │ CodeQL │ Copilot │ Renovate                       │
│       ↓         ↓        ↓          ↓                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │              SHARED CONTEXT LAYER                        │   │
│  │  Common prompts • Shared results • Unified reporting     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                           ↑                                     │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                      ECO-BOT                             │   │
│  │  Carbon Analysis • Pareto Optimization • Policy Learning │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Features

### 🌍 Ecological Analysis
- **Carbon Intensity** - Based on SCI specification (ISO/IEC 21031:2024)
- **Energy Patterns** - Detect busy-waiting, inefficient I/O, resource waste
- **Sustainability Score** - Normalized 0-100 eco-friendliness rating

### 📊 Economic Optimization
- **Pareto Optimality** - Multi-objective optimization analysis
- **Allocative Efficiency** - Resource utilization assessment
- **Technical Debt** - Economic modeling of code quality

### 🤖 Bot Modes
- **Consultant** - Answers questions, explains trade-offs
- **Advisor** - Proactive suggestions on PRs
- **Regulator** - Enforces policy compliance
- **Policy Developer** - Learns from practice, evolves rules

### 🔗 Integrations
- GitHub Actions & Apps
- GitLab CI/CD
- Copilot/Claude Code prompts
- SARIF output for code scanning
- OpenTelemetry metrics

## Architecture

Eco-Bot is a polyglot system using the best language for each task:

| Component | Language | Purpose |
|-----------|----------|---------|
| Code Analyzer | **Haskell** | AST analysis, type safety |
| Doc Analyzer | **OCaml** | NLP, semantic extraction |
| Policy Engine | **Datalog + DeepProbLog** | Rule inference, ML |
| Bot Integration | **TypeScript** | GitHub/GitLab APIs |
| Databases | **ArangoDB + Virtuoso** | Graph + Semantic storage |
| Math Proofs | **Echidna** | Formal verification |

See [ARCHITECTURE.md](./ARCHITECTURE.md) for detailed design.

## Quick Start

### GitHub Action

```yaml
# .github/workflows/eco-bot.yml
name: Eco-Bot Analysis
on: [pull_request]

jobs:
  eco-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: hyperpolymath/eco-bot-action@v1
        with:
          mode: advisor
          eco-threshold: 60
          econ-threshold: 50
```

### AI Assistant Integration

Add Eco-Bot context to your AI coding assistants:

```bash
# Copy to your repository
cp prompts/copilot-instructions.md .github/copilot-instructions.md
cp prompts/claude-code-instructions.md .claude-code/instructions.md
```

## Metrics

### Health Index Formula

```
HealthIndex = 0.4×EcoScore + 0.3×EconScore + 0.3×QualityScore
```

### Ecological Score (SCI-based)

```
EcoScore = w₁×CarbonScore + w₂×EnergyScore + w₃×ResourceScore
```

Based on the [Software Carbon Intensity](https://sci.greensoftware.foundation/) specification.

### Economic Score (Pareto-based)

```
EconScore = w₁×ParetoDistance + w₂×AllocationEfficiency + w₃×(100-DebtRatio)
```

## Policy Engine

Eco-Bot uses a hybrid reasoning system:

**Datalog** - Deterministic rules for certain knowledge:
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
Theory → Practice → Observation → Theory Update
```

## Database Architecture

**Twin database design** for different query patterns:

| Database | Type | Use Case |
|----------|------|----------|
| ArangoDB | Graph + Document | Dependencies, history, metrics |
| Virtuoso | RDF Triple Store | Semantic knowledge, ontologies |

Both sync bidirectionally and feed into [Echidna](https://gitlab.com/hyperpolymath/echidna) for formal verification.

## Development

### Prerequisites

- Haskell (GHC 9.4+)
- OCaml (4.14+)
- Python 3.11+
- Node.js 18+
- ArangoDB 3.11+
- Virtuoso 7+

### Build

```bash
# Haskell analyzer
cd analyzers/code-haskell && cabal build

# OCaml analyzer
cd analyzers/docs-ocaml && dune build

# Bot integration
cd bot-integration && npm install && npm run build
```

### Configuration

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

## References

- [Green Software Foundation](https://greensoftware.foundation/)
- [Software Carbon Intensity (SCI) Spec](https://sci.greensoftware.foundation/)
- [DeepProbLog](https://github.com/ML-KULeuven/deepproblog)
- [Pareto Efficiency in Software](https://patrickkarsh.medium.com/pareto-efficiency-a-guide-for-software-engineers-3de566e58b75)
- [Awesome Green Software](https://github.com/Green-Software-Foundation/awesome-green-software)

## License

Apache 2.0 - See [LICENSE](./LICENSE)

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md)

---

*Making software development ecologically and economically conscious, one PR at a time.* 🌱
