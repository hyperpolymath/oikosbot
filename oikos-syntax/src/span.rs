// SPDX-License-Identifier: EUPL-1.2
// SPDX-FileCopyrightText: 2026 Jonathan D.A. Jewell <j.d.a.jewell@open.ac.uk>

//! Source-location types used throughout the AST.
//!
//! Every AST node carries a [`Span`] so that error diagnostics can point
//! directly at the offending source text.

use serde::{Deserialize, Serialize};

/// A byte-offset range within a single source file.
///
/// `start` is inclusive; `end` is exclusive (i.e. `[start, end)`).
/// Both offsets are measured in UTF-8 bytes from the start of the file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    /// Construct a span from inclusive start and exclusive end byte offsets.
    pub fn new(start: usize, end: usize) -> Self {
        debug_assert!(start <= end, "Span start must not exceed end");
        Self { start, end }
    }

    /// Return the length of the span in bytes.
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// True when the span covers zero bytes (e.g. a synthetic/inserted node).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Extend this span to also cover `other`, returning the hull.
    pub fn hull(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    /// A zero-length sentinel span used for synthetic AST nodes.
    pub const SYNTHETIC: Span = Span { start: 0, end: 0 };
}

/// An AST value together with the source location it was parsed from.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

impl<T> Spanned<T> {
    pub fn new(node: T, span: Span) -> Self {
        Self { node, span }
    }

    /// Discard the span, returning only the inner value.
    pub fn into_node(self) -> T {
        self.node
    }

    /// Map the inner node, preserving the span.
    pub fn map<U, F: FnOnce(T) -> U>(self, f: F) -> Spanned<U> {
        Spanned {
            node: f(self.node),
            span: self.span,
        }
    }
}
