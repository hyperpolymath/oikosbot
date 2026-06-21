<!--
SPDX-License-Identifier: MPL-2.0
Copyright (c) Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>
-->
# Security Policy
*OikosBot* adheres to **Rhodium Standard Repo (RSR)** principles, emphasizing **reversibility, attestation, and minimal attack surfaces**.

---

## Supported Versions
| Version | Supported          | Notes                                  |
|---------|--------------------|----------------------------------------|
| 0.x.x   | :white_check_mark: | Only the **latest minor version** receives security updates. |

> **Note**: OikosBot is in **early development**. Security updates are prioritized for the latest release.

---

## Reporting a Vulnerability
**Do not report vulnerabilities publicly** (e.g., GitHub/GitLab Issues).
Instead, report privately by email to **j.d.a.jewell@open.ac.uk**.

---

### What to Include
Provide **detailed, actionable information**:
- **Type of issue**:
  - Example: Buffer overflow, XSS, supply chain tampering, or **waste metric spoofing**.
- **Affected components**:
  - Source file paths (e.g., `crates/oikosbot-analysis/src/security.rs`).
  - **Commit hash/tag/branch** or direct URL.
- **Reproduction steps**:
  - Command-line invocations, config snippets, or **Justfile recipes** used.
- **Impact**:
  - How could an attacker exploit this? (e.g., "Fake carbon savings reports," "CI/CD pipeline hijacking.")
- **Proof-of-Concept**:
  - Code snippets or **SHAKE256 hashes** of malicious inputs (if applicable).

---

## Response Timeline
| Phase               | Target          | Notes                                  |
|---------------------|-----------------|----------------------------------------|
| **Initial Response** | ≤48 hours       | Acknowledges receipt.                  |
| **Status Update**   | ≤7 days         | Progress or mitigation advice.         |
| **Resolution**      | ≤30 days        | For **critical issues** (e.g., RCE, data leaks). |
| **Attestation**     | Post-resolution | Logs signed with **Ed448** in `logs/`. |

---

## Security Considerations

### Data Handling
OikosBot processes:
- **Source code** (for waste analysis).
- **Dependency graphs** (economic/ecological impact).
- **CI/CD configurations** (e.g., GitLab pipelines).
- **VoID/Dublin Core metadata** (interoperability with WordPress/Drupal).

**Guiding Principles**:
- **Minimal Retention**: Data deleted post-audit unless **explicitly logged for reversibility**.
- **Hashing**: All logs use **SHAKE256/Ed448** (see `logs/README.md`).

### Integration Security
- **Environment Variables**:
  ```bash
  # Example: .env
  OIKOSBOT_API_KEY="x"  # Never commit this!
  GITHUB_TOKEN="y"      # Use GitLab CI variables or encrypted secrets.
