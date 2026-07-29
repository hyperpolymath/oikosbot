// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell

//! # OikosBot Pareto
//!
//! Multi-objective Pareto-optimality engine — the economic core of OikosBot.
//!
//! This crate is the executable counterpart of two prior artefacts:
//!
//! * `analyzers/code-haskell/src/Eco/Pareto.hs` — the reference scaffold.
//!   Ported here with corrections: objectives are min-max **normalized before
//!   any distance is measured** (raw gCO2e/joules/bytes are not commensurable),
//!   objective **weights participate in the metric** (they were declared but
//!   unused), all functions are **total** (no `head`-style panics on empty
//!   input), and dominance is **ε-tolerant** so estimate noise cannot
//!   manufacture a "strictly better" objective.
//! * `policy-engine/datalog/eco_rules.dl` — the declarative dominance spec.
//!   `dominates` here implements the same semantics (at ε = 0): at least as
//!   good on every objective, strictly better on at least one.
//!
//! Two products are built on the core:
//!
//! * **PR verdicts** ([`compare`], [`assess`]): base-vs-head objective vectors
//!   classify a change as a Pareto improvement, a Pareto regression, a
//!   trade-off, or neutral. Per the OikosBot doctrine — "don't improve one
//!   axis at another's expense without saying so" — trade-offs and regressions
//!   ask for documentation ([`tradeoff_documented`]); enforcement stays
//!   advisory unless a repo opts into regulator mode.
//! * **Intra-repo frontier** ([`pareto_scores`], [`refactor_candidates`]):
//!   code units are points in objective space; distance from the frontier
//!   yields ParetoScore (0–100) and dominated-by counts rank refactor
//!   candidates (mirrors `needs_refactor` in `eco_rules.dl`).

#![forbid(unsafe_code)]

use oikosbot_metrics::{
    AnalysisResult, Confidence, EconScore, HealthIndex, ParetoInfo, ShadowPrices,
};
use serde::{Deserialize, Serialize};

/// Default tolerance below which two objective values are considered equal.
///
/// OikosBot's resource metrics are heuristic estimates; differences smaller
/// than this cannot honestly be called an improvement or a regression.
pub const DEFAULT_EPSILON: f64 = 1e-6;

/// Direction of optimization for an objective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
    /// Lower is better (e.g. carbon, energy, latency, memory).
    Minimize,
    /// Higher is better (e.g. maintainability, test coverage).
    Maximize,
}

/// One optimization objective: a named axis with a direction and a weight.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Objective {
    pub name: String,
    pub direction: Direction,
    /// Relative importance (weights are normalized to sum 1 internally,
    /// so only ratios matter).
    pub weight: f64,
}

impl Objective {
    pub fn new(name: &str, direction: Direction, weight: f64) -> Self {
        Objective {
            name: name.to_string(),
            direction,
            weight,
        }
    }
}

/// The seven standard OikosBot objectives, as specified in
/// `Eco.Pareto.standardObjectives` (ARCHITECTURE.adoc).
pub fn standard_objectives() -> Vec<Objective> {
    vec![
        Objective::new("carbon_intensity", Direction::Minimize, 0.20),
        Objective::new("energy_consumption", Direction::Minimize, 0.15),
        Objective::new("execution_time", Direction::Minimize, 0.15),
        Objective::new("memory_usage", Direction::Minimize, 0.10),
        Objective::new("maintainability", Direction::Maximize, 0.15),
        Objective::new("test_coverage", Direction::Maximize, 0.10),
        Objective::new("technical_debt", Direction::Minimize, 0.15),
    ]
}

/// The objective set used when treating per-unit [`AnalysisResult`]s as points
/// in objective space. Only axes the Rust analyzer actually measures today:
/// the four `ResourceProfile` axes (minimize) plus the quality score
/// (maximize, as the maintainability proxy).
pub fn result_objectives() -> Vec<Objective> {
    vec![
        Objective::new("carbon_intensity", Direction::Minimize, 0.30),
        Objective::new("energy_consumption", Direction::Minimize, 0.20),
        Objective::new("execution_time", Direction::Minimize, 0.15),
        Objective::new("memory_usage", Direction::Minimize, 0.10),
        Objective::new("maintainability", Direction::Maximize, 0.25),
    ]
}

