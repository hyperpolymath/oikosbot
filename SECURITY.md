# Security Policy

## Supported Versions

| Version | Supported          |
| ------- | ------------------ |
| 0.x.x   | :white_check_mark: |

As Oikos Bot is in early development, only the latest version receives security updates.

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report vulnerabilities via one of:

1. **GitHub Security Advisories**: [Report a vulnerability](https://github.com/hyperpolymath/oikos-bot/security/advisories/new)
2. **Email**: security@hyperpolymath.com (if available)

### What to include

- Type of issue (e.g., buffer overflow, SQL injection, cross-site scripting)
- Full paths of source file(s) related to the issue
- Location of affected source code (tag/branch/commit or direct URL)
- Step-by-step instructions to reproduce the issue
- Proof-of-concept or exploit code (if possible)
- Impact of the issue, including how an attacker might exploit it

### Response Timeline

- **Initial Response**: Within 48 hours
- **Status Update**: Within 7 days
- **Resolution Target**: Within 30 days for critical issues

## Security Considerations

### Data Handling

Oikos Bot analyzes code repositories and may process:
- Source code content
- Dependency information
- Build configurations
- CI/CD pipeline definitions

### Integration Security

When integrating Oikos Bot:
- Use environment variables for sensitive configuration
- Never commit API keys or tokens
- Use GitHub's encrypted secrets for CI/CD
- Review permissions granted to GitHub Apps

### Dependency Security

This project uses:
- Dependabot for automated dependency updates
- CodeQL for static analysis
- OpenSSF Scorecard for supply chain security

## Acknowledgments

We appreciate security researchers who help keep Oikos Bot secure. Contributors will be acknowledged (with permission) in release notes.
