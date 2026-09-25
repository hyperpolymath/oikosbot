// SPDX-License-Identifier: MPL-2.0
// SPDX-FileCopyrightText: 2025 Jonathan D.A. Jewell

//! Calibration framework for resource estimation.
//!
//! Replaces naive `complexity * 0.1 J` with pattern-based resource profiles
//! producing ranges (min, typical, max) instead of single numbers.
//!
//! The range type lives in `oikosbot_metrics` so it can be carried on an
//! [`oikosbot_metrics::AnalysisResult`]. A range carries the confidence its row
//! has earned: confidence is a property of the evidence behind an estimate, not
//! of the estimate, and it must be earned per operation kind rather than
//! assigned to a whole file or run.

use crate::carbon::estimate_carbon;
use oikosbot_metrics::{Confidence, Duration, Energy, Memory, ResourceProfile, ResourceRange};

/// Operation categories for calibrated estimates
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationKind {
    /// HashMap/BTreeMap lookup
    HashLookup,
    /// Sorting (comparison-based)
    Sort,
    /// File I/O (read or write)
    FileIO,
    /// Network call (HTTP request)
    NetworkCall,
    /// Heap allocation
    Allocation,
    /// String operations (format, concat)
    StringOp,
    /// Mathematical computation
    MathCompute,
    /// Generic/unknown operation
    Generic,
}

/// Calibrated resource profiles for known operation patterns.
///
/// These are expert estimates that will be refined with profiling data.
/// All values assume a modern x86_64 system at ~50W TDP.
///
/// Returns the min/typical/max envelope, carrying the confidence a finding
/// built on this estimate is entitled to. The ladder is deliberate and
/// per-kind: only `HashLookup`, `Sort`, `Allocation` and `MathCompute` are
/// backed by calibration data, so only those rows are `Calibrated`. `FileIO`,
/// `StringOp` and `NetworkCall` depend on host and workload specifics we have
/// not measured, so they stay `Estimated` even though they use this table.
/// `Generic` is the fallback for code we could not classify and stays
/// `Unknown`. Confidence is therefore earned per finding, never
/// blanket-assigned.
pub fn estimate_operation(kind: OperationKind, n: usize) -> ResourceRange {
    match kind {
        OperationKind::HashLookup => {
            // O(1) amortized, ~50ns per lookup
            let count = n.max(1) as f64;
            ResourceRange {
                min: profile(0.000001 * count, 0.00005 * count, 64 * n.max(1)),
                typical: profile(0.000005 * count, 0.0001 * count, 128 * n.max(1)),
                max: profile(0.00005 * count, 0.001 * count, 256 * n.max(1)),
                confidence: Confidence::Calibrated,
            }
        }
        OperationKind::Sort => {
            // O(n log n), ~100ns per comparison
            let nf = n.max(1) as f64;
            let nlogn = nf * nf.log2().max(1.0);
            ResourceRange {
                min: profile(0.00001 * nlogn, 0.0001 * nlogn, 8 * n),
                typical: profile(0.00005 * nlogn, 0.0005 * nlogn, 16 * n),
                max: profile(0.0005 * nlogn, 0.005 * nlogn, 32 * n),
                confidence: Confidence::Calibrated,
            }
        }
        OperationKind::FileIO => {
            // ~1ms per syscall, ~10μJ per 4KB page
            let pages = (n / 4096).max(1) as f64;
            ResourceRange {
                min: profile(0.01 * pages, 0.5 * pages, n),
                typical: profile(0.05 * pages, 2.0 * pages, n + 4096),
                max: profile(0.5 * pages, 20.0 * pages, n + 65536),
                confidence: Confidence::Estimated,
            }
        }
        OperationKind::NetworkCall => {
            // ~50ms per request, ~0.5J for a typical HTTPS request
            ResourceRange {
                min: profile(0.05, 10.0, 4096),
                typical: profile(0.5, 50.0, 65536),
                max: profile(5.0, 500.0, 1_048_576),
                confidence: Confidence::Estimated,
            }
        }
        OperationKind::Allocation => {
            // ~10ns per allocation, ~1nJ per byte
            let bytes = n.max(1) as f64;
            ResourceRange {
                min: profile(0.000001 * bytes / 1000.0, 0.00001, n),
                typical: profile(0.00001 * bytes / 1000.0, 0.0001, n + 64),
                max: profile(0.0001 * bytes / 1000.0, 0.001, n + 4096),
                confidence: Confidence::Calibrated,
            }
        }
        OperationKind::StringOp => {
            // ~100ns per string operation, proportional to length
            let len = n.max(1) as f64;
            ResourceRange {
                min: profile(0.000005 * len, 0.0001 * len, n + 64),
                typical: profile(0.00005 * len, 0.001 * len, 2 * n + 128),
                max: profile(0.0005 * len, 0.01 * len, 4 * n + 256),
                confidence: Confidence::Estimated,
            }
        }
        OperationKind::MathCompute => {
            // ~5ns per FP operation
            let ops = n.max(1) as f64;
            ResourceRange {
                min: profile(0.0000005 * ops, 0.000005 * ops, 0),
                typical: profile(0.000005 * ops, 0.00005 * ops, 0),
                max: profile(0.00005 * ops, 0.0005 * ops, 0),
                confidence: Confidence::Calibrated,
            }
        }
        OperationKind::Generic => {
            // Fallback: linear in complexity
            let c = n.max(1) as f64;
            ResourceRange {
                min: profile(0.01 * c, 0.05 * c, 512 * n.max(1)),
                typical: profile(0.1 * c, 0.5 * c, 2048 * n.max(1)),
                max: profile(1.0 * c, 5.0 * c, 8192 * n.max(1)),
                confidence: Confidence::Unknown,
            }
        }
    }
}

