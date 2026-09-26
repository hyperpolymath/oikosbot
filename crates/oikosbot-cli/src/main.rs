// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! # OikosBot CLI
//!
//! Ecological and economic code analysis tool.
//! Built with Eclexia principles - exploring resource-aware design.

#![forbid(unsafe_code)]
mod config;
mod estate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use oikosbot_analysis::analyze_file;
use std::fs;
use std::path::PathBuf;
use tracing::info;
use walkdir::WalkDir;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "oikosbot")]
#[command(about = "Ecological & Economic Code Analysis", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a single file
    Analyze {
        /// File to analyze
        file: PathBuf,

        /// Output format (text, json, sarif)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Analyze a directory recursively
    Check {
        /// Directory to check
        path: PathBuf,

        /// Minimum eco score threshold (0-100); defaults to the config's
        /// thresholds.eco_minimum, else 50
        #[arg(long)]
        eco_threshold: Option<f64>,

        /// Path to an .oikos.yml config; defaults to <path>/.oikos.yml if present
        #[arg(long)]
        config: Option<PathBuf>,

        /// Output format (text, json, sarif)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include security-sustainability correlation (requires panic-attack feature)
        #[arg(long)]
        security: bool,

        /// Directory containing Eclexia policy files (.ecl)
        #[arg(long)]
        policy_dir: Option<PathBuf>,
    },

    /// Generate a full report for a directory (alias for check with defaults)
    Report {
        /// Directory to analyze
        path: PathBuf,

        /// Output format (text, json, sarif)
        #[arg(short, long, default_value = "sarif")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Minimum eco score threshold (0-100); defaults to the config's
        /// thresholds.eco_minimum, else 50
        #[arg(long)]
        eco_threshold: Option<f64>,

        /// Path to an .oikos.yml config; defaults to <path>/.oikos.yml if present
        #[arg(long)]
        config: Option<PathBuf>,

        /// Include security-sustainability correlation (requires panic-attack feature)
        #[arg(long)]
        security: bool,

        /// Directory containing Eclexia policy files (.ecl)
        #[arg(long)]
        policy_dir: Option<PathBuf>,
    },

    /// Compare base vs head analyses and issue a Pareto verdict
    ///
    /// Verdicts: pareto-improvement (better on >=1 objective, worse on none),
    /// pareto-regression (worse on >=1, better on none), trade-off (mixed —
    /// must be documented), neutral (no objective moved beyond tolerance).
    Compare {
        /// Base directory or file (e.g. a checkout of the target branch)
        base: PathBuf,

        /// Head directory or file (the proposed change)
        head: PathBuf,

        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Write output to file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// File containing the PR description; checked for a documented
        /// trade-off ("Pareto-Trade-off:" trailer or heading)
        #[arg(long)]
        pr_body: Option<PathBuf>,

        /// Path to an .oikos.yml config; defaults to <head>/.oikos.yml if present
        #[arg(long)]
        config: Option<PathBuf>,

        /// Exit non-zero on an undocumented, actionable regression or
        /// trade-off (regulator behaviour; default is advisory)
        #[arg(long)]
        check: bool,
    },

    /// Run as a gitbot-fleet member
    Fleet {
        /// Repository path to analyze
        path: PathBuf,

        /// Path to shared context JSON file
        #[arg(short, long)]
        context: Option<PathBuf>,
    },

    /// Show analysis of oikosbot itself (dogfooding!)
    SelfAnalyze,

    /// Estate-level telemetry, capability and DEA analysis (read-only)
    Estate {
        #[command(subcommand)]
        cmd: estate::EstateCmd,
    },
}

