// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath

# Contributing to Oikos Bot

Thank you for your interest in contributing to Oikos Bot! This document provides guidelines and information for contributors.

## Code of Conduct

By participating in this project, you agree to abide by our [Code of Conduct](CODE_OF_CONDUCT.md).

## Getting Started

### Prerequisites

- Deno 2.1+
- Git
- For full development: GHC 9.4+, OCaml 4.14+, Rust (stable)

### Development Setup

```bash
# Clone the repository
git clone https://github.com/hyperpolymath/oikos-bot.git
cd oikos-bot

# Using Guix (recommended)
guix shell -m guix/manifest.scm

# Using Nix
nix develop

# Or manually install dependencies
```

### Building

```bash
# Bot integration (ReScript + Deno)
cd bot-integration
npm install  # Only for ReScript compiler
deno task build:rescript

# Run locally
deno task dev
```

## Language Policy

Oikos Bot follows the **Hyperpolymath Language Standard**:

### Allowed Languages

| Language | Use Case |
|----------|----------|
| ReScript | Primary application code |
| Deno | Runtime & package management |
| Rust | Performance-critical, systems |
| Haskell | Code analysis engine |
| OCaml | Documentation analyzer |
| Guile Scheme | Configuration (STATE.scm, META.scm) |
| Bash/POSIX | Scripts, automation (minimal) |

### Not Permitted

- TypeScript (use ReScript instead)
- Node.js/npm/Bun (use Deno instead)
- Go (use Rust instead)
- Python (except for DeepProbLog policy engine)

## How to Contribute

### Reporting Bugs

1. Check existing issues to avoid duplicates
2. Use the bug report template
3. Include:
   - Clear description of the bug
   - Steps to reproduce
   - Expected vs actual behavior
   - Environment details (OS, versions)

### Suggesting Features

1. Check existing issues and discussions
2. Use the feature request template
3. Explain the use case and benefit

### Pull Requests

1. **Fork** the repository
2. **Create a branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. **Make changes** following our coding standards
4. **Test** your changes:
   ```bash
   cd bot-integration && deno task test
   ```
5. **Commit** with clear messages:
   ```bash
   git commit -m "feat: add eco-score visualization"
   ```
6. **Push** and create a PR

### Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` New feature
- `fix:` Bug fix
- `docs:` Documentation
- `refactor:` Code refactoring
- `test:` Tests
- `chore:` Maintenance

### Code Style

- **ReScript**: Follow ReScript best practices
- **Deno**: Use `deno fmt` and `deno lint`
- **Haskell**: Use `ormolu` for formatting
- **OCaml**: Use `ocamlformat`
- **All files**: Include SPDX headers

```
// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2024-2025 hyperpolymath
```

## Architecture Overview

```
oikos-bot/
├── analyzers/
│   ├── code-haskell/    # Haskell code analyzer
│   └── docs-ocaml/      # OCaml documentation analyzer
├── bot-integration/     # ReScript + Deno webhooks
├── policy-engine/       # Datalog + DeepProbLog
├── containers/          # Container definitions
├── guix/                # Guix package definitions
└── nix/                 # Nix flake
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for detailed design.

## Testing

### Running Tests

```bash
# Bot integration
cd bot-integration
deno task test

# Haskell analyzer
cd analyzers/code-haskell
cabal test

# All components
make test
```

### Writing Tests

- Write tests for new features
- Maintain or improve coverage
- Include edge cases

## Documentation

- Update README.md for user-facing changes
- Update ARCHITECTURE.md for design changes
- Add inline comments for complex logic
- Keep examples up to date

## Release Process

1. Version bump in relevant files
2. Update CHANGELOG.md
3. Create release tag
4. CI builds and publishes

## Getting Help

- Open an issue for questions
- Check existing documentation
- Review closed issues

## Recognition

Contributors are recognized in:
- Release notes
- CONTRIBUTORS file (coming soon)

Thank you for contributing to ecological and economically conscious software development!
