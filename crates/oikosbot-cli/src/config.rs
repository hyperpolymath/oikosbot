// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell

//! `.oikos.yml` configuration loading.
//!
//! Parses the per-repo OikosBot configuration carried by consumer
//! repositories (and the full `config/oikos.yaml` reference schema — both
//! dialects share the fields consumed here). Parsing is deliberately lenient:
//! unknown keys are ignored so the full reference config always loads.

use anyhow::{Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::Deserialize;
use std::path::{Path, PathBuf};

/// Operating mode, per config: consultant | advisor | regulator.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    Consultant,
    #[default]
    Advisor,
    Regulator,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawConfig {
    mode: Option<String>,
    thresholds: RawThresholds,
    /// Estate `.oikos.yml` dialect: top-level exclude list.
    exclude: Vec<String>,
    analysis: RawAnalysis,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawThresholds {
    eco_minimum: Option<RawThresholdLevel>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawThresholdLevel {
    carbon: Option<f64>,
    energy: Option<f64>,
    enforcement: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(default)]
struct RawAnalysis {
    languages: Vec<String>,
    /// `config/oikos.yaml` dialect: exclude nested under `analysis`.
    exclude: Vec<String>,
}

/// Configuration resolved into the shapes the CLI acts on.
#[derive(Debug)]
pub struct ResolvedConfig {
    pub mode: Mode,
    /// `thresholds.eco_minimum.carbon` — the eco-score floor.
    pub eco_threshold: Option<f64>,
    /// True when violations should fail the run
    /// (`enforcement: blocking` or regulator mode).
    pub enforcement_blocking: bool,
    pub exclude: GlobSet,
    /// File extensions to analyze, intersected with what the analyzer
    /// actually supports (rs / js / py today).
    pub allowed_extensions: Vec<&'static str>,
    /// Where this config was loaded from (for logging).
    pub source: PathBuf,
}

const DEFAULT_EXTENSIONS: [&str; 3] = ["rs", "js", "py"];

fn language_extension(language: &str) -> Option<&'static str> {
    match language.to_ascii_lowercase().as_str() {
        "rust" => Some("rs"),
        "javascript" => Some("js"),
        "python" => Some("py"),
        _ => None, // listed but unsupported by the tree-sitter analyzer today
    }
}

/// Load and resolve a config file.
pub fn load(path: &Path) -> Result<ResolvedConfig> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("cannot read config {}", path.display()))?;
    let raw: RawConfig = serde_yaml_ng::from_str(&text)
        .with_context(|| format!("cannot parse config {}", path.display()))?;

    let mode = match raw.mode.as_deref() {
        Some("consultant") => Mode::Consultant,
        Some("regulator") => Mode::Regulator,
        Some("advisor") | None => Mode::Advisor,
        Some(other) => {
            tracing::warn!(
                "unknown mode {other:?} in {}; using advisor",
                path.display()
            );
            Mode::Advisor
        }
    };

    let level = raw.thresholds.eco_minimum.unwrap_or_default();
    let enforcement_blocking =
        mode == Mode::Regulator || level.enforcement.as_deref() == Some("blocking");
    // The eco floor: carbon threshold, falling back to energy if only that
    // is set (both express the same 0-100 eco-score floor today).
    let eco_threshold = level.carbon.or(level.energy);

    let mut globs = GlobSetBuilder::new();
    for pattern in raw.exclude.iter().chain(raw.analysis.exclude.iter()) {
        match Glob::new(pattern) {
            Ok(g) => {
                globs.add(g);
            }
            Err(e) => tracing::warn!("ignoring invalid exclude glob {pattern:?}: {e}"),
        }
    }
    let exclude = globs.build().context("cannot build exclude globs")?;

    let allowed_extensions: Vec<&'static str> = if raw.analysis.languages.is_empty() {
        DEFAULT_EXTENSIONS.to_vec()
    } else {
        let exts: Vec<&'static str> = raw
            .analysis
            .languages
            .iter()
            .filter_map(|l| language_extension(l))
            .collect();
        if exts.is_empty() {
            // Config names only languages the analyzer cannot parse yet;
            // analyzing nothing would be a silent no-scan. Fall back loudly.
            tracing::warn!(
                "no configured language is supported by the analyzer; \
                 falling back to {:?}",
                DEFAULT_EXTENSIONS
            );
            DEFAULT_EXTENSIONS.to_vec()
        } else {
            exts
        }
    };

    Ok(ResolvedConfig {
        mode,
        eco_threshold,
        enforcement_blocking,
        exclude,
        allowed_extensions,
        source: path.to_path_buf(),
    })
}