fn main() {
    if let Err(error) = run() {
        eprintln!("no check performed: {error:#}");
        std::process::exit(2);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let threshold_check = matches!(&cli.command, Commands::Check { .. });

    // Set up logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(log_level)
        .init();

    match cli.command {
        Commands::Analyze {
            file,
            format,
            output,
        } => {
            info!("Analyzing file: {}", file.display());
            let mut results = analyze_file(&file)?;
            oikosbot_pareto::apply_to_results(&mut results, oikosbot_pareto::DEFAULT_EPSILON);
            emit_output(&results, &format, output.as_deref())?;
        }

        Commands::Check {
            path,
            eco_threshold,
            config,
            format,
            output,
            security,
            policy_dir,
        }
        | Commands::Report {
            path,
            format,
            output,
            eco_threshold,
            config,
            security,
            policy_dir,
        } => {
            info!("Checking directory: {}", path.display());

            let cfg = config::resolve(config.as_deref(), &path)?;
            if let Some(ref c) = cfg {
                info!(
                    "config: {} (mode {:?}, blocking: {})",
                    c.source.display(),
                    c.mode,
                    c.enforcement_blocking
                );
            }
            let eco_threshold = eco_threshold
                .or(cfg.as_ref().and_then(|c| c.eco_threshold))
                .unwrap_or(50.0);

            anyhow::ensure!(
                eco_threshold.is_finite() && (0.0..=100.0).contains(&eco_threshold),
                "eco threshold must be finite and between 0 and 100"
            );
            let mut all_results = collect_directory_results(&path, cfg.as_ref())?;

            // Intra-repo Pareto pass over the organic code units (before
            // synthetic policy/security findings join the set): frontier
            // membership, ParetoScore, and the full EconScore composition.
            oikosbot_pareto::apply_to_results(&mut all_results, oikosbot_pareto::DEFAULT_EPSILON);

            // Security-sustainability correlation
            if security {
                run_security_correlation(&path, &mut all_results)?;
            }

            // Policy evaluation
            if let Some(ref pdir) = policy_dir {
                run_policy_evaluation(pdir, &mut all_results)?;
            }

            // Emit formatted output
            match format.as_str() {
                "sarif" | "json" => {
                    emit_output(&all_results, &format, output.as_deref())?;
                }
                "text" => {
                    println!(
                        "Checking directory: {} (eco threshold: {})\n",
                        path.display(),
                        eco_threshold
                    );

                    let mut files_below_threshold = 0u32;
                    for result in &all_results {
                        if result.health.eco_score.0 < eco_threshold {
                            files_below_threshold += 1;
                            println!(
                                "  BELOW THRESHOLD: {} :: {} (eco: {:.1}, threshold: {})",
                                result.location.file,
                                result.location.name.as_deref().unwrap_or("<anon>"),
                                result.health.eco_score.0,
                                eco_threshold
                            );
                        }
                    }

                    print_summary(&all_results, eco_threshold, files_below_threshold);

                    if let Some(ref out_path) = output {
                        // Also write text summary to file
                        let text = format_results_text(&all_results);
                        fs::write(out_path, text)?;
                        println!("\nOutput written to: {}", out_path.display());
                    }

                    // Gate evaluation is below rendering, independent of format.
                }
                _ => anyhow::bail!("Unsupported format: {}", format),
            }
            // `report` is always advisory. `check` must not green-light a gate
            // merely because estimates have fallen below/above a score floor.
            if threshold_check && cfg.as_ref().is_none_or(|c| c.enforcement_blocking) {
                anyhow::ensure!(
                    all_results.iter().all(|r| matches!(
                        r.confidence,
                        oikosbot_metrics::Confidence::Measured
                            | oikosbot_metrics::Confidence::Calibrated
                    )),
                    "threshold check NOT enforced: resource figures are estimates; use report for advisory output"
                );
                if all_results
                    .iter()
                    .any(|r| r.health.eco_score.0 < eco_threshold)
                {
                    std::process::exit(1);
                }
            }
        }

        Commands::Compare {
            base,
            head,
            format,
            output,
            pr_body,
            config,
            check,
        } => {
            let cfg = config::resolve(config.as_deref(), &head)?;
            let collect = |p: &std::path::Path| -> Result<Vec<oikosbot_metrics::AnalysisResult>> {
                if p.is_file() {
                    analyze_file(p)
                } else {
                    collect_directory_results(p, cfg.as_ref())
                }
            };
            let base_results = collect(&base)?;
            let head_results = collect(&head)?;

            let objectives = oikosbot_pareto::result_objectives();
            let (Some(base_point), Some(head_point)) = (
                oikosbot_pareto::aggregate_point(&base_results),
                oikosbot_pareto::aggregate_point(&head_results),
            ) else {
                anyhow::bail!(
                    "no analyzable files under {} and/or {}",
                    base.display(),
                    head.display()
                );
            };

            let confidence = oikosbot_pareto::weaker_confidence(
                oikosbot_pareto::aggregate_confidence(&base_results),
                oikosbot_pareto::aggregate_confidence(&head_results),
            );
            let confidences = vec![confidence; objectives.len()];
            let assessment = oikosbot_pareto::assess(
                &objectives,
                &base_point,
                &head_point,
                &confidences,
                oikosbot_pareto::DEFAULT_EPSILON,
            );

            let documented = match pr_body {
                Some(ref p) => Some(oikosbot_pareto::tradeoff_documented(&fs::read_to_string(
                    p,
                )?)),
                None => None,
            };

            match format.as_str() {
                "json" => {
                    let payload = serde_json::json!({
                        "assessment": assessment,
                        "confidence": format!("{:?}", confidence),
                        "tradeoff_documented": documented,
                    });
                    let text = serde_json::to_string_pretty(&payload)?;
                    match output {
                        Some(ref path) => {
                            fs::write(path, &text)?;
                            eprintln!("Output written to: {}", path.display());
                        }
                        None => println!("{}", text),
                    }
                }
                "text" => {
                    let text = format_comparison(&assessment, confidence, documented);
                    match output {
                        Some(ref path) => fs::write(path, text)?,
                        None => print!("{text}"),
                    }
                }
                _ => anyhow::bail!("Unsupported comparison format: {}", format),
            }

            // Advisory by default; --check enforces the trade-off doctrine —
            // and only on verdicts whose drivers are measured or calibrated.
            let needs_documentation = matches!(
                assessment.verdict,
                oikosbot_pareto::ParetoVerdict::Regression
                    | oikosbot_pareto::ParetoVerdict::TradeOff
            );
            if check {
                anyhow::ensure!(
                    assessment.verdict != oikosbot_pareto::ParetoVerdict::Indeterminate
                        && matches!(confidence, oikosbot_metrics::Confidence::Measured
                            | oikosbot_metrics::Confidence::Calibrated),
                    "--check requested but NOT enforced: no validated resource evidence; advisory result only (use compare without --check)"
                );
                if needs_documentation && documented != Some(true) && assessment.actionable {
                    std::process::exit(1);
                }
            }
        }

        Commands::Fleet { path, context } => {
            // The gitbot-fleet bridge (`oikosbot-fleet`) is intentionally NOT a
            // dependency of the standalone CLI — OikosBot and gitbot-fleet are
            // separate projects. The bridge crate is excluded from the default
            // workspace; build it explicitly when wiring OikosBot into the fleet.
            let ctx = context
                .as_deref()
                .map(|p| p.display().to_string())
                .unwrap_or_default();
            eprintln!(
                "Fleet integration lives in the optional `oikosbot-fleet` crate, which is\n\
                 excluded from the default workspace so OikosBot builds standalone.\n\n\
                 To run it (with hyperpolymath/gitbot-fleet checked out as a sibling):\n  \
                 cargo run --manifest-path crates/oikosbot-fleet/Cargo.toml -- {} {}\n\n\
                 See crates/oikosbot-fleet/README.adoc and DISAMBIGUATION.adoc.",
                path.display(),
                ctx,
            );
        }

        Commands::SelfAnalyze => {
            println!("OikosBot Self-Analysis (Dogfooding!)");
            println!("==========================================\n");
            println!("Analyzing oikosbot's own resource usage...\n");

            let analyzer_src = PathBuf::from("crates/oikosbot-analysis/src/analyzer.rs");
            if analyzer_src.exists() {
                let results = analyze_file(&analyzer_src)?;
                print_results_text(&results);

                println!("\nMeta-Analysis:");
                println!("This analyzer used minimal resources to analyze itself.");
                println!("Eclexia-inspired design: explicit resource tracking from day 1.");
            } else {
                println!("Run from oikosbot repository root.");
            }
        }

        Commands::Estate { cmd } => {
            estate::run(cmd)?;
        }
    }

    Ok(())
}

/// Collect analysis results from all supported files in a directory,
/// honoring the config's exclude globs and language list when present.
fn collect_directory_results(
    path: &std::path::Path,
    cfg: Option<&config::ResolvedConfig>,
) -> Result<Vec<oikosbot_metrics::AnalysisResult>> {
    let mut all_results = Vec::new();
    let default_extensions = ["rs", "js", "py"];
    let allowed: &[&str] = cfg
        .map(|c| c.allowed_extensions.as_slice())
        .unwrap_or(&default_extensions);

    for entry in WalkDir::new(path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_str().unwrap_or("");
            !matches!(
                name,
                "target" | "node_modules" | ".git" | "dist" | "build" | ".cache"
            )
        })
    {
        let entry = entry?;
        let entry_path = entry.path();
        // Do not follow symlink files outside the declared analysis boundary.
        if !entry.file_type().is_file() {
            continue;
        }

        let ext = entry_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        if !allowed.contains(&ext) {
            continue;
        }

        // Exclude globs match against the path relative to the analyzed root
        // (the shape estate .oikos.yml patterns like "**/target/**" expect).
        if let Some(c) = cfg {
            let relative = entry_path.strip_prefix(path).unwrap_or(entry_path);
            if c.exclude.is_match(relative) {
                continue;
            }
        }

        all_results.extend(analyze_file(entry_path)?);
    }

    anyhow::ensure!(
        !all_results.is_empty(),
        "no analyzable code units under {} (unsupported, empty or excluded input)",
        path.display()
    );
    Ok(all_results)
}

