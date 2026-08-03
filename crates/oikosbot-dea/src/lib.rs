// SPDX-License-Identifier: MPL-2.0
//! Data Envelopment Analysis: input-oriented CCR and BCC, solved as plain
//! LPs. Envelopment form yields θ and the peer reference set; the multiplier
//! (dual) form yields the virtual input/output weights — the shadow prices.
//! Solving BOTH and asserting the objectives agree is a built-in
//! strong-duality verification (spec §Verification 3).
use anyhow::Result;
use good_lp::{constraint, variable, Expression, ProblemVariables, Solution, SolverModel};
use serde::Serialize;

/// A Decision-Making Unit: one repo (or other estate entity) scored by DEA.
#[derive(Debug, Clone)]
pub struct Dmu {
    pub name: String,
    pub inputs: Vec<f64>,
    pub outputs: Vec<f64>,
}

/// Efficiency score for a single DMU, computed from paired envelopment and
/// multiplier LPs.
#[derive(Debug, Clone, Serialize)]
pub struct DeaScore {
    pub name: String,
    /// Envelopment objective (CCR, constant returns to scale).
    pub theta_ccr: f64,
    /// Multiplier objective — must equal `theta_ccr` within tolerance
    /// (strong duality).
    pub theta_mult: f64,
    /// Envelopment objective under variable returns to scale (BCC).
    pub theta_bcc: f64,
    /// Peer DMUs with positive envelopment weight (λ_j > 1e-6), excluding self.
    pub peers: Vec<(String, f64)>,
    /// v_i — input shadow prices from the multiplier form.
    pub input_weights: Vec<f64>,
    /// u_r — output shadow prices from the multiplier form.
    pub output_weights: Vec<f64>,
}

const FLOOR: f64 = 1e-6;
const PEER_TOL: f64 = 1e-6;

fn floored(d: &Dmu) -> (Vec<f64>, Vec<f64>) {
    (
        d.inputs.iter().map(|v| v.max(FLOOR)).collect(),
        d.outputs.iter().map(|v| v.max(FLOOR)).collect(),
    )
}

/// Envelopment form. vrs=false → CCR; vrs=true adds Σλ=1 → BCC.
fn envelopment(o: usize, xs: &[Vec<f64>], ys: &[Vec<f64>], vrs: bool) -> Result<(f64, Vec<f64>)> {
    let n = xs.len();
    let mut pb = ProblemVariables::new();
    let theta = pb.add(variable().min(0.0));
    let lam: Vec<_> = (0..n).map(|_| pb.add(variable().min(0.0))).collect();
    let mut model = pb.minimise(theta).using(good_lp::solvers::highs::highs);
    for (i, &xoi) in xs[o].iter().enumerate() {
        let lhs: Expression = lam.iter().enumerate().map(|(j, l)| *l * xs[j][i]).sum();
        model = model.with(constraint!(lhs <= theta * xoi));
    }
    for (r, &yor) in ys[o].iter().enumerate() {
        let lhs: Expression = lam.iter().enumerate().map(|(j, l)| *l * ys[j][r]).sum();
        model = model.with(constraint!(lhs >= yor));
    }
    if vrs {
        let s: Expression = lam.iter().map(|l| Expression::from(*l)).sum();
        model = model.with(constraint!(s == 1.0));
    }
    let sol = model.solve()?;
    Ok((
        sol.value(theta),
        lam.iter().map(|l| sol.value(*l)).collect(),
    ))
}

/// Multiplier form: max u·y_o  s.t. v·x_o = 1,  u·y_j − v·x_j ≤ 0 ∀j,  u,v ≥ 0.
fn multiplier(o: usize, xs: &[Vec<f64>], ys: &[Vec<f64>]) -> Result<(f64, Vec<f64>, Vec<f64>)> {
    let (m, s) = (xs[o].len(), ys[o].len());
    let mut pb = ProblemVariables::new();
    let v: Vec<_> = (0..m).map(|_| pb.add(variable().min(0.0))).collect();
    let u: Vec<_> = (0..s).map(|_| pb.add(variable().min(0.0))).collect();
    let obj: Expression = u.iter().enumerate().map(|(r, uu)| *uu * ys[o][r]).sum();
    let mut model = pb
        .maximise(obj.clone())
        .using(good_lp::solvers::highs::highs);
    let norm: Expression = v.iter().enumerate().map(|(i, vv)| *vv * xs[o][i]).sum();
    model = model.with(constraint!(norm == 1.0));
    for j in 0..xs.len() {
        let uy: Expression = u.iter().enumerate().map(|(r, uu)| *uu * ys[j][r]).sum();
        let vx: Expression = v.iter().enumerate().map(|(i, vv)| *vv * xs[j][i]).sum();
        model = model.with(constraint!(uy - vx <= 0.0));
    }
    let sol = model.solve()?;
    Ok((
        sol.eval(&obj),
        v.iter().map(|x| sol.value(*x)).collect(),
        u.iter().map(|x| sol.value(*x)).collect(),
    ))
}