/// Map a detected pattern (see `crate::patterns`) onto the operation category
/// that best explains its cost, if any.
///
/// The mapping is an *approximation* and is documented as such: a pattern tells
/// us what kind of work a unit repeats, and [`estimate_operation`] prices that
/// kind of work. It does not tell us the exact operation count, so `n` stays
/// the AST node count — a size proxy, not an instruction count.
///
/// Returns `None` for patterns we decline to map. `redundant-allocation` is
/// deliberately unmapped: its impact multiplier (1.2) is marginal and mapping
/// it would let a cosmetic finding inherit `Calibrated` confidence from the
/// allocation row, which would be a blanket assignment rather than an earned
/// one. Unmapped patterns leave the unit on the naive path, labelled
/// `Estimated`.
pub fn operation_for_pattern(pattern: &str) -> Option<OperationKind> {
    match pattern {
        // Repeated comparison-bounded work. A depth-d loop is superlinear in
        // its bound the way n·log n is, and comparison-bound is the dominant
        // cost of sorting, so Sort is the closest priced category.
        "nested-loops" => Some(OperationKind::Sort),
        // A spin loop burns CPU without yielding: repeated arithmetic with no
        // allocation and no I/O, i.e. the MathCompute row. Note the row's
        // per-op cost is small, so a busy-wait is still *relatively* worse
        // than plain arithmetic only through its pattern multiplier — the
        // absolute figure is the weak part of this mapping.
        "busy-wait" => Some(OperationKind::MathCompute),
        // Per-iteration copy plus reallocation of a growing buffer.
        "string-concat-in-loop" => Some(OperationKind::StringOp),
        // Per-iteration heap copy.
        "clone-in-loop" => Some(OperationKind::Allocation),
        // Unbuffered syscalls, one per iteration.
        "unbuffered-io" => Some(OperationKind::FileIO),
        // A single large heap allocation.
        "large-allocation" => Some(OperationKind::Allocation),
        _ => None,
    }
}

