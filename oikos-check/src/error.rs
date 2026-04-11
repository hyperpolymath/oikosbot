// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Error types for `oikos-check`.

use thiserror::Error;

use oikos_desugar::DesugarError;
use oikos_syntax::span::Span;

/// A type-check error produced by `oikos-check`.
#[derive(Debug, Error)]
pub enum CheckError {
    /// The Godley matrix column for a sector does not sum to zero.
    #[error(
        "accounting identity violated: sector `{sector}` column sums to {net:+}, not zero"
    )]
    GodleyImbalance { sector: String, net: i64, span: Span },

    /// An instrument typestate machine contains an unreachable state.
    #[error(
        "instrument `{instrument}`: state `{state}` is unreachable from initial state"
    )]
    UnreachableState {
        instrument: String,
        state: String,
        span: Span,
    },

    /// An instrument typestate machine has no path from a non-terminal state
    /// to any terminal state (Settled or Void), meaning the instrument cannot
    /// be discharged.
    #[error(
        "instrument `{instrument}`: state `{state}` has no path to a terminal state"
    )]
    NonTerminatingState {
        instrument: String,
        state: String,
        span: Span,
    },

    /// A currency dimension mismatch.
    #[error("dimension mismatch at {span:?}: cannot combine {lhs} and {rhs}")]
    DimensionMismatch { lhs: String, rhs: String, span: Span },

    /// A reference to an account that was not declared.
    #[error("unknown account `{name}` at {span:?}")]
    UnknownAccount { name: String, span: Span },

    /// A reference to a period that was not declared.
    #[error("unknown period `{name}` at {span:?}")]
    UnknownPeriod { name: String, span: Span },

    /// An error propagated from the desugaring pass.
    #[error("desugaring error: {0}")]
    Desugar(#[from] DesugarError),
}
