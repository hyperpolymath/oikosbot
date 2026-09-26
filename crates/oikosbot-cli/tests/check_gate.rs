// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! End-to-end controls for advisory output and explicit no-check status.
//! Pattern-table estimates have no calibration receipts and must not gate.

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
/// which maps onto the heuristic `Sort` row. The body avoids every other
/// pattern — no `.clone()`, no `File::open`, no `vec![]`, no string `+`, no
/// `to_string` — so exactly one heuristic row prices the unit and nothing
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
fn recognized_regression_cannot_manufacture_a_gate() {
    let base = tree(&[("base.rs", &nested_loop_work(3))]);
    let head = tree(&[("head.rs", &nested_loop_work(4))]);

    let out = compare(base.path(), head.path(), &["--check"]);

    assert_eq!(
        out.status.code(),
        Some(2),
        "an unvalidated regression must return no-check (2); stdout: {}",
        stdout(&out)
    );
    let stdout = stdout(&out);
    assert!(
        stdout.contains("PARETO REGRESSION"),
        "expected a regression verdict; stdout: {stdout}"
    );
    assert!(
        stdout.contains("Confidence: Estimated"),
        "the drivers must retain heuristic confidence; stdout: {stdout}"
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
fn documentation_cannot_manufacture_evidence() {
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
        Some(2),
        "documentation cannot make estimates enforceable; stdout: {}",
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
        Some(2),
        "heuristic estimates return no-check rather than pass; stdout: {}",
        stdout(&out)
    );
    assert!(
        stderr(&out).contains("NOT enforced"),
        "a gate that cannot fire must say so unmissably; stderr: {}",
        stderr(&out)
    );
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_oikosbot"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn json_stdout_is_json_and_text_output_is_written() {
    let dir = tree(&[("clean.rs", "fn clean() -> u32 { 1 }")]);
    let source = dir.path().join("clean.rs");
    let out = run(&["analyze", source.to_str().unwrap(), "--format", "json"]);
    assert!(out.status.success(), "{}", stderr(&out));
    let payload: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(payload.is_array());
    let file = dir.path().join("output.txt");
    let out = run(&[
        "analyze",
        source.to_str().unwrap(),
        "--output",
        file.to_str().unwrap(),
    ]);
    assert!(out.status.success());
    assert!(!fs::read_to_string(file).unwrap().is_empty());
}

#[test]
fn report_is_advisory_but_check_never_passes_on_estimates_in_any_format() {
    let dir = tree(&[("clean.rs", "fn clean() -> u32 { 1 }")]);
    for format in ["text", "json", "sarif"] {
        let out = run(&[
            "report",
            dir.path().to_str().unwrap(),
            "--format",
            format,
            "--eco-threshold",
            "100",
        ]);
        assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
        let out = run(&["check", dir.path().to_str().unwrap(), "--format", format]);
        assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    }
}

#[test]
fn empty_invalid_excluded_and_unsupported_are_not_successful_scans() {
    for files in [
        vec![],
        vec![("broken.rs", "fn broken( {")],
        vec![("unsupported.txt", "hello")],
        vec![
            ("clean.rs", "fn clean() {}"),
            (".oikos.yml", "exclude: ['**/*.rs']"),
        ],
        vec![
            ("clean.rs", "fn clean() {}"),
            (".oikos.yml", "analysis:\n  languages: [typescript]"),
        ],
        vec![
            ("clean.rs", "fn clean() {}"),
            (".oikos.yml", "exclude: ['[']"),
        ],
        vec![("clean.rs", "fn clean() {}"), (".oikos.yml", "analysis: [")],
    ] {
        let dir = tree(&files);
        let out = run(&["report", dir.path().to_str().unwrap()]);
        assert_eq!(out.status.code(), Some(2), "{}", stderr(&out));
    }
}

#[test]
fn silence_firing_and_taxonomy_survive_the_cli_sarif_boundary() {
    let clean = tree(&[("clean.rs", "fn clean() -> u32 { 1 }")]);
    let firing = tree(&[("loops.rs", &nested_loop_work(3))]);
    for (dir, count) in [(&clean, 0), (&firing, 1)] {
        let out = run(&["report", dir.path().to_str().unwrap(), "--format", "sarif"]);
        assert!(out.status.success(), "{}", stderr(&out));
        let log: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        let findings = log["runs"][0]["results"].as_array().unwrap();
        assert_eq!(findings.len(), count);
        if count == 1 {
            assert_eq!(findings[0]["ruleId"], "oikosbot/nested-loops");
            assert_eq!(findings[0]["properties"]["taxonomy"]["intent"], "wish");
            assert_eq!(
                findings[0]["properties"]["taxonomy"]["locus"],
                "externalities"
            );
            assert!(findings[0].get("fixes").is_none());
        }
    }
}

#[cfg(unix)]
#[test]
fn directory_scan_does_not_follow_external_file_symlinks() {
    let outside = tree(&[("source.rs", &nested_loop_work(3))]);
    let root = tree(&[]);
    std::os::unix::fs::symlink(
        outside.path().join("source.rs"),
        root.path().join("link.rs"),
    )
    .unwrap();
    assert_eq!(
        run(&["report", root.path().to_str().unwrap()])
            .status
            .code(),
        Some(2)
    );
}

#[test]
fn requested_unavailable_security_is_not_silent_success() {
    let dir = tree(&[("clean.rs", "fn clean() {}")]);
    let out = run(&["report", dir.path().to_str().unwrap(), "--security"]);
    assert_eq!(out.status.code(), Some(2));
    assert!(stderr(&out).contains("unavailable"));
}

#[test]
fn requested_missing_policy_directory_is_not_silent_success() {
    let dir = tree(&[("clean.rs", "fn clean() {}")]);
    let missing = dir.path().join("missing-policies");
    let out = run(&[
        "report",
        dir.path().to_str().unwrap(),
        "--policy-dir",
        missing.to_str().unwrap(),
    ]);
    assert_eq!(out.status.code(), Some(2));
}
