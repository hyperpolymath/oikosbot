// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell

//! Experimental evidence-scoped, non-compensatory decision contract.
//!
//! This is a runtime refinement prototype, not a new type calculus or proof
//! of environmental benefit. Private constructors prevent unchecked numeric
//! states; they do NOT authenticate a caller's model, receipts or authority.
//! In particular there is no conversion from this module to permission to
//! merge, auto-fix, spend a budget, or claim a real-world Pareto improvement.

use crate::Direction;
use oikosbot_metrics::Confidence;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Interval {
    lower: f64,
    upper: f64,
}

impl Interval {
    pub fn new(lower: f64, upper: f64) -> Result<Self, &'static str> {
        if !lower.is_finite() || !upper.is_finite() || lower > upper {
            return Err("interval must have ordered finite bounds");
        }
        Ok(Self { lower, upper })
    }
}

/// Axes are dimensioned and ordered explicitly. A limit is a hard ceiling
/// for Minimize or hard floor for Maximize, never a compensable weight.
#[derive(Debug, Clone, PartialEq)]
pub struct Axis {
    pub name: String,
    pub unit: String,
    pub direction: Direction,
    pub tolerance: f64,
    pub limit: Option<f64>,
}

/// Equality is a compatibility requirement, not evidence that the context
/// is faithful to reality. All identifiers should be content-addressed in a
/// future ingestion adapter. No serde deserializer may bypass validation.
#[derive(Debug, Clone, PartialEq)]
pub struct Context {
    pub workload: String,
    pub functional_unit: String,
    pub boundary: String,
    pub horizon: String,
    pub model_revision: String,
    pub policy_revision: String,
    pub axes: Vec<Axis>,
}

impl Context {
    fn validate(&self) -> Result<(), &'static str> {
        if [
            &self.workload,
            &self.functional_unit,
            &self.boundary,
            &self.horizon,
            &self.model_revision,
            &self.policy_revision,
        ]
        .iter()
        .any(|s| s.trim().is_empty())
            || self.axes.is_empty()
        {
            return Err("comparison requires an explicit nonempty context");
        }
        for (i, axis) in self.axes.iter().enumerate() {
            if axis.name.trim().is_empty()
                || axis.unit.trim().is_empty()
                || !axis.tolerance.is_finite()
                || axis.tolerance < 0.0
                || axis.limit.is_some_and(|limit| !limit.is_finite())
                || self.axes[..i].iter().any(|a| a.name == axis.name)
            {
                return Err("invalid or duplicate objective");
            }
        }
        Ok(())
    }
}

/// A declared warrant, NOT an authenticated proof. One per objective; a
/// caller cannot turn an energy receipt into evidence for quality or carbon.
#[derive(Debug, Clone)]
pub struct Evidence {
    pub confidence: Confidence,
    pub source: String,
    pub assumptions: Vec<String>,
}

#[derive(Debug)]
pub struct Observation<'a> {
    context: &'a Context,
    values: Vec<Interval>,
    evidence: Vec<Evidence>,
}

impl<'a> Observation<'a> {
    pub fn new(
        context: &'a Context,
        values: Vec<Interval>,
        evidence: Vec<Evidence>,
    ) -> Result<Self, &'static str> {
        context.validate()?;
        if values.len() != context.axes.len() || evidence.len() != values.len() {
            return Err("each declared objective requires a value and evidence record");
        }
        if evidence.iter().any(|e| e.source.trim().is_empty()) {
            return Err("evidence source must be explicit, including heuristic sources");
        }
        Ok(Self {
            context,
            values,
            evidence,
        })
    }

    pub fn evidence(&self) -> &[Evidence] {
        &self.evidence
    }

    /// Check every hard limit over the WHOLE interval. No amount of money
    /// saved can compensate for an ecological ceiling breach (or vice versa).
    /// An overlap is insufficient evidence of feasibility, not proof of breach.
    pub fn admit(&self) -> Result<Feasible<'_, 'a>, &'static str> {
        for (axis, value) in self.context.axes.iter().zip(&self.values) {
            if let Some(limit) = axis.limit {
                let within = match axis.direction {
                    Direction::Minimize => value.upper <= limit,
                    Direction::Maximize => value.lower >= limit,
                };
                if !within {
                    return Err("hard-limit feasibility is not established");
                }
            }
        }
        Ok(Feasible { observation: self })
    }
}

/// Witness of numeric feasibility under the DECLARED model, not deployment
/// approval. No public constructor, no deserialization escape hatch.
#[derive(Debug)]
pub struct Feasible<'o, 'c> {
    observation: &'o Observation<'c>,
}

