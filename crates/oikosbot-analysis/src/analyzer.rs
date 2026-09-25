// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Core analysis engine using tree-sitter AST

use crate::calibration::{estimate_operation, operation_for_pattern, OperationKind};
use crate::carbon::estimate_carbon;
use crate::language::Language;
use crate::patterns::{detect_patterns, PatternMatch};
use anyhow::{Context, Result};
use oikosbot_metrics::*;
use std::fs;
use std::path::Path;
use tree_sitter::{Parser, Tree};

pub struct Analyzer {
    _language: Language,
    parser: Parser,
    file_path: Option<String>,
}

impl Analyzer {
    pub fn new(language: Language) -> Result<Self> {
        let mut parser = Parser::new();
        parser
            .set_language(&language.parser())
            .context("Failed to set parser language")?;

        Ok(Analyzer {
            _language: language,
            parser,
            file_path: None,
        })
    }

    pub fn analyze_file(&mut self, path: &Path) -> Result<Vec<AnalysisResult>> {
        let source = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        self.file_path = Some(path.display().to_string());
        let results = self.analyze_source(&source);
        self.file_path = None;
        results
    }

    pub fn analyze_source(&mut self, source: &str) -> Result<Vec<AnalysisResult>> {
        let tree = self
            .parser
            .parse(source, None)
            .context("Failed to parse source")?;

        self.analyze_tree(source, &tree)
    }

    fn analyze_tree(&self, source: &str, tree: &Tree) -> Result<Vec<AnalysisResult>> {
        let mut results = Vec::new();

        // Walk the AST and analyze each function
        let root = tree.root_node();
        let mut cursor = root.walk();

        self.visit_node(source, &root, &mut cursor, &mut results);

        Ok(results)
    }