/// Emit analysis results in the requested format
fn emit_output(
    results: &[oikosbot_metrics::AnalysisResult],
    format: &str,
    output: Option<&std::path::Path>,
) -> Result<()> {
    anyhow::ensure!(!results.is_empty(), "no analyzable code units");
    let text = match format {
        "sarif" => oikosbot_sarif::to_sarif_json(results, VERSION)?,
        "json" => serde_json::to_string_pretty(results)?,
        "text" => format_results_text(results),
        other => anyhow::bail!("Unsupported format: {}", other),
    };

    match output {
        Some(path) => {
            fs::write(path, &text)?;
            eprintln!("Output written to: {}", path.display());
        }
        None => {
            println!("{}", text);
        }
    }

    Ok(())
}

fn format_comparison(
    assessment: &oikosbot_pareto::Comparison,
    confidence: oikosbot_metrics::Confidence,
    documented: Option<bool>,
) -> String {
    let label = match assessment.verdict {
        oikosbot_pareto::ParetoVerdict::Improvement => "PARETO IMPROVEMENT",
        oikosbot_pareto::ParetoVerdict::Regression => "PARETO REGRESSION",
        oikosbot_pareto::ParetoVerdict::TradeOff => "TRADE-OFF",
        oikosbot_pareto::ParetoVerdict::Neutral => "NEUTRAL",
        oikosbot_pareto::ParetoVerdict::Indeterminate => "NO COMPARISON",
    };
    let mut text = format!("Pareto verdict: {label}\nConfidence: {confidence:?}\n");
    for d in &assessment.deltas {
        text.push_str(&format!(
            "  {}: base {:.4}, head {:.4}, signed improvement {:.4}\n",
            d.name, d.base, d.head, d.improvement
        ));
    }
    text.push_str(&format!("Drivers: {}\n", assessment.drivers.join(", ")));
    if !assessment.actionable {
        text.push_str("Advisory: heuristic estimates cannot drive blocking decisions.\n");
    }
    if let Some(present) = documented {
        text.push_str(if present {
            "Trade-off documentation: present (not approval)\n"
        } else {
            "Trade-off documentation: MISSING\n"
        });
    }
    text
}