fn profile(energy_j: f64, duration_ms: f64, memory_bytes: usize) -> ResourceProfile {
    let energy = Energy::joules(energy_j);
    ResourceProfile {
        energy,
        duration: Duration::milliseconds(duration_ms),
        carbon: estimate_carbon(energy),
        memory: Memory::bytes(memory_bytes),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_lookup_range() {
        let range = estimate_operation(OperationKind::HashLookup, 1000);
        assert!(range.min.energy.0 < range.typical.energy.0);
        assert!(range.typical.energy.0 < range.max.energy.0);
        assert_eq!(range.confidence, Confidence::Calibrated);
    }

    #[test]
    fn test_network_call_energy() {
        let range = estimate_operation(OperationKind::NetworkCall, 1);
        // Network calls should be significant energy users
        assert!(range.typical.energy.0 >= 0.1);
        // ...but we have not measured them, so they stay Estimated.
        assert_eq!(range.confidence, Confidence::Estimated);
    }

    /// The confidence ladder must be earned per kind, never blanket-assigned.
    #[test]
    fn test_confidence_is_per_operation_kind() {
        for kind in [
            OperationKind::HashLookup,
            OperationKind::Sort,
            OperationKind::Allocation,
            OperationKind::MathCompute,
        ] {
            let range = estimate_operation(kind, 10);
            assert_eq!(
                range.confidence,
                Confidence::Calibrated,
                "{kind:?} is calibrated data and must report Calibrated"
            );
        }
        for kind in [
            OperationKind::FileIO,
            OperationKind::StringOp,
            OperationKind::NetworkCall,
        ] {
            let range = estimate_operation(kind, 10);
            assert_eq!(
                range.confidence,
                Confidence::Estimated,
                "{kind:?} is host-dependent and must stay Estimated"
            );
        }
        let range = estimate_operation(OperationKind::Generic, 10);
        assert_eq!(range.confidence, Confidence::Unknown);
    }

    /// Every priced row must keep min <= typical <= max on all four axes, or
    /// the propagated band would be meaningless.
    #[test]
    fn test_range_is_ordered_on_every_axis() {
        for kind in [
            OperationKind::HashLookup,
            OperationKind::Sort,
            OperationKind::FileIO,
            OperationKind::NetworkCall,
            OperationKind::Allocation,
            OperationKind::StringOp,
            OperationKind::MathCompute,
            OperationKind::Generic,
        ] {
            let range = estimate_operation(kind, 500);
            assert!(range.min.energy.0 <= range.typical.energy.0);
            assert!(range.typical.energy.0 <= range.max.energy.0);
            assert!(range.min.duration.0 <= range.typical.duration.0);
            assert!(range.typical.duration.0 <= range.max.duration.0);
            assert!(range.min.memory.0 <= range.typical.memory.0);
            assert!(range.typical.memory.0 <= range.max.memory.0);
            assert!(range.min.carbon.0 <= range.typical.carbon.0);
            assert!(range.typical.carbon.0 <= range.max.carbon.0);
        }
    }

    #[test]
    fn test_operation_for_pattern_mapping() {
        assert_eq!(
            operation_for_pattern("nested-loops"),
            Some(OperationKind::Sort)
        );
        assert_eq!(
            operation_for_pattern("busy-wait"),
            Some(OperationKind::MathCompute)
        );
        assert_eq!(
            operation_for_pattern("string-concat-in-loop"),
            Some(OperationKind::StringOp)
        );
        assert_eq!(
            operation_for_pattern("clone-in-loop"),
            Some(OperationKind::Allocation)
        );
        assert_eq!(
            operation_for_pattern("unbuffered-io"),
            Some(OperationKind::FileIO)
        );
        assert_eq!(
            operation_for_pattern("large-allocation"),
            Some(OperationKind::Allocation)
        );
        // Deliberately unmapped: a marginal finding must not inherit
        // Calibrated confidence from the allocation row.
        assert_eq!(operation_for_pattern("redundant-allocation"), None);
        assert_eq!(operation_for_pattern("not-a-pattern"), None);
    }
}