    fn visit_node(
        &self,
        source: &str,
        node: &tree_sitter::Node,
        cursor: &mut tree_sitter::TreeCursor,
        results: &mut Vec<AnalysisResult>,
    ) {
        // Analyze functions
        if self.is_function_node(node) {
            if let Some(result) = self.analyze_function(source, node) {
                results.push(result);
            }
        }

        // Recurse to children
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                self.visit_node(source, &child, cursor, results);

                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }
    }

    fn is_function_node(&self, node: &tree_sitter::Node) -> bool {
        matches!(
            node.kind(),
            "function_item"          // Rust
            | "function_declaration" // JS
            | "arrow_function"       // JS
            | "method_declaration"   // JS
            | "function_definition" // Python
        )
    }

    fn analyze_function(&self, source: &str, node: &tree_sitter::Node) -> Option<AnalysisResult> {
        let location = self.node_location(source, node)?;

        // Estimate resources based on code patterns
        let complexity = self.estimate_complexity(node);

        // Detect problematic patterns first: they decide which calibration row
        // (if any) the estimate is entitled to use.
        let pattern_matches = detect_patterns(source, node);
        let patterns: Vec<String> = pattern_matches.iter().map(|p| p.name.clone()).collect();
        let recommendations = self.generate_recommendations(&patterns);

        let (resources, confidence, resource_range) =
            self.estimate_resources(&pattern_matches, complexity);

        // Derive rule_id and suggestion from most significant pattern
        let (rule_id, suggestion) = if let Some(pm) = pattern_matches.first() {
            (format!("oikosbot/{}", pm.name), pm.suggestion.clone())
        } else {
            ("oikosbot/general".to_string(), None)
        };

        // Calculate scores
        let eco_score = self.calculate_eco_score(&resources);
        let econ_score = self.calculate_econ_score(complexity);
        let quality_score = self.calculate_quality_score(complexity);

        let health = HealthIndex::compute(eco_score, econ_score, quality_score);

        let end = node.end_position();

        Some(AnalysisResult {
            location,
            resources,
            health,
            recommendations,
            rule_id,
            suggestion,
            end_location: Some((end.row + 1, end.column + 1)),
            confidence,
            pareto: None,
            resource_range,
        })
    }

    fn node_location(&self, source: &str, node: &tree_sitter::Node) -> Option<CodeLocation> {
        let start = node.start_position();
        let end = node.end_position();
        let name = self.extract_function_name(source, node);

        Some(CodeLocation {
            file: self
                .file_path
                .clone()
                .unwrap_or_else(|| String::from("<source>")),
            line: start.row + 1,
            column: start.column + 1,
            end_line: Some(end.row + 1),
            end_column: Some(end.column + 1),
            name,
        })
    }

    fn extract_function_name(&self, source: &str, node: &tree_sitter::Node) -> Option<String> {
        // Try to find name node (language-specific)
        let mut cursor = node.walk();
        if cursor.goto_first_child() {
            loop {
                let child = cursor.node();
                if child.kind() == "identifier" {
                    return Some(child.utf8_text(source.as_bytes()).ok()?.to_string());
                }
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
        }
        None
    }

    fn estimate_complexity(&self, node: &tree_sitter::Node) -> usize {
        // Simple complexity: count nodes
        let mut count = 1;
        let mut cursor = node.walk();

        if cursor.goto_first_child() {
            loop {
                count += self.estimate_complexity(&cursor.node());
                if !cursor.goto_next_sibling() {
                    break;
                }
            }
            cursor.goto_parent();
        }

        count
    }

    /// Estimate the resources a unit uses, together with the confidence that
    /// estimate is entitled to and the uncertainty band behind it.
    ///
    /// Two paths, chosen by evidence rather than by preference:
    ///
    /// * **Calibrated path** — the unit carries a detected pattern that maps to
    ///   a known operation category (`calibration::operation_for_pattern`). The
    ///   estimate comes from `calibration::estimate_operation` and the
    ///   confidence is whatever that row earns (Calibrated for measured rows,
    ///   Estimated for host-dependent ones). The min/typical/max band is
    ///   propagated so consumers can see the spread instead of treating the
    ///   point estimate as exact.
    /// * **Naive path** — no recognised pattern, so there is nothing to price.
    ///   The historical `complexity * constant` heuristic is kept, the finding
    ///   is labelled `Estimated`, and no band is claimed.
    ///
    /// When several patterns map, the one with the largest impact multiplier
    /// wins (first one on ties, i.e. detection order): the most severe
    /// recognised cost driver prices the unit. `redundant-allocation` is
    /// intentionally unmapped and therefore never promotes a unit off the naive
    /// path.
    fn estimate_resources(
        &self,
        pattern_matches: &[PatternMatch],
        complexity: usize,
    ) -> (ResourceProfile, Confidence, Option<ResourceRange>) {
        let mut dominant: Option<(OperationKind, f64)> = None;
        for pattern in pattern_matches {
            let Some(kind) = operation_for_pattern(&pattern.name) else {
                continue;
            };
            if dominant.is_none_or(|(_, mult)| pattern.impact_multiplier > mult) {
                dominant = Some((kind, pattern.impact_multiplier));
            }
        }

        match dominant {
            Some((kind, _)) => {
                let range = estimate_operation(kind, complexity);
                // The row that priced the unit also states how much its figure
                // is worth; that confidence travels with the finding.
                (range.typical.clone(), range.confidence, Some(range))
            }
            None => {
                let profile = self.naive_resources(complexity);
                (profile, Confidence::Estimated, None)
            }
        }
    }

    /// The historical heuristic: four axes derived from the AST node count
    /// alone. Kept for units with no recognised pattern — but note that all
    /// four axes are the same linear function of `complexity`, so a Pareto
    /// frontier computed over them is a one-dimensional sort. That is why the
    /// calibrated path exists.
    fn naive_resources(&self, complexity: usize) -> ResourceProfile {
        let energy = Energy::joules(complexity as f64 * 0.1);
        let duration = Duration::milliseconds(complexity as f64 * 0.5);
        let carbon = estimate_carbon(energy);
        let memory = Memory::kilobytes(complexity * 2);

        ResourceProfile {
            energy,
            duration,
            carbon,
            memory,
        }
    }

    fn calculate_eco_score(&self, resources: &ResourceProfile) -> EcoScore {
        // Lower resource usage = higher score
        // Baseline: 100J = 50 score, scale logarithmically
        let energy_score = (100.0 - (resources.energy.0.ln() * 10.0)).max(0.0);
        EcoScore::new(energy_score)
    }

    /// Provisional economic score from complexity alone — the technical-debt
    /// proxy term. The full ARCHITECTURE.adoc composition
    /// (EconScore = 0.5·Pareto + 0.3·Allocation + 0.2·Debt) needs the whole
    /// result set and is applied by `oikosbot_pareto::apply_to_results`,
    /// which folds this value in as DebtScore.
    fn calculate_econ_score(&self, complexity: usize) -> EconScore {
        // Lower complexity = higher efficiency
        let score = (100.0 - (complexity as f64 * 0.5)).max(0.0);
        EconScore::new(score)
    }

    fn calculate_quality_score(&self, complexity: usize) -> f64 {
        // Simple quality metric based on complexity
        (100.0 - (complexity as f64 * 0.3)).max(0.0)
    }

    fn generate_recommendations(&self, patterns: &[String]) -> Vec<String> {
        let mut recs = Vec::new();

        for pattern in patterns {
            match pattern.as_str() {
                "busy-wait" => recs
                    .push("Replace busy-wait loop with async/await or blocking sleep".to_string()),
                "nested-loops" => recs.push(
                    "Consider algorithm optimization to reduce nested iterations".to_string(),
                ),
                "large-allocation" => recs
                    .push("Review memory allocation - consider streaming or chunking".to_string()),
                "string-concat-in-loop" => recs.push(
                    "Use String::with_capacity + push_str or collect with iterators".to_string(),
                ),
                "clone-in-loop" => {
                    recs.push("Consider borrowing instead of cloning inside loop body".to_string())
                }
                "unbuffered-io" => recs.push(
                    "Wrap File with BufReader/BufWriter to reduce syscall overhead".to_string(),
                ),
                "redundant-allocation" => {
                    recs.push("Accept &str instead of String where borrow suffices".to_string())
                }
                _ => {}
            }
        }

        if recs.is_empty() {
            recs.push("Code looks efficient - keep up the good work!".to_string());
        }

        recs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn analyze(source: &str) -> Vec<AnalysisResult> {
        let mut analyzer = Analyzer::new(Language::Rust).expect("analyzer builds");
        analyzer.analyze_source(source).expect("analysis succeeds")
    }

    /// `depth` nested `for` loops — the smallest nest flagged as
    /// `nested-loops` is depth 3. The body avoids every other pattern (no
    /// `.clone()`, no `File::open`, no `vec![]`, no string `+`, no
    /// `to_string`) so exactly one calibrated row prices the unit.
    fn nested_loop_work(depth: usize) -> String {
        let mut body =
            String::from("fn deep_work(items: &[u32]) -> usize {\n    let mut total = 0;\n");
        for _ in 0..depth {
            body.push_str("    for i in 0..8 {\n");
        }
        body.push_str("        total += 1;\n");
        for _ in 0..depth {
            body.push_str("    }\n");
        }
        body.push_str("    total\n}\n");
        body
    }

    /// Straight-line code with no recognised pattern.
    fn plain_work(statements: usize) -> String {
        let mut body = String::from("fn plain_work() -> usize {\n    let mut total = 0;\n");
        for i in 0..statements {
            body.push_str(&format!("    total += {};\n", i + 1));
        }
        body.push_str("    total\n}\n");
        body
    }

    /// Falsifier for the old behaviour: a recognised pattern used to be
    /// labelled `Estimated` like everything else, so no finding could ever be
    /// `Calibrated` and `--check` could never block.
    #[test]
    fn recognized_pattern_earns_calibrated_confidence_and_a_band() {
        let results = analyze(&nested_loop_work(3));
        assert_eq!(results.len(), 1, "one function, one finding");

        let result = &results[0];
        assert_eq!(result.confidence, Confidence::Calibrated);
        assert_eq!(result.rule_id, "oikosbot/nested-loops");

        // The band is propagated, not collapsed: min <= typical <= max, and the
        // point estimate IS the typical bound.
        let range = result
            .resource_range
            .as_ref()
            .expect("a calibrated estimate must carry its band");
        assert!(range.min.energy.0 <= range.typical.energy.0);
        assert!(range.typical.energy.0 <= range.max.energy.0);
        assert!((range.typical.energy.0 - result.resources.energy.0).abs() < 1e-12);
        assert!(range.max.memory.0 >= range.typical.memory.0);
    }

    /// Unrecognised code keeps the naive path and is honest about it: no band is
    /// claimed where none was computed.
    #[test]
    fn unrecognized_code_stays_estimated_with_no_band() {
        let results = analyze(&plain_work(4));
        assert_eq!(results.len(), 1);

        let result = &results[0];
        assert_eq!(result.confidence, Confidence::Estimated);
        assert_eq!(result.rule_id, "oikosbot/general");
        assert!(result.resource_range.is_none());

        // The naive path is unchanged: a positive, complexity-derived figure
        // (0.1 J per AST node) with no band around it.
        assert!(result.resources.energy.0 > 0.0);
    }

    /// The calibrated path must actually change the numbers, or the wiring
    /// would be decoration. Same unit, same size, different evidence.
    #[test]
    fn calibration_changes_the_estimate() {
        let plain = analyze(&plain_work(40));
        let nested = analyze(&nested_loop_work(3));

        // The naive path charges 0.1 J per node; the calibrated Sort row
        // charges microjoules per comparison-bounded operation. A nested-loop
        // unit is no longer free just because it is small, and plain code is
        // no longer expensive just because it is long.
        assert!(
            plain[0].resources.energy.0 > nested[0].resources.energy.0,
            "naive energy {:.4} J should dwarf calibrated {:.6} J",
            plain[0].resources.energy.0,
            nested[0].resources.energy.0
        );
    }
}