/// Score every DMU against the frontier formed by all DMUs, via paired
/// input-oriented CCR/BCC envelopment LPs and the CCR multiplier (dual) LP.
///
/// Inputs/outputs are floored at `1e-6` before solving (zero inputs/outputs
/// are common in the estate — dead repos — and break LP feasibility).
pub fn dea(dmus: &[Dmu]) -> Result<Vec<DeaScore>> {
    let xs: Vec<Vec<f64>> = dmus.iter().map(|d| floored(d).0).collect();
    let ys: Vec<Vec<f64>> = dmus.iter().map(|d| floored(d).1).collect();
    (0..dmus.len())
        .map(|o| {
            let (theta_ccr, lam) = envelopment(o, &xs, &ys, false)?;
            let (theta_bcc, _) = envelopment(o, &xs, &ys, true)?;
            let (theta_mult, vw, uw) = multiplier(o, &xs, &ys)?;
            let peers = lam
                .iter()
                .enumerate()
                .filter(|(j, l)| **l > PEER_TOL && *j != o)
                .map(|(j, l)| (dmus[j].name.clone(), *l))
                .collect();
            Ok(DeaScore {
                name: dmus[o].name.clone(),
                theta_ccr,
                theta_mult,
                theta_bcc,
                peers,
                input_weights: vw,
                output_weights: uw,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn dmu(n: &str, x: f64, y: f64) -> Dmu {
        Dmu {
            name: n.into(),
            inputs: vec![x],
            outputs: vec![y],
        }
    }
    #[test]
    fn single_io_matches_closed_form() {
        // A(2,2)=1.0, B(4,4)=1.0, C(8,4)=0.5
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("B", 4.0, 4.0), dmu("C", 8.0, 4.0)]).unwrap();
        let get = |n: &str| r.iter().find(|s| s.name == n).unwrap();
        assert!((get("A").theta_ccr - 1.0).abs() < 1e-6);
        assert!((get("B").theta_ccr - 1.0).abs() < 1e-6);
        assert!((get("C").theta_ccr - 0.5).abs() < 1e-6);
    }
    #[test]
    fn strong_duality_holds_and_scores_bounded() {
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("B", 4.0, 4.0), dmu("C", 8.0, 4.0)]).unwrap();
        for s in &r {
            assert!(
                (s.theta_ccr - s.theta_mult).abs() < 1e-5,
                "duality gap for {}",
                s.name
            );
            assert!(s.theta_ccr > 0.0 && s.theta_ccr <= 1.0 + 1e-9);
            assert!(s.theta_bcc + 1e-9 >= s.theta_ccr); // BCC frontier is closer
        }
    }
    #[test]
    fn inefficient_unit_names_frontier_peers() {
        let r = dea(&[dmu("A", 2.0, 2.0), dmu("C", 8.0, 4.0)]).unwrap();
        let c = r.iter().find(|s| s.name == "C").unwrap();
        assert!(c.peers.iter().any(|(n, w)| n == "A" && *w > 0.0));
    }
    #[test]
    fn two_input_dominated_unit_is_inefficient() {
        // D uses strictly more of both inputs than E for the same output.
        let e = Dmu {
            name: "E".into(),
            inputs: vec![1.0, 1.0],
            outputs: vec![1.0],
        };
        let d = Dmu {
            name: "D".into(),
            inputs: vec![2.0, 2.0],
            outputs: vec![1.0],
        };
        let r = dea(&[e, d]).unwrap();
        let d = r.iter().find(|s| s.name == "D").unwrap();
        assert!((d.theta_ccr - 0.5).abs() < 1e-6);
    }
}