impl Feasible<'_, '_> {
    pub fn evidence(&self) -> &[Evidence] {
        self.observation.evidence()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Relation {
    Improvement,
    Regression,
    TradeOff,
    Neutral,
    /// Overlapping uncertainty is not neutrality or a proved trade-off.
    Unresolved,
}

/// Universal interval comparison: a claim must hold for every pair in the
/// two boxes. This intentionally ignores correlation and may be conservative.
/// It cannot establish which box contains the actual world.
pub fn compare(base: &Observation<'_>, head: &Observation<'_>) -> Result<Relation, &'static str> {
    if base.context != head.context {
        return Err("incompatible workload, units, boundary, horizon, model or policy");
    }
    let mut no_worse = true;
    let mut no_better = true;
    let mut better = false;
    let mut worse = false;
    let mut neutral = true;
    for ((axis, b), h) in base.context.axes.iter().zip(&base.values).zip(&head.values) {
        let (least, most) = match axis.direction {
            Direction::Minimize => (b.lower - h.upper, b.upper - h.lower),
            Direction::Maximize => (h.lower - b.upper, h.upper - b.lower),
        };
        if !least.is_finite() || !most.is_finite() {
            return Err("comparison overflow");
        }
        no_worse &= least >= 0.0;
        no_better &= most <= 0.0;
        better |= least > axis.tolerance;
        worse |= most < -axis.tolerance;
        neutral &= least.abs() <= axis.tolerance && most.abs() <= axis.tolerance;
    }
    Ok(if no_worse && better {
        Relation::Improvement
    } else if no_better && worse {
        Relation::Regression
    } else if better && worse {
        Relation::TradeOff
    } else if neutral {
        Relation::Neutral
    } else {
        Relation::Unresolved
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> Context {
        Context {
            workload: "fixture-v1".into(),
            functional_unit: "one completed request".into(),
            boundary: "operational electricity only".into(),
            horizon: "one invocation".into(),
            model_revision: "synthetic-v1".into(),
            policy_revision: "test-policy-v1".into(),
            axes: vec![
                Axis {
                    name: "carbon".into(),
                    unit: "gCO2e".into(),
                    direction: Direction::Minimize,
                    tolerance: 0.0,
                    limit: Some(10.0),
                },
                Axis {
                    name: "cost".into(),
                    unit: "USD".into(),
                    direction: Direction::Minimize,
                    tolerance: 0.0,
                    limit: Some(20.0),
                },
            ],
        }
    }

    fn observation<'a>(ctx: &'a Context, bounds: &[(f64, f64)]) -> Observation<'a> {
        Observation::new(
            ctx,
            bounds
                .iter()
                .map(|&(lo, hi)| Interval::new(lo, hi).unwrap())
                .collect(),
            bounds
                .iter()
                .map(|_| Evidence {
                    confidence: Confidence::Estimated,
                    source: "synthetic test fixture, not measurement".into(),
                    assumptions: vec!["box contains modeled possibilities only".into()],
                })
                .collect(),
        )
        .unwrap()
    }

    #[test]
    fn numeric_refinements_reject_invalid_values() {
        for (lo, hi) in [(2.0, 1.0), (f64::NAN, 1.0), (0.0, f64::INFINITY)] {
            assert!(Interval::new(lo, hi).is_err());
        }
        let ctx = context();
        assert!(Observation::new(&ctx, vec![], vec![]).is_err());
        let mut invalid = context();
        invalid.axes[1].name = "carbon".into();
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn savings_cannot_buy_an_ecological_exception() {
        let ctx = context();
        let good = observation(&ctx, &[(8.0, 10.0), (10.0, 15.0)]);
        let bad = observation(&ctx, &[(9.0, 11.0), (0.0, 0.0)]);
        assert!(good.admit().is_ok());
        assert!(bad.admit().is_err());
        assert_eq!(
            good.admit().unwrap().evidence()[0].confidence,
            Confidence::Estimated
        );
    }

    #[test]
    fn interval_overlap_is_not_a_point_estimate_win() {
        let ctx = context();
        let base = observation(&ctx, &[(4.0, 8.0), (5.0, 5.0)]);
        let head = observation(&ctx, &[(3.0, 7.0), (5.0, 5.0)]);
        assert_eq!(compare(&base, &head).unwrap(), Relation::Unresolved);
        let separated = observation(&ctx, &[(1.0, 2.0), (5.0, 5.0)]);
        assert_eq!(compare(&base, &separated).unwrap(), Relation::Improvement);
        assert_eq!(compare(&separated, &base).unwrap(), Relation::Regression);
        let trade = observation(&ctx, &[(1.0, 2.0), (6.0, 6.0)]);
        assert_eq!(compare(&base, &trade).unwrap(), Relation::TradeOff);
    }

    #[test]
    fn scope_and_units_cannot_be_silently_transported() {
        let ctx = context();
        let mut other = context();
        other.axes[0].unit = "kgCO2e".into();
        let a = observation(&ctx, &[(5.0, 5.0), (5.0, 5.0)]);
        let b = observation(&other, &[(1.0, 1.0), (1.0, 1.0)]);
        assert!(compare(&a, &b).is_err());
        assert_eq!(compare(&a, &a).unwrap(), Relation::Neutral);
    }
}
