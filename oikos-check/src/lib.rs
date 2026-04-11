// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! `oikos-check` — type checker for the Oikos DSL.
//!
//! This crate wraps the Ephapax type checker and enforces Oikos-specific
//! SFC invariants that are expressed at the DSL level before desugaring.
//!
//! # Checking pipeline
//!
//! ```text
//! oikos_syntax::Model
//!   └─▶ oikos_check::check_model()
//!         ├─▶ well-formedness: periods, accounts, sectors
//!         ├─▶ Godley matrix: column-sum invariant
//!         ├─▶ instrument typestate: reachability, no dead states
//!         ├─▶ dimension check: currency mismatches
//!         └─▶ oikos_desugar::desugar()
//!               └─▶ ephapax_typing::check()   ← blocked on Ephapax parser
//! ```
//!
//! # Status: scaffold
//!
//! Ephapax type-checker integration is blocked until `ephapax-typing` is
//! available as a stable crate dependency.  The pre-desugar checks
//! (Godley invariant, dimension consistency) can be implemented independently.

pub mod error;
pub mod godley_check;
pub mod instrument_check;

pub use error::CheckError;
use oikos_syntax::Model;

/// Run all Oikos type checks on a parsed model.
///
/// Checks are run in priority order: cheaper structural checks first,
/// then the Godley invariant, then instrument typestate reachability,
/// then desugaring + Ephapax type checking.
///
/// # Errors
///
/// Returns the first fatal error encountered, together with any additional
/// diagnostics collected before that error forced termination.
pub fn check_model(model: &Model) -> Result<(), Vec<CheckError>> {
    let mut errors: Vec<CheckError> = Vec::new();

    // Phase 1 — Godley column-sum invariant.
    if let Err(godley_errors) = godley_check::check(model) {
        errors.extend(godley_errors);
    }

    // Phase 2 — instrument typestate reachability.
    if let Err(inst_errors) = instrument_check::check(model) {
        errors.extend(inst_errors);
    }

    // Phase 3 — desugar to Ephapax IR + Ephapax type checking (blocked).
    // Uncomment once ephapax-typing is available:
    //
    // match oikos_desugar::desugar(model) {
    //     Ok(ir) => {
    //         if let Err(e) = ephapax_typing::check(&ir) {
    //             errors.push(CheckError::EphapaxTypeError(e));
    //         }
    //     }
    //     Err(e) => errors.push(CheckError::Desugar(e)),
    // }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}