/// Signed improvement of `a` over `b` on one objective; positive means
/// `a` is better.
fn improvement(direction: Direction, a: f64, b: f64) -> f64 {
    match direction {
        Direction::Minimize => b - a,
        Direction::Maximize => a - b,
    }
}

/// Does point `a` Pareto-dominate point `b`?
///
/// True iff `a` is at least as good as `b` on every objective (within `eps`)
/// and strictly better (by more than `eps`) on at least one. Mismatched
/// vector lengths never dominate (total, no panic).
pub fn dominates(objectives: &[Objective], a: &[f64], b: &[f64], eps: f64) -> bool {
    if a.len() != objectives.len() || b.len() != objectives.len() {
        return false;
    }
    let mut any_strict = false;
    for (i, obj) in objectives.iter().enumerate() {
        let imp = improvement(obj.direction, a[i], b[i]);
        if imp < -eps {
            return false; // a is worse on this objective
        }
        if imp > eps {
            any_strict = true;
        }
    }
    any_strict
}

/// Indices of the non-dominated points (the Pareto frontier).
///
/// Empty input yields an empty frontier; a single point is trivially optimal.
pub fn frontier_indices(objectives: &[Objective], points: &[Vec<f64>], eps: f64) -> Vec<usize> {
    (0..points.len())
        .filter(|&i| {
            !points
                .iter()
                .enumerate()
                .any(|(j, p)| j != i && dominates(objectives, p, &points[i], eps))
        })
        .collect()
}

/// How many other points dominate each point.
pub fn dominated_by_counts(objectives: &[Objective], points: &[Vec<f64>], eps: f64) -> Vec<usize> {
    (0..points.len())
        .map(|i| {
            points
                .iter()
                .enumerate()
                .filter(|(j, p)| *j != i && dominates(objectives, p, &points[i], eps))
                .count()
        })
        .collect()
}

/// Min-max normalize each objective across the point set, oriented so that
/// 1.0 is always best and 0.0 always worst regardless of direction.
///
/// An objective with no spread across the set carries no ranking information
/// and maps to 0.5 everywhere.
pub fn normalize(objectives: &[Objective], points: &[Vec<f64>]) -> Vec<Vec<f64>> {
    if points.is_empty() {
        return Vec::new();
    }
    let n_obj = objectives.len();
    let mut mins = vec![f64::INFINITY; n_obj];
    let mut maxs = vec![f64::NEG_INFINITY; n_obj];
    for p in points {
        for i in 0..n_obj.min(p.len()) {
            mins[i] = mins[i].min(p[i]);
            maxs[i] = maxs[i].max(p[i]);
        }
    }
    points
        .iter()
        .map(|p| {
            (0..n_obj)
                .map(|i| {
                    let v = p.get(i).copied().unwrap_or(mins[i]);
                    let range = maxs[i] - mins[i];
                    if range <= f64::EPSILON {
                        0.5
                    } else {
                        match objectives[i].direction {
                            Direction::Minimize => (maxs[i] - v) / range,
                            Direction::Maximize => (v - mins[i]) / range,
                        }
                    }
                })
                .collect()
        })
        .collect()
}

/// Weights normalized to sum 1 (uniform if all weights are zero/invalid).
fn normalized_weights(objectives: &[Objective]) -> Vec<f64> {
    let sum: f64 = objectives.iter().map(|o| o.weight.max(0.0)).sum();
    if sum <= f64::EPSILON {
        let n = objectives.len().max(1);
        return vec![1.0 / n as f64; objectives.len()];
    }
    objectives.iter().map(|o| o.weight.max(0.0) / sum).collect()
}

