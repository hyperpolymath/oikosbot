// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Chumsky parser combinators for the Oikos DSL.
//!
//! # Status: scaffold
//!
//! The entry point [`parse_model`] is wired up but returns a placeholder
//! until the parser combinators are implemented.  The function signature
//! and return type are stable.

use oikos_syntax::Model;

use crate::error::ParseError;

/// Parse a full Oikos source text, returning the top-level [`Model`] AST node.
///
/// Returns `(None, errors)` when the source is so malformed that no partial
/// model can be recovered.  Returns `(Some(model), errors)` when at least a
/// partial model is available, along with any non-fatal errors encountered.
pub fn parse_model(
    _source: &str,
) -> Result<(Option<Model>, Vec<ParseError>), ParseError> {
    // TODO: implement using chumsky combinators.
    //
    // Outline of the grammar (see spec/SPEC.adoc for the full BNF):
    //
    //   model        ::= "model" IDENT "(" "period" ":" IDENT ")" "{" item* "}"
    //   item         ::= period_decl | rate_decl | account_decl
    //                  | instrument_decl | sector_decl | godley_block
    //                  | transfer_expr | convert_expr | close_expr
    //   period_decl  ::= "period" IDENT ":" period_kind "from" DATE "to" DATE
    //   godley_block ::= "godley" "{" godley_table "}"
    //   transfer_expr::= "transfer" account_ref "→" account_ref "{" amount_clause "}"
    //   …
    //
    // Error recovery should use chumsky's `.recover_with(skip_then_retry_until(…))`
    // at statement boundaries.

    Err(ParseError::Internal {
        message: "parser not yet implemented — see oikos-parser/src/parser.rs".into(),
    })
}
