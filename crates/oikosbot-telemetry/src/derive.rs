// SPDX-License-Identifier: MPL-2.0
//! Derived metrics with honest confidence labels.
//!
//! Transforms flat RunRow telemetry into aggregated, economically meaningful
//! metrics with explicit confidence levels. The confidence ladder is the honesty
//! contract: only API-sourced quantities are Measured; coefficients over
//! measurements are Calibrated; anything resting on a declared assumption
//! (grid region) is Estimated.

use crate::rows::RunRow;
use oikosbot_metrics::Confidence;
use std::collections::BTreeMap;

/// Assumptions for deriving energy, carbon, and cost from runtime duration.
///
/// Each field includes documented constants and sources; these are the
/// calibration parameters that will be published to users.
pub struct Assumptions {
    /// Active power attributable to a 4-vCPU share of an AMD EPYC 7763 host
    /// (Azure Standard_D4ads_v5, GitHub-hosted ubuntu-latest). 280 W TDP ×
    /// vhost ratio 0.03125 (4/128 threads, per Eco-CI) ≈ 8.75 W at full load;
    /// declared average utilisation 0.5 → 4.4 W, rounded conservatively up.
    pub power_w: f64,

    /// Azure fleet PUE per Cloud Carbon Footprint published constants.
    pub pue: f64,

    /// DECLARED grid intensity, gCO2e/kWh. GitHub does not expose runner
    /// region; this is an assumption, which is why carbon is Estimated.
    /// 475 = global average, matching the existing carbon.rs constant.
    pub grid_gco2_per_kwh: f64,

    /// Imputed price of a hosted-runner minute (USD, linux 2-core list price).
    /// VERIFY against the live GitHub pricing docs before publishing money
    /// figures — a ~40% cut took effect 2026-01-01.
    pub usd_per_minute: f64,
}

impl Default for Assumptions {
    fn default() -> Self {
        Self {
            power_w: 5.0,
            pue: 1.185,
            grid_gco2_per_kwh: 475.0,
            usd_per_minute: 0.008,
        }
    }
}

/// One repository's aggregated energy, carbon, and imputed cost metrics.
pub struct DerivedRepo {
    /// Repository identifier (e.g. "owner/repo")
    pub repo: String,

    /// Total wall-clock minutes of all runs in this repository
    pub wall_minutes: f64,

    /// Total energy in kWh (accounting for PUE)
    pub energy_kwh: f64,

    /// Total carbon in grams of CO2 equivalent
    pub carbon_g: f64,

    /// Imputed total cost in USD
    pub imputed_cost_usd: f64,
}

/// Aggregate runs by repository and derive metrics.
///
/// Sums duration_s across all runs for each repo, then applies the
/// Assumptions to compute wall_minutes, energy_kwh, carbon_g, and imputed_cost_usd.
pub fn derive_per_repo(runs: &[RunRow], a: &Assumptions) -> Vec<DerivedRepo> {
    let mut minutes: BTreeMap<String, f64> = BTreeMap::new();
    for r in runs {
        *minutes.entry(r.repo.clone()).or_default() += r.duration_s as f64 / 60.0;
    }
    minutes
        .into_iter()
        .map(|(repo, wall_minutes)| {
            let energy_kwh = wall_minutes / 60.0 * a.power_w / 1000.0 * a.pue;
            DerivedRepo {
                carbon_g: energy_kwh * a.grid_gco2_per_kwh,
                imputed_cost_usd: wall_minutes * a.usd_per_minute,
                repo,
                wall_minutes,
                energy_kwh,
            }
        })
        .collect()
}

/// Return the confidence level for a derived metric.
///
/// The ladder is the honesty contract: only API-sourced quantities are
/// Measured; coefficients over measurements are Calibrated; anything
/// resting on a declared assumption (grid region) is Estimated.
pub fn confidence_of(metric: &str) -> Confidence {
    match metric {
        "wall_minutes" => Confidence::Measured,
        "energy_kwh" | "imputed_cost_usd" => Confidence::Calibrated,
        "carbon_g" => Confidence::Estimated,
        _ => Confidence::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(repo: &str, dur: i64) -> RunRow {
        RunRow {
            repo: repo.to_string(),
            run_id: 0,
            workflow_name: String::new(),
            workflow_path: String::new(),
            event: String::new(),
            conclusion: "success".to_string(),
            started_at: String::new(),
            updated_at: String::new(),
            duration_s: dur,
        }
    }

    #[test]
    fn derives_minutes_energy_carbon_cost() {
        let a = Assumptions::default();
        let d = derive_per_repo(&[run("o/r", 600)], &a);
        assert!((d[0].wall_minutes - 10.0).abs() < 1e-9);
        let expect_kwh = 10.0 / 60.0 * a.power_w / 1000.0 * a.pue;
        assert!((d[0].energy_kwh - expect_kwh).abs() < 1e-12);
        assert!((d[0].carbon_g - expect_kwh * a.grid_gco2_per_kwh).abs() < 1e-9);
    }

    #[test]
    fn confidence_ladder_is_honest() {
        assert_eq!(confidence_of("wall_minutes"), Confidence::Measured);
        assert_eq!(confidence_of("energy_kwh"), Confidence::Calibrated);
        assert_eq!(confidence_of("carbon_g"), Confidence::Estimated);
        assert_eq!(confidence_of("imputed_cost_usd"), Confidence::Calibrated);
    }
}
