// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! `oikos-parser` — Logos lexer and Chumsky parser for the Oikos DSL.
//!
//! # Pipeline
//!
//! ```text
//! source text
//!     └─▶ Lexer (Logos)   → token stream
//!             └─▶ Parser (Chumsky) → oikos_syntax::Model
//! ```
//!
//! # Status
//!
//! Scaffold only.  The lexer token enum and parser combinators are defined
//! here as stubs; full implementation follows once the surface syntax is
//! finalised in `spec/SPEC.adoc`.
//!
//! # Error recovery
//!
//! Chumsky's error recovery is used throughout: the parser returns
//! `(Option<Model>, Vec<ParseError>)` so that callers receive all errors
//! in a single pass rather than failing on the first problem.

pub mod error;
pub mod lexer;
pub mod parser;

pub use error::ParseError;
pub use oikos_syntax::Model;

/// Parse an Oikos source string, returning the AST and any parse errors.
///
/// Errors are non-fatal where possible: the parser attempts recovery and
/// continues, accumulating all errors for batch reporting via `ariadne`.
///
/// # Arguments
///
/// * `source`   — the full source text of the `.oikos` file
/// * `filename` — used only in diagnostic messages; need not be a real path
///
/// # Errors
///
/// Returns `Err` only when parsing fails so badly that no partial AST can be
/// recovered.  Recoverable errors are collected in the `Vec<ParseError>`
/// on the `Ok` path.
pub fn parse(
    source: &str,
    filename: &str,
) -> Result<(Option<Model>, Vec<ParseError>), ParseError> {
    let _filename = filename; // used in diagnostics once implemented
    parser::parse_model(source)
}