/// Find a repo-local config next to the analyzed directory:
/// `.oikos.yml` or `.oikos.yaml`.
pub fn discover(dir: &Path) -> Option<PathBuf> {
    [".oikos.yml", ".oikos.yaml"]
        .iter()
        .map(|name| dir.join(name))
        .find(|p| p.is_file())
}

/// Resolve the effective config: an explicit `--config` path wins; otherwise
/// discover a repo-local `.oikos.yml`; otherwise `None` (built-in defaults).
pub fn resolve(explicit: Option<&Path>, target_dir: &Path) -> Result<Option<ResolvedConfig>> {
    match explicit {
        Some(p) => load(p).map(Some),
        None => match discover(target_dir) {
            Some(p) => {
                tracing::info!("using repo config {}", p.display());
                load(&p).map(Some)
            }
            None => Ok(None),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_config(content: &str) -> tempfile::NamedTempFile {
        let mut f = tempfile::NamedTempFile::new().unwrap();
        f.write_all(content.as_bytes()).unwrap();
        f
    }

    #[test]
    fn parses_estate_dialect() {
        // Shape carried by the 17+ estate .oikos.yml files (e.g. paint-type).
        let f = write_config(
            r#"
mode: advisor
thresholds:
  eco_minimum:
    carbon: 50
    energy: 50
    enforcement: warning
exclude:
  - "**/node_modules/**"
  - "**/target/**"
analysis:
  languages:
    - rust
    - typescript
    - python
"#,
        );
        let c = load(f.path()).unwrap();
        assert_eq!(c.mode, Mode::Advisor);
        assert_eq!(c.eco_threshold, Some(50.0));
        assert!(!c.enforcement_blocking);
        assert!(c.exclude.is_match("crates/foo/target/debug/x.rs"));
        assert!(!c.exclude.is_match("crates/foo/src/lib.rs"));
        // typescript is listed but unsupported; rust + python survive.
        assert_eq!(c.allowed_extensions, vec!["rs", "py"]);
    }

    #[test]
    fn parses_full_reference_config_leniently() {
        // The full config/oikos.yaml has many extra sections; they must not
        // break parsing, and analysis.exclude (nested dialect) must be honored.
        let f = write_config(
            r#"
mode: regulator
thresholds:
  eco_minimum:
    carbon: 50
    energy: 50
    description: "Minimum acceptable eco standards"
    enforcement: blocking
weights:
  ecological: 0.4
analysis:
  languages: [rust]
  exclude:
    - "**/dist/**"
databases:
  verisimdb:
    endpoint: "${VERISIMDB_ENDPOINT}"
"#,
        );
        let c = load(f.path()).unwrap();
        assert_eq!(c.mode, Mode::Regulator);
        assert!(c.enforcement_blocking);
        assert!(c.exclude.is_match("web/dist/bundle.js"));
    }

    #[test]
    fn defaults_without_config_fields() {
        let f = write_config("{}\n");
        let c = load(f.path()).unwrap();
        assert_eq!(c.mode, Mode::Advisor);
        assert_eq!(c.eco_threshold, None);
        assert!(!c.enforcement_blocking);
        assert_eq!(c.allowed_extensions, DEFAULT_EXTENSIONS.to_vec());
    }
}
