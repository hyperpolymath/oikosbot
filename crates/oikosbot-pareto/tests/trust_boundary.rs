// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell

use oikosbot_metrics::Confidence;
use oikosbot_pareto::{
    assess, compare, dominates, frontier_indices, pareto_scores, Direction, Objective,
    ParetoVerdict,
};

fn axes() -> Vec<Objective> {
    ["carbon", "cost", "time"]
        .iter()
        .map(|name| Objective::new(name, Direction::Minimize, 1.0))
        .collect()
}

#[test]
fn tolerated_losses_must_not_create_a_dominance_cycle() {
    // Old epsilon relation: a > b > c > a, empty frontier, all scores 100.
    let points = vec![
        vec![0.0, 1.0, 2.0],
        vec![2.0, 0.0, 1.0],
        vec![1.0, 2.0, 0.0],
    ];
    let objectives = axes();
    assert_eq!(frontier_indices(&objectives, &points, 1.0), vec![0, 1, 2]);
    for a in &points {
        for b in &points {
            assert!(!dominates(&objectives, a, b, 1.0));
        }
    }
    assert!(dominates(&objectives, &[0.0; 3], &[2.0; 3], 1.0));
}

#[test]
fn malformed_data_is_not_neutral_actionable_or_optimal() {
    let objectives = axes();
    for bad in [vec![], vec![1.0], vec![f64::NAN; 3], vec![f64::INFINITY; 3]] {
        let result = assess(
            &objectives,
            &[1.0; 3],
            &bad,
            &[Confidence::Measured; 3],
            0.0,
        );
        assert_eq!(result.verdict, ParetoVerdict::Indeterminate);
        assert!(!result.actionable);
        assert!(frontier_indices(&objectives, &[bad.clone()], 0.0).is_empty());
        assert_eq!(pareto_scores(&objectives, &[bad], 0.0), vec![0.0]);
    }
    for eps in [-1.0, f64::NAN, f64::INFINITY] {
        assert_eq!(
            compare(&objectives, &[1.0; 3], &[2.0; 3], eps),
            ParetoVerdict::Indeterminate
        );
    }
}

#[test]
fn unchanged_axis_is_still_part_of_the_no_worse_claim() {
    let result = assess(
        &axes(),
        &[2.0; 3],
        &[1.0, 2.0, 2.0],
        &[
            Confidence::Measured,
            Confidence::Unknown,
            Confidence::Measured,
        ],
        0.0,
    );
    assert_eq!(result.verdict, ParetoVerdict::Improvement);
    assert!(!result.actionable);
}

#[test]
fn dominance_is_transitive_on_a_finite_grid() {
    let objectives = axes();
    let mut points = Vec::new();
    for x in 0..3 {
        for y in 0..3 {
            for z in 0..3 {
                points.push(vec![x as f64, y as f64, z as f64]);
            }
        }
    }
    for eps in [0.0, 0.5, 1.0] {
        for a in &points {
            for b in &points {
                for c in &points {
                    if dominates(&objectives, a, b, eps) && dominates(&objectives, b, c, eps) {
                        assert!(dominates(&objectives, a, c, eps));
                    }
                }
            }
        }
    }
}