/// Weighted Euclidean distance in normalized space. With weights summing to 1
/// and coordinates in [0,1], the distance is bounded by 1.
fn weighted_distance(weights: &[f64], a: &[f64], b: &[f64]) -> f64 {
    weights
        .iter()
        .zip(a.iter().zip(b.iter()))
        .map(|(w, (x, y))| w * (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

/// ParetoScore for every point: 100 for frontier members, otherwise
/// `100·(1 − d)` where `d` is the weighted distance (in normalized space)
/// to the nearest frontier point.
///
/// This is the "distance from Pareto frontier" term of
/// `EconScore = w1·ParetoScore + w2·AllocationScore + w3·DebtScore`
/// (ARCHITECTURE.adoc, Metrics & Scoring).
pub fn pareto_scores(objectives: &[Objective], points: &[Vec<f64>], eps: f64) -> Vec<f64> {
    if points.is_empty() {
        return Vec::new();
    }
    let frontier = frontier_indices(objectives, points, eps);
    let normalized = normalize(objectives, points);
    let weights = normalized_weights(objectives);
    (0..points.len())
        .map(|i| {
            if frontier.contains(&i) {
                return 100.0;
            }
            let d = frontier
                .iter()
                .map(|&f| weighted_distance(&weights, &normalized[i], &normalized[f]))
                .fold(f64::INFINITY, f64::min);
            if d.is_finite() {
                (100.0 * (1.0 - d)).clamp(0.0, 100.0)
            } else {
                // No frontier can only happen on empty input, handled above;
                // stay total anyway.
                100.0
            }
        })
        .collect()
}

/// Indices of dominated points ranked as refactor candidates: most-dominated
/// first (ties broken by index for determinism). Non-dominated points are
/// not candidates. Mirrors `needs_refactor(_, "pareto_optimization", _)` in
/// `eco_rules.dl`.
pub fn refactor_candidates(objectives: &[Objective], points: &[Vec<f64>], eps: f64) -> Vec<usize> {
    let counts = dominated_by_counts(objectives, points, eps);
    let mut candidates: Vec<usize> = (0..points.len()).filter(|&i| counts[i] > 0).collect();
    candidates.sort_by(|&a, &b| counts[b].cmp(&counts[a]).then(a.cmp(&b)));
    candidates
}

/// For a dominated point, name the objectives where some dominating point is
/// strictly better, with the achievable value. Empty for frontier members.
pub fn suggest_improvements(
    objectives: &[Objective],
    point: &[f64],
    all_points: &[Vec<f64>],
    eps: f64,
) -> Vec<String> {
    let dominators: Vec<&Vec<f64>> = all_points
        .iter()
        .filter(|p| dominates(objectives, p, point, eps))
        .collect();
    if dominators.is_empty() {
        return Vec::new();
    }
    objectives
        .iter()
        .enumerate()
        .filter_map(|(i, obj)| {
            let best = dominators
                .iter()
                .map(|p| p[i])
                .fold(dominators[0][i], |acc, v| match obj.direction {
                    Direction::Minimize => acc.min(v),
                    Direction::Maximize => acc.max(v),
                });
            let imp = improvement(obj.direction, best, point[i]);
            if imp > eps {
                Some(format!(
                    "{}: a dominating alternative achieves {:.4} (current {:.4})",
                    obj.name, best, point[i]
                ))
            } else {
                None
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------
// PR verdicts (base vs head)
// ---------------------------------------------------------------------------

/// Classification of a change in objective space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ParetoVerdict {
    /// Head dominates base: better on ≥1 objective, worse on none.
    Improvement,
    /// Base dominates head: worse on ≥1 objective, better on none.
    Regression,
    /// Better on some objectives, worse on others — must be documented.
    TradeOff,
    /// No objective moved by more than ε.
    Neutral,
}

/// Per-objective delta between base and head.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectiveDelta {
    pub name: String,
    pub base: f64,
    pub head: f64,
    /// Signed improvement of head over base (positive = better).
    pub improvement: f64,
}

/// Full base-vs-head assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comparison {
    pub verdict: ParetoVerdict,
    pub deltas: Vec<ObjectiveDelta>,
    /// Objectives that actually moved (|improvement| > ε) — the verdict's
    /// drivers.
    pub drivers: Vec<String>,
    /// Whether the verdict may drive a blocking decision: every driving
    /// objective must be backed by `Measured` or `Calibrated` inputs.
    /// Heuristic estimates advise; they do not block.
    pub actionable: bool,
}

/// Classify head vs base (see [`ParetoVerdict`]).
pub fn compare(objectives: &[Objective], base: &[f64], head: &[f64], eps: f64) -> ParetoVerdict {
    if dominates(objectives, head, base, eps) {
        ParetoVerdict::Improvement
    } else if dominates(objectives, base, head, eps) {
        ParetoVerdict::Regression
    } else {
        let any_moved = objectives
            .iter()
            .enumerate()
            .any(|(i, obj)| improvement(obj.direction, head[i], base[i]).abs() > eps);
        if any_moved {
            ParetoVerdict::TradeOff
        } else {
            ParetoVerdict::Neutral
        }
    }
}

fn confidence_rank(c: Confidence) -> u8 {
    match c {
        Confidence::Measured => 3,
        Confidence::Calibrated => 2,
        Confidence::Estimated => 1,
        Confidence::Unknown => 0,
    }
}

/// Assess head vs base with per-objective confidence gating.
///
/// `confidences[i]` is the confidence of objective `i`'s measurement (pass a
/// uniform slice if only an aggregate is known). Lengths are reconciled
/// leniently: missing confidences default to `Unknown`.
pub fn assess(
    objectives: &[Objective],
    base: &[f64],
    head: &[f64],
    confidences: &[Confidence],
    eps: f64,
) -> Comparison {
    let verdict = compare(objectives, base, head, eps);
    let deltas: Vec<ObjectiveDelta> = objectives
        .iter()
        .enumerate()
        .map(|(i, obj)| ObjectiveDelta {
            name: obj.name.clone(),
            base: base.get(i).copied().unwrap_or(f64::NAN),
            head: head.get(i).copied().unwrap_or(f64::NAN),
            improvement: improvement(
                obj.direction,
                head.get(i).copied().unwrap_or(f64::NAN),
                base.get(i).copied().unwrap_or(f64::NAN),
            ),
        })
        .collect();
    let drivers: Vec<String> = deltas
        .iter()
        .filter(|d| d.improvement.abs() > eps)
        .map(|d| d.name.clone())
        .collect();
    let actionable = !drivers.is_empty()
        && deltas.iter().enumerate().all(|(i, d)| {
            if d.improvement.abs() > eps {
                let c = confidences.get(i).copied().unwrap_or(Confidence::Unknown);
                confidence_rank(c) >= confidence_rank(Confidence::Calibrated)
            } else {
                true
            }
        });
    Comparison {
        verdict,
        deltas,
        drivers,
        actionable,
    }
}

/// Does a PR body (or commit message) document its Pareto trade-off?
///
/// Accepted markers, per the trade-off-documentation doctrine:
/// a `Pareto-Trade-off:` trailer line, or a Markdown/AsciiDoc heading
/// containing "pareto trade-off".
pub fn tradeoff_documented(text: &str) -> bool {
    text.lines().any(|line| {
        let t = line.trim().to_ascii_lowercase();
        t.starts_with("pareto-trade-off:")
            || ((t.starts_with('#') || t.starts_with('=')) && t.contains("pareto trade-off"))
    })
}

// ---------------------------------------------------------------------------
// Wiring into AnalysisResult collections
// ---------------------------------------------------------------------------

/// Extract the [`result_objectives`] vector from one analysis result.
pub fn result_point(result: &AnalysisResult) -> Vec<f64> {
    vec![
        result.resources.carbon.0,
        result.resources.energy.0,
        result.resources.duration.0,
        result.resources.memory.0 as f64,
        result.health.quality_score,
    ]
}

/// Intra-repo Pareto pass over a set of analysis results.
///
/// Treats every result as a point in [`result_objectives`] space, computes
/// frontier membership, ParetoScore, and dominated-by counts, stores them in
/// each result's `pareto` field, and replaces the provisional (complexity
/// only) EconScore with the ARCHITECTURE.adoc composition:
///
/// `EconScore = 0.5·ParetoScore + 0.3·AllocationScore + 0.2·DebtScore`
///
/// * AllocationScore: shadow-price cost ([`ShadowPrices`]) min-max inverted
///   across the set — allocative efficiency relative to peers.
/// * DebtScore: the analyzer's provisional complexity-based EconScore,
///   reinterpreted as the technical-debt proxy it actually is.
///
/// The overall HealthIndex is recomputed from the new EconScore.
pub fn apply_to_results(results: &mut [AnalysisResult], eps: f64) {
    if results.is_empty() {
        return;
    }
    let objectives = result_objectives();
    let points: Vec<Vec<f64>> = results.iter().map(result_point).collect();
    let scores = pareto_scores(&objectives, &points, eps);
    let counts = dominated_by_counts(&objectives, &points, eps);
    let frontier = frontier_indices(&objectives, &points, eps);

    let prices = ShadowPrices::default();
    let costs: Vec<f64> = results.iter().map(|r| r.resources.cost(&prices)).collect();
    let (min_cost, max_cost) = costs
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &c| {
            (lo.min(c), hi.max(c))
        });
    let cost_range = max_cost - min_cost;

    for (i, result) in results.iter_mut().enumerate() {
        let allocation_score = if cost_range <= f64::EPSILON {
            50.0
        } else {
            100.0 * (max_cost - costs[i]) / cost_range
        };
        let debt_score = result.health.econ_score.0;
        let econ = EconScore::new(0.5 * scores[i] + 0.3 * allocation_score + 0.2 * debt_score);
        result.health =
            HealthIndex::compute(result.health.eco_score, econ, result.health.quality_score);
        result.pareto = Some(ParetoInfo {
            status: if frontier.contains(&i) {
                "optimal".to_string()
            } else {
                "dominated".to_string()
            },
            score: scores[i],
            dominated_by: counts[i],
        });
    }
}

/// Aggregate a result set into one repo-level objective vector
/// (sums for resources, mean for quality). Returns `None` for empty input.
pub fn aggregate_point(results: &[AnalysisResult]) -> Option<Vec<f64>> {
    if results.is_empty() {
        return None;
    }
    let n = results.len() as f64;
    let sum = |f: &dyn Fn(&AnalysisResult) -> f64| results.iter().map(f).sum::<f64>();
    Some(vec![
        sum(&|r| r.resources.carbon.0),
        sum(&|r| r.resources.energy.0),
        sum(&|r| r.resources.duration.0),
        sum(&|r| r.resources.memory.0 as f64),
        sum(&|r| r.health.quality_score) / n,
    ])
}

/// The weaker of two confidence levels.
pub fn weaker_confidence(a: Confidence, b: Confidence) -> Confidence {
    if confidence_rank(a) <= confidence_rank(b) {
        a
    } else {
        b
    }
}

/// Weakest measurement confidence in a result set (`Unknown` for empty input).
pub fn aggregate_confidence(results: &[AnalysisResult]) -> Confidence {
    results
        .iter()
        .map(|r| r.confidence)
        .min_by_key(|c| confidence_rank(*c))
        .unwrap_or(Confidence::Unknown)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn objs2() -> Vec<Objective> {
        vec![
            Objective::new("carbon", Direction::Minimize, 0.5),
            Objective::new("quality", Direction::Maximize, 0.5),
        ]
    }

    /// Deterministic xorshift for randomized property checks (no external
    /// dependency, reproducible by construction).
    struct XorShift(u64);
    impl XorShift {
        fn next_f64(&mut self) -> f64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            (x % 10_000) as f64 / 100.0
        }
        fn point(&mut self, dims: usize) -> Vec<f64> {
            (0..dims).map(|_| self.next_f64()).collect()
        }
    }

    #[test]
    fn dominance_matches_datalog_semantics() {
        // eco_rules.dl: A dominates B iff >= on all metrics, > on at least one.
        let o = objs2();
        assert!(dominates(&o, &[1.0, 9.0], &[2.0, 9.0], 0.0)); // less carbon, equal quality
        assert!(dominates(&o, &[1.0, 9.0], &[1.0, 8.0], 0.0)); // equal carbon, more quality
        assert!(!dominates(&o, &[1.0, 9.0], &[1.0, 9.0], 0.0)); // equal points: no strict edge
        assert!(!dominates(&o, &[1.0, 8.0], &[2.0, 9.0], 0.0)); // trade-off: incomparable
    }

    #[test]
    fn dominance_is_irreflexive_and_antisymmetric() {
        let o = objs2();
        let mut rng = XorShift(0x00D1CE5EED);
        for _ in 0..200 {
            let a = rng.point(2);
            let b = rng.point(2);
            assert!(!dominates(&o, &a, &a, DEFAULT_EPSILON), "irreflexive");
            assert!(
                !(dominates(&o, &a, &b, DEFAULT_EPSILON) && dominates(&o, &b, &a, DEFAULT_EPSILON)),
                "antisymmetric"
            );
        }
    }

    #[test]
    fn epsilon_swallows_noise() {
        let o = objs2();
        // A nanoscale carbon "win" is not a strict improvement at ε=1e-6.
        assert!(!dominates(
            &o,
            &[1.0 - 1e-9, 5.0],
            &[1.0, 5.0],
            DEFAULT_EPSILON
        ));
        assert_eq!(
            compare(&o, &[1.0, 5.0], &[1.0 - 1e-9, 5.0], DEFAULT_EPSILON),
            ParetoVerdict::Neutral
        );
    }

    #[test]
    fn frontier_contains_no_dominated_point() {
        let o = objs2();
        let mut rng = XorShift(0xFACADE);
        let points: Vec<Vec<f64>> = (0..60).map(|_| rng.point(2)).collect();
        let frontier = frontier_indices(&o, &points, DEFAULT_EPSILON);
        assert!(!frontier.is_empty(), "non-empty input has a frontier");
        for &i in &frontier {
            for (j, p) in points.iter().enumerate() {
                if j != i {
                    assert!(
                        !dominates(&o, p, &points[i], DEFAULT_EPSILON),
                        "frontier point {i} dominated by {j}"
                    );
                }
            }
        }
    }

    #[test]
    fn totality_on_degenerate_inputs() {
        let o = objs2();
        // Empty set: no panic (the Haskell reference crashed via `head`).
        assert!(frontier_indices(&o, &[], DEFAULT_EPSILON).is_empty());
        assert!(pareto_scores(&o, &[], DEFAULT_EPSILON).is_empty());
        // Single point: trivially optimal, score 100.
        let one = vec![vec![3.0, 4.0]];
        assert_eq!(frontier_indices(&o, &one, DEFAULT_EPSILON), vec![0]);
        assert_eq!(pareto_scores(&o, &one, DEFAULT_EPSILON), vec![100.0]);
        // Mismatched dimensionality never dominates.
        assert!(!dominates(&o, &[1.0], &[2.0, 3.0], DEFAULT_EPSILON));
    }

    #[test]
    fn scores_are_bounded_and_frontier_scores_100() {
        let o = objs2();
        let mut rng = XorShift(0xBEEFCAFE);
        let points: Vec<Vec<f64>> = (0..80).map(|_| rng.point(2)).collect();
        let frontier = frontier_indices(&o, &points, DEFAULT_EPSILON);
        let scores = pareto_scores(&o, &points, DEFAULT_EPSILON);
        for (i, s) in scores.iter().enumerate() {
            assert!((0.0..=100.0).contains(s), "score {s} out of bounds");
            if frontier.contains(&i) {
                assert_eq!(*s, 100.0);
            } else {
                assert!(*s < 100.0, "dominated point scored 100");
            }
        }
    }

    #[test]
    fn normalization_orients_best_to_one() {
        let o = objs2();
        let points = vec![vec![10.0, 1.0], vec![20.0, 3.0], vec![30.0, 2.0]];
        let norm = normalize(&o, &points);
        // Least carbon (10.0) normalizes to 1.0 on a Minimize axis.
        assert!((norm[0][0] - 1.0).abs() < 1e-12);
        assert!((norm[2][0] - 0.0).abs() < 1e-12);
        // Highest quality (3.0) normalizes to 1.0 on a Maximize axis.
        assert!((norm[1][1] - 1.0).abs() < 1e-12);
        assert!((norm[0][1] - 0.0).abs() < 1e-12);
    }

    #[test]
    fn verdict_symmetry() {
        let o = objs2();
        let mut rng = XorShift(0x5EED);
        for _ in 0..200 {
            let a = rng.point(2);
            let b = rng.point(2);
            let ab = compare(&o, &a, &b, DEFAULT_EPSILON);
            let ba = compare(&o, &b, &a, DEFAULT_EPSILON);
            match ab {
                ParetoVerdict::Improvement => assert_eq!(ba, ParetoVerdict::Regression),
                ParetoVerdict::Regression => assert_eq!(ba, ParetoVerdict::Improvement),
                ParetoVerdict::TradeOff => assert_eq!(ba, ParetoVerdict::TradeOff),
                ParetoVerdict::Neutral => assert_eq!(ba, ParetoVerdict::Neutral),
            }
        }
    }

    #[test]
    fn tradeoff_verdict_on_mixed_movement() {
        let o = objs2();
        // Carbon worsens, quality improves: a trade-off, not a regression.
        assert_eq!(
            compare(&o, &[1.0, 5.0], &[2.0, 9.0], DEFAULT_EPSILON),
            ParetoVerdict::TradeOff
        );
    }

    #[test]
    fn confidence_gates_actionability() {
        let o = objs2();
        let base = [2.0, 5.0];
        let head = [1.0, 5.0]; // carbon improved
        let measured = assess(
            &o,
            &base,
            &head,
            &[Confidence::Measured; 2],
            DEFAULT_EPSILON,
        );
        assert_eq!(measured.verdict, ParetoVerdict::Improvement);
        assert!(measured.actionable);
        assert_eq!(measured.drivers, vec!["carbon".to_string()]);

        let guessed = assess(
            &o,
            &base,
            &head,
            &[Confidence::Estimated; 2],
            DEFAULT_EPSILON,
        );
        assert!(!guessed.actionable, "heuristic estimates must not block");

        // Confidence on a non-driving objective is irrelevant.
        let mixed = assess(
            &o,
            &base,
            &head,
            &[Confidence::Measured, Confidence::Unknown],
            DEFAULT_EPSILON,
        );
        assert!(mixed.actionable);
    }

    #[test]
    fn tradeoff_documentation_markers() {
        assert!(tradeoff_documented(
            "Speeds up hot loop.\n\nPareto-Trade-off: +12% memory for -40% latency; accepted."
        ));
        assert!(tradeoff_documented(
            "## Pareto Trade-off\nMore RAM, less CO2."
        ));
        assert!(tradeoff_documented(
            "== Pareto Trade-off\nAsciiDoc heading."
        ));
        assert!(!tradeoff_documented(
            "Fixes a bug. No trade-offs discussed."
        ));
    }

    #[test]
    fn refactor_candidates_ranked_by_domination() {
        let o = objs2();
        let points = vec![
            vec![1.0, 9.0], // optimal
            vec![2.0, 8.0], // dominated by 0
            vec![3.0, 7.0], // dominated by 0 and 1
        ];
        let candidates = refactor_candidates(&o, &points, DEFAULT_EPSILON);
        assert_eq!(candidates, vec![2, 1]);
        assert!(!suggest_improvements(&o, &points[2], &points, DEFAULT_EPSILON).is_empty());
        assert!(suggest_improvements(&o, &points[0], &points, DEFAULT_EPSILON).is_empty());
    }
}
