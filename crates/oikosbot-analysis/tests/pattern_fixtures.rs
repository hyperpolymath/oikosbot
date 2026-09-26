// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Paired silent/firing fixtures for every analyzer rule (issue #81, P0).
//!
//! The acceptance rule from #81 is that *every* gate demonstrates both a clean
//! input that stays silent and a broken input that fires **for the intended
//! reason**. Each pair below differs in exactly one feature — the one that
//! triggers the rule under test — so a regression cannot pass by accident: if
//! the silent side starts firing, the rule has become trigger-happy, and if the
//! firing side goes quiet, the rule has stopped working.
//!
//! Every fixture is a single function, so each pair asserts on exactly one
//! finding. That matters because `AnalysisResult::rule_id` reports the *most
//! significant* pattern found, in `patterns::detect_patterns` order; a fixture
//! that tripped two rules would only prove the first one. The pairs are
//! therefore also a check that the rules do not fire spuriously on each
//! other's inputs.
//!
//! These assert observable behaviour — the `rule_id` the analyzer publishes —
//! not the private detectors, and they need no filesystem: `analyze_source`
//! takes the source directly.

use oikosbot_analysis::{analyze_source, Language};
use std::path::Path;

/// Every `rule_id` the analyzer produces for a source, in source order.
fn rules(source: &str) -> Vec<String> {
    let results = analyze_source(source, Language::Rust).expect("analysis must not fail");
    results.into_iter().map(|r| r.rule_id).collect()
}

/// The one finding a single-function fixture must produce.
///
/// Panics if the fixture does not yield exactly one finding, which keeps a
/// malformed fixture from quietly testing nothing.
fn sole_rule(source: &str) -> String {
    let found = rules(source);
    assert_eq!(
        found.len(),
        1,
        "expected exactly one finding from a one-function fixture, got {found:?}"
    );
    found.into_iter().next().expect("one finding")
}

// ---------------------------------------------------------------------------
// nested-loops — fires at loop depth >= 3
// ---------------------------------------------------------------------------

/// Three nested `for` loops: the smallest nest the rule flags.
const NESTED_FIRE: &str = r#"fn three_deep(rows: &[Vec<u32>]) -> usize {
    let mut total = 0;
    for row in rows {
        for cell in row {
            for _shift in 0..4 {
                total += cell as usize;
            }
        }
    }
    total
}
"#;

/// The same nest one level shallower. Identical shape, one level short of the
/// threshold — silence must be because of the depth, not because the body is
/// somehow clean.
const NESTED_SILENT: &str = r#"fn two_deep(rows: &[Vec<u32>]) -> usize {
    let mut total = 0;
    for row in rows {
        for cell in row {
            total += cell as usize;
        }
    }
    total
}
"#;