fn print_summary(
    all_results: &[oikosbot_metrics::AnalysisResult],
    eco_threshold: f64,
    files_below_threshold: u32,
) {
    let total_files = all_results
        .iter()
        .map(|r| r.location.file.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len();

    println!("\n--- Summary ---");
    println!("Files analyzed:        {}", total_files);
    println!("Functions found:       {}", all_results.len());
    println!("Below threshold:       {}", files_below_threshold);

    if !all_results.is_empty() {
        let avg_eco: f64 = all_results
            .iter()
            .map(|r| r.health.eco_score.0)
            .sum::<f64>()
            / all_results.len() as f64;
        let avg_overall: f64 =
            all_results.iter().map(|r| r.health.overall).sum::<f64>() / all_results.len() as f64;
        let total_energy: f64 = all_results.iter().map(|r| r.resources.energy.0).sum();
        let total_carbon: f64 = all_results.iter().map(|r| r.resources.carbon.0).sum();

        println!("Avg eco score:         {:.1}/100", avg_eco);
        println!("Avg overall health:    {:.1}/100", avg_overall);
        println!("Total est. energy:     {:.2} J", total_energy);
        println!("Total est. carbon:     {:.4} gCO2e", total_carbon);
    }

    if files_below_threshold > 0 {
        println!(
            "\nResult: FAIL ({} functions below eco threshold {})",
            files_below_threshold, eco_threshold
        );
    } else {
        println!(
            "\nResult: PASS (all functions meet eco threshold {})",
            eco_threshold
        );
    }
}

fn print_results_text(results: &[oikosbot_metrics::AnalysisResult]) {
    for result in results {
        println!(
            "\nFunction: {}",
            result.location.name.as_deref().unwrap_or("<anonymous>")
        );
        println!(
            "   Location: {}:{}:{}",
            result.location.file, result.location.line, result.location.column
        );
        println!("\n   Resources:");
        println!("     Energy:   {:.2} J", result.resources.energy.0);
        println!("     Time:     {:.2} ms", result.resources.duration.0);
        println!("     Carbon:   {:.4} gCO2e", result.resources.carbon.0);
        println!("     Memory:   {} bytes", result.resources.memory.0);

        println!("\n   Health Index:");
        println!("     Eco:      {:.1}/100", result.health.eco_score.0);
        println!("     Econ:     {:.1}/100", result.health.econ_score.0);
        println!("     Quality:  {:.1}/100", result.health.quality_score);
        println!("     Overall:  {:.1}/100", result.health.overall);

        if !result.recommendations.is_empty() {
            println!("\n   Recommendations:");
            for rec in &result.recommendations {
                println!("     - {}", rec);
            }
        }
    }

    println!("\nAnalysis complete");
}

/// Run Eclexia policy evaluation
fn run_policy_evaluation(
    policy_dir: &std::path::Path,
    results: &mut Vec<oikosbot_metrics::AnalysisResult>,
) -> Result<()> {
    match oikosbot_eclexia::evaluate_policies(policy_dir, results) {
        Ok(decisions) => {
            let warns = decisions
                .iter()
                .filter(|d| d.outcome != oikosbot_eclexia::PolicyOutcome::Pass)
                .count();
            eprintln!(
                "Policy evaluation: {} policies, {} warnings/failures",
                decisions.len(),
                warns
            );
            for d in &decisions {
                if d.outcome != oikosbot_eclexia::PolicyOutcome::Pass {
                    eprintln!("  {:?}: {} - {}", d.outcome, d.policy_name, d.message);
                }
            }
            // Convert policy decisions to analysis results for SARIF output
            let policy_results = oikosbot_eclexia::decisions_to_results(&decisions);
            results.extend(policy_results);
        }
        Err(e) => {
            return Err(e);
        }
    }
    Ok(())
}

/// Run security-sustainability correlation if the feature is available
fn run_security_correlation(
    path: &std::path::Path,
    results: &mut Vec<oikosbot_metrics::AnalysisResult>,
) -> Result<()> {
    if let Some(directive) = oikosbot_analysis::directives::check_directive(path, "panic-attack") {
        anyhow::ensure!(
            directive.allow,
            "security scan denied by repository directive"
        );
    }
    let correlation = oikosbot_analysis::security::correlate(path, results)?;
    results.extend(correlation.security_findings);
    Ok(())
}

fn format_results_text(results: &[oikosbot_metrics::AnalysisResult]) -> String {
    let mut out = String::new();

    for result in results {
        out.push_str(&format!(
            "\nFunction: {}\n",
            result.location.name.as_deref().unwrap_or("<anonymous>")
        ));
        out.push_str(&format!(
            "   Location: {}:{}:{}\n",
            result.location.file, result.location.line, result.location.column
        ));
        out.push_str(&format!("   Energy: {:.2} J\n", result.resources.energy.0));
        out.push_str(&format!(
            "   Carbon: {:.4} gCO2e\n",
            result.resources.carbon.0
        ));
        out.push_str(&format!(
            "   Eco: {:.1}/100  Overall: {:.1}/100\n",
            result.health.eco_score.0, result.health.overall
        ));
        out.push_str(&format!(
            "   Duration: {:.4} ms  Memory: {} bytes\n   Confidence: {:?}\n   Rule: {}\n",
            result.resources.duration.0,
            result.resources.memory.0,
            result.confidence,
            result.rule_id
        ));
        for recommendation in &result.recommendations {
            out.push_str(&format!("   Recommendation: {recommendation}\n"));
        }
    }

    out
}
