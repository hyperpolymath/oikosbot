// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Reserved security-sustainability integration boundary.
//!
//! The upstream panic-attacker dependency was removed when this workspace
//! became standalone. Enabling the retained feature must compile, but cannot
//! pretend a scan happened. Restore the adapter only with pinned dependencies,
//! boundary tests, provenance and a reviewed result protocol.

use anyhow::Result;
use oikosbot_metrics::AnalysisResult;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct SecurityCorrelation {
    pub eco_results: Vec<AnalysisResult>,
    pub security_findings: Vec<AnalysisResult>,
    pub composite_score: f64,
}

pub fn correlate(
    _repo_path: &Path,
    _eco_results: &[AnalysisResult],
) -> Result<SecurityCorrelation> {
    anyhow::bail!("security correlation unavailable: panic-attacker adapter is not integrated")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unintegrated_scanner_never_returns_an_empty_success() {
        let error = correlate(Path::new("."), &[]).unwrap_err();
        assert!(error.to_string().contains("unavailable"));
    }
}
