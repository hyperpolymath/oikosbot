// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! End-to-end proof that `oikosbot compare --check` can actually block.
//!
//! Before the calibration wiring every finding was `Estimated`, so `--check`
//! could only ever print the inert-enforcement warning: the gate could not
//! fail. These tests pin the behaviour that makes the trade-off doctrine real —
//!
//! * a regression whose driving objectives are calibrated **blocks**;
//! * the same regression without `--check` stays advisory;
//! * a documented trade-off is accepted;
//! * a heuristic (unrecognised) regression still refuses to block, and says so
//!   loudly rather than passing silently.
//!
//! Fixtures are sized so both sides land on a calibrated row: base is a
//! depth-3 loop nest, head the same nest one level deeper. Both are therefore
//! `Calibrated`, and the deeper nest is worse on every objective, so the
//! verdict is a genuine `Regression` with real drivers behind it.

use std::fs;
use std::path::Path;
use std::process::{Command, Output};
use tempfile::TempDir;

/// Write a throwaway source tree and return its handle.
fn tree(files: &[(&str, &str)]) -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    for (name, body) in files {
        fs::write(dir.path().join(name), body).expect("write fixture");
    }
    dir
}

/// A Rust function whose body is `depth` nested `for` loops.
///
/// Depth 3 is the smallest nest `detect_patterns` flags as `nested-loops`,
/// which maps onto the calibrated `Sort` row. The body avoids every other
/// pattern — no `.clone()`, no `File::open`, no `vec![]`, no string `+`, no
/// `to_string` — so exactly one calibrated row prices the unit and nothing
/// drags its confidence back to `Estimated`.
fn nested_loop_work(depth: usize) -> String {
    let mut body = String::from("fn deep_work(items: &[u32]) -> usize {\n    let mut total = 0;\n");
    for _ in 0..depth {
        body.push_str("    for i in 0..8 {\n");
    }
    body.push_str("        total += 1;\n");
    for _ in 0..depth {
        body.push_str("    }\n");
    }
    body.push_str("    total\n}\n");
    body
}

/// A function with no recognised pattern: `statements` straight-line
/// increments. Both sides of a comparison using this stay on the naive,
/// complexity-derived path and are therefore `Estimated`.
fn plain_work(statements: usize) -> String {
    let mut body = String::from("fn plain_work() -> usize {\n    let mut total = 0;\n");
    for i in 0..statements {
        body.push_str(&format!("    total += {};\n", i + 1));
    }
    body.push_str("    total\n}\n");
    body
}

fn compare(base: &Path, head: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oikosbot"))
        .arg("compare")
        .arg(base)
        .arg(head)
        .args(extra)
        .output()
        .expect("spawn oikosbot")
}

fn stdout(out: &Output) -> String {
    String::from_utf8_lossy(&out.stdout).to_string()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).to_string()
}

#[test]
fn undocumented_calibrated_regression_blocks_under_check() {
    let base = tree(&[("base.rs", &nested_loop_work(3))]);
    let head = tree(&[("head.rs", &nested_loop_work(4))]);

    let out = compare(base.path(), head.path(), &["--check"]);

    assert_eq!(
        out.status.code(),
        Some(1),
        "an undocumented calibrated regression must exit 1; stdout: {}",
        stdout(&out)
    );
    let stdout = stdout(&out);
    assert!(
        stdout.contains("PARETO REGRESSION"),
        "expected a regression verdict; stdout: {stdout}"
    );
    assert!(
        stdout.contains("Confidence: Calibrated"),
        "the drivers must be calibrated, not heuristic; stdout: {stdout}"
    );
}

#[test]
fn same_regression_is_advisory_without_check() {
    let base = tree(&[("base.rs", &nested_loop_work(3))]);
    let head = tree(&[("head.rs", &nested_loop_work(4))]);

    let out = compare(base.path(), head.path(), &[]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "without --check the run is advisory; stderr: {}",
        stderr(&out)
    );
    assert!(stdout(&out).contains("PARETO REGRESSION"));
}

#[test]
fn documented_regression_is_accepted_under_check() {
    let base = tree(&[("base.rs", &nested_loop_work(3))]);
    let head = tree(&[
        ("head.rs", &nested_loop_work(4)),
        (
            "pr.md",
            "Summary\n\nPareto-Trade-off: depth-4 nest kept for cache locality.\n",
        ),
    ]);
    // The CLI resolves --pr-body relative to its own working directory, so the
    // fixture must be addressed absolutely.
    let pr_body = head.path().join("pr.md");

    let out = compare(
        base.path(),
        head.path(),
        &["--check", "--pr-body", pr_body.to_str().unwrap()],
    );

    assert_eq!(
        out.status.code(),
        Some(0),
        "a documented trade-off must be accepted; stdout: {}",
        stdout(&out)
    );
}

#[test]
fn heuristic_regression_refuses_to_block_and_says_so() {
    let base = tree(&[("base.rs", &plain_work(2))]);
    let head = tree(&[("head.rs", &plain_work(8))]);

    let out = compare(base.path(), head.path(), &["--check"]);

    assert_eq!(
        out.status.code(),
        Some(0),
        "heuristic estimates advise, they do not block; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("NOT enforced"),
        "a gate that cannot fire must say so unmissably; stderr: {}",
        stderr(&out)
    );
}
