//! # Sweepline-based pattern matching and translation equivalence construction
//!
//! This module implements a sweepline algorithm for exact translation matching between
//! a **sorted** dataset of points (tick, pitch) and a sorted pattern. It is used to discover
//! translational equivalences (TECs) by first finding maximal translatable patterns (MTPs)
//! and then verifying all translations that embed the pattern into the dataset.

use crate::tec::TranslationalEquivalence;
use crate::sia::find_mtps;
use std::collections::HashSet;

/// Finds all translation vectors `w` such that `pattern + w` is entirely contained in `dataset`.
///
/// **IMPORTANT**: `dataset` MUST be lexicographically sorted by `(tick, pitch)` in ascending order.
/// The same applies to `pattern`. Passing an unsorted slice leads to incorrect results.
///
/// Implements Ukkonen et al. Algorithm 1 (sweepline exact matching).
///
/// # Arguments
/// * `dataset` - Reference to a **lexicographically sorted** slice of points `(tick, pitch)`.
/// * `pattern` - Reference to a **lexicographically sorted** slice of points forming the pattern.
/// * `restrict_dpitch_zero` - If `true`, only translations with zero pitch difference are returned.
///
/// # Returns
/// A `HashSet` of non‑zero translation vectors `(dtick, dpitch)` that map the pattern into the dataset.
///
/// # Panics
/// In debug builds, this function will panic if `dataset` or `pattern` are not sorted.
pub fn exact_match_pattern(
    dataset: &[(u32, u32)],
    pattern: &[(u32, u32)],
    restrict_dpitch_zero: bool,
) -> HashSet<(i32, i32)> {
    debug_assert!(
        dataset.windows(2).all(|w| w[0] <= w[1]),
        "dataset must be lexicographically sorted"
    );
    debug_assert!(
        pattern.windows(2).all(|w| w[0] <= w[1]),
        "pattern must be lexicographically sorted"
    );

    let m = pattern.len();
    let n = dataset.len();
    if m < 2 || n < m {
        return HashSet::new();
    }

    // Pointers q[0..m] (0‑based for simplicity) – holds next candidate index in dataset.
    let mut q = vec![0usize; m];
    let p0 = pattern[0];
    let mut translators = HashSet::new();

    // Main loop: only consider dataset points that can possibly be the image of pattern[0]
    for j in 0..=(n - m) {
        let (tj, pj) = dataset[j];
        if restrict_dpitch_zero && pj != p0.1 {
            continue;
        }

        let f = (tj as i32 - p0.0 as i32, pj as i32 - p0.1 as i32);
        if f == (0, 0) || (restrict_dpitch_zero && f.1 != 0) {
            continue;
        }

        // q[0] should be at least j (already satisfied by j itself, but we take max with previous)
        q[0] = q[0].max(j);
        let mut matched = true;

        for idx in 1..m {
            // target point = pattern[idx] + f
            let target = (
                (pattern[idx].0 as i32 + f.0) as u32,
                (pattern[idx].1 as i32 + f.1) as u32,
            );
            // Advance pointer to the first dataset point >= target
            let mut ptr = q[idx].max(q[idx - 1]); // never go backwards
            while ptr < n && dataset[ptr] < target {
                ptr += 1;
            }
            q[idx] = ptr;
            if ptr >= n || dataset[ptr] != target {
                matched = false;
                break;
            }
        }

        if matched {
            translators.insert(f);
        }
    }

    translators
}

/// Builds translational equivalences (TECs) using sweepline both for MTP discovery and exact matching.
///
/// This function first calls `find_mtps` to obtain a set of maximal translatable patterns (MTPs)
/// from the dataset. For each such pattern (except the trivial zero translation), it then computes
/// all translation vectors that map the pattern into the dataset using `exact_match_pattern`.
/// The resulting `(pattern, translation_set)` pairs are wrapped into `TranslationalEquivalence`
/// structures and returned as a vector.
///
/// # Arguments
/// * `dataset` - A **lexicographically sorted** vector of points `(tick, pitch)`.
/// * `restrict_dpitch_zero` - If `true`, only translations with zero pitch difference are considered.
///
/// # Returns
/// A vector of `TranslationalEquivalence` objects, each containing a pattern and the set of
/// translations that embed it into the dataset.
///
/// # Notes
/// The `find_mtps` function is expected to return a collection of `(translation, pattern)` pairs.
/// Patterns with fewer than 2 points are skipped because they cannot form a meaningful equivalence.
pub fn build_tecs_from_mtps(
    dataset: &Vec<(u32, u32)>,
    restrict_dpitch_zero: bool,
) -> Vec<TranslationalEquivalence> {
    let mtps = find_mtps(dataset, restrict_dpitch_zero);
    let mut tecs = Vec::new();

    for (v, pattern) in mtps {
        if v == (0, 0) || pattern.len() < 2 {
            continue;
        }
        let translators = exact_match_pattern(dataset, &pattern, restrict_dpitch_zero);
        if !translators.is_empty() {
            tecs.push(TranslationalEquivalence::new(pattern, translators, None));
        }
    }
    tecs
}