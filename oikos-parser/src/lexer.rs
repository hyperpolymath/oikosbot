// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Logos lexer for the Oikos DSL.
//!
//! # Status: scaffold
//!
//! Token variants are declared; the regex patterns are stubs to be filled in
//! once the surface syntax grammar is stable.  All keywords follow the
//! economic naming register: `account`, `sector`, `godley`, `instrument`,
//! `transfer`, `close`, `convert`, `period`, `rate`, `model`.

use logos::Logos;

/// The token stream produced by the Logos lexer.
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n]+")] // skip whitespace
#[logos(skip r"--[^\n]*")]    // skip line comments (`--` like Haskell/Lua)
pub enum Token {
    // ── Keywords ─────────────────────────────────────────────────────────────
    #[token("model")]      KwModel,
    #[token("period")]     KwPeriod,
    #[token("rate")]       KwRate,
    #[token("account")]    KwAccount,
    #[token("sector")]     KwSector,
    #[token("instrument")] KwInstrument,
    #[token("godley")]     KwGodley,
    #[token("transfer")]   KwTransfer,
    #[token("convert")]    KwConvert,
    #[token("close")]      KwClose,
    #[token("from")]       KwFrom,
    #[token("into")]       KwInto,
    #[token("via")]        KwVia,
    #[token("on")]         KwOn,
    #[token("states")]     KwStates,
    #[token("initial")]    KwInitial,
    #[token("transitions")]KwTransitions,
    #[token("asset")]      KwAsset,
    #[token("liability")]  KwLiability,
    #[token("amount")]     KwAmount,
    #[token("description")]KwDescription,
    #[token("balance")]    KwBalance,
    #[token("fraction")]   KwFraction,

    // ── Account kinds ─────────────────────────────────────────────────────────
    #[token("Stock")] KwStock,
    #[token("Flow")]  KwFlow,

    // ── Period kinds ──────────────────────────────────────────────────────────
    #[token("FiscalYear")] KwFiscalYear,
    #[token("HalfYear")]   KwHalfYear,
    #[token("Quarter")]    KwQuarter,
    #[token("Month")]      KwMonth,
    #[token("Week")]       KwWeek,

    // ── Punctuation ───────────────────────────────────────────────────────────
    #[token("{")]  LBrace,
    #[token("}")]  RBrace,
    #[token("(")]  LParen,
    #[token(")")]  RParen,
    #[token("|")]  Pipe,
    #[token(":")]  Colon,
    #[token(",")]  Comma,
    #[token(";")]  Semicolon,
    #[token("→")]  Arrow,       // U+2192 RIGHTWARDS ARROW
    #[token("->")]  AsciiArrow, // ASCII fallback

    // ── Literals ─────────────────────────────────────────────────────────────
    /// Decimal number, optionally with underscores as digit separators.
    #[regex(r"[0-9][0-9_]*(\.[0-9][0-9_]*)?")]
    Number,

    /// ISO 8601 date: YYYY-MM-DD
    #[regex(r"\d{4}-\d{2}-\d{2}")]
    Date,

    /// Identifier: starts with a letter or underscore, continues with
    /// alphanumerics and underscores.
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    Ident,

    /// Double-quoted string literal.
    #[regex(r#""([^"\\]|\\.)*""#)]
    StringLit,
}