#[test]
fn nested_loops_fires_at_depth_three_and_not_at_two() {
    assert_eq!(sole_rule(NESTED_FIRE), "oikosbot/nested-loops");
    assert_eq!(sole_rule(NESTED_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// busy-wait — a `loop`/`while` whose body never blocks
// ---------------------------------------------------------------------------

/// Spins on an atomic with no sleep, await, yield or blocking call.
const BUSY_WAIT_FIRE: &str = r#"fn spin(flag: &AtomicBool) -> usize {
    let mut spins = 0;
    loop {
        if flag.load(Ordering::Relaxed) {
            break;
        }
        spins += 1;
    }
    spins
}
"#;

/// The same loop with one sleep per iteration: it still polls, but it no longer
/// burns the CPU continuously, which is what the rule is about.
const BUSY_WAIT_SILENT: &str = r#"fn paced(flag: &AtomicBool) -> usize {
    let mut spins = 0;
    loop {
        if flag.load(Ordering::Relaxed) {
            break;
        }
        spins += 1;
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    spins
}
"#;

#[test]
fn busy_wait_fires_on_a_loop_that_never_blocks() {
    assert_eq!(sole_rule(BUSY_WAIT_FIRE), "oikosbot/busy-wait");
    assert_eq!(sole_rule(BUSY_WAIT_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// string-concat-in-loop — `s = s + "…"` inside a loop body
// ---------------------------------------------------------------------------

/// Builds a string with `+` inside the loop: quadratic allocation.
const CONCAT_FIRE: &str = r#"fn joined(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        out = out + "-";
    }
    out
}
"#;

/// The same loop using `push_str`, which amortises the allocation. Note the
/// loop still allocates once up front — the difference is per-iteration
/// reallocation, which is the thing being flagged.
const CONCAT_SILENT: &str = r#"fn pushed(parts: &[&str]) -> String {
    let mut out = String::new();
    for part in parts {
        out.push_str("-");
    }
    out
}
"#;

#[test]
fn string_concat_fires_only_on_concatenation_not_appending() {
    assert_eq!(sole_rule(CONCAT_FIRE), "oikosbot/string-concat-in-loop");
    assert_eq!(sole_rule(CONCAT_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// clone-in-loop — `.clone()` inside a loop body
// ---------------------------------------------------------------------------

/// Clones each element on every iteration.
const CLONE_FIRE: &str = r#"fn copied(items: &[String]) -> usize {
    let mut total = 0;
    for item in items {
        total += item.clone().len();
    }
    total
}
"#;

/// The same loop borrowing instead of copying.
const CLONE_SILENT: &str = r#"fn borrowed(items: &[String]) -> usize {
    let mut total = 0;
    for item in items {
        total += item.len();
    }
    total
}
"#;

#[test]
fn clone_in_loop_fires_on_the_clone_and_not_the_borrow() {
    assert_eq!(sole_rule(CLONE_FIRE), "oikosbot/clone-in-loop");
    assert_eq!(sole_rule(CLONE_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// unbuffered-io — `File::open`/`File::create` with no `BufReader`/`BufWriter`
// ---------------------------------------------------------------------------

/// Reads straight from the file handle.
const IO_FIRE: &str = r#"fn raw_read(path: &Path) -> std::io::Result<usize> {
    let mut count = 0;
    let mut handle = File::open(path)?;
    count += 1;
    Ok(count)
}
"#;

/// The same read wrapped in a `BufReader`. The `File::open` is still there —
/// the pair proves the rule looks for buffering, not for file access.
const IO_SILENT: &str = r#"fn buffered_read(path: &Path) -> std::io::Result<usize> {
    let mut count = 0;
    let mut handle = BufReader::new(File::open(path)?);
    count += 1;
    Ok(count)
}
"#;

#[test]
fn unbuffered_io_fires_without_a_bufwrapper_and_not_with_one() {
    assert_eq!(sole_rule(IO_FIRE), "oikosbot/unbuffered-io");
    assert_eq!(sole_rule(IO_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// large-allocation — a capacity literal above 1 MB
// ---------------------------------------------------------------------------

/// Allocates 2 MB up front.
const ALLOC_FIRE: &str = r#"fn big_buffer() -> Vec<u8> {
    let buffer: Vec<u8> = Vec::with_capacity(2_000_000);
    buffer
}
"#;

/// The same allocation at 1 KB: under the threshold, so no finding.
const ALLOC_SILENT: &str = r#"fn small_buffer() -> Vec<u8> {
    let buffer: Vec<u8> = Vec::with_capacity(1024);
    buffer
}
"#;

#[test]
fn large_allocation_fires_above_the_threshold_and_not_below() {
    assert_eq!(sole_rule(ALLOC_FIRE), "oikosbot/large-allocation");
    assert_eq!(sole_rule(ALLOC_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// redundant-allocation — five or more `.to_string()`/`.to_owned()` calls
// ---------------------------------------------------------------------------

/// Five stringifyings: at the threshold the rule fires.
const REDUNDANT_FIRE: &str = r#"fn quint(a: &str, b: &str, c: u16, d: &str, e: &str) {
    let _a = a.to_string();
    let _b = b.to_string();
    let _c = c.to_string();
    let _d = d.to_string();
    let _e = e.to_string();
}
"#;

/// Four of the same: below the threshold, deliberately, so the pair pins the
/// boundary rather than the presence of `.to_string()`.
const REDUNDANT_SILENT: &str = r#"fn quad(a: &str, b: &str, c: u16, d: &str) {
    let _a = a.to_string();
    let _b = b.to_string();
    let _c = c.to_string();
    let _d = d.to_string();
}
"#;

#[test]
fn redundant_allocation_fires_at_five_calls_and_not_four() {
    assert_eq!(sole_rule(REDUNDANT_FIRE), "oikosbot/redundant-allocation");
    assert_eq!(sole_rule(REDUNDANT_SILENT), "oikosbot/general");
}

// ---------------------------------------------------------------------------
// "No check performed" is not a pass (#81 acceptance rule)
// ---------------------------------------------------------------------------

/// A source with no functions in it. The analyzer must report *nothing* rather
/// than inventing a clean bill of health.
#[test]
fn source_without_functions_yields_no_findings() {
    let source = "// only a comment and a struct\nstruct Point {\n    x: u32,\n    y: u32,\n}\n";
    assert!(
        rules(source).is_empty(),
        "a file with no functions must not produce findings"
    );
}

/// Unparseable source. Tree-sitter is error-tolerant and will happily hand back
/// a partial tree, so this asserts the analyzer does not dress a parse failure
/// up as a finding on code it never understood.
#[test]
fn malformed_source_yields_no_findings() {
    let malformed = "fn broken( {\n    let x = ;\n";
    assert!(
        rules(malformed).is_empty(),
        "unparsed input must not be reported as analysed code"
    );

    let truncated = "fn half(a: u32) -> u32 {\n    let b = a +\n";
    assert!(
        rules(truncated).is_empty(),
        "truncated input must not be reported as analysed code"
    );
}

/// An extension the analyzer does not support. This is the clearest case of
/// "no check performed": it must be an error, never a silent success.
#[test]
fn unsupported_extension_is_an_error_not_a_pass() {
    let err = Language::detect(Path::new("notes.txt"))
        .expect_err("an unsupported extension must not be silently accepted");
    assert!(
        err.to_string().contains("Unsupported file extension"),
        "the error should name the reason, got: {err}"
    );
}
