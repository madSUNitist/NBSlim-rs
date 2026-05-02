//! COSIATEC: greedy iterative compression using translational equivalence classes (TECs).

use std::collections::HashSet;

use crate::siatec::build_tecs_from_mtps;
use crate::sweepline::build_tecs_from_mtps as build_tecs_from_mtps_sweepline;
use crate::tec::{TranslationalEquivalence, tec_sort_key};

/// COSIATEC: a greedy, iterative compression algorithm based on translational equivalence classes (TECs).
///
/// The algorithm repeatedly runs SIATEC on the **remaining uncovered points**, selects the
/// “best” TEC according to the multi‑level comparison rules (compression ratio, compactness,
/// coverage size, pattern size, temporal width, bounding box area), adds it to the result list,
/// and removes its covered points from the remaining set. This process continues until all
/// points are covered. When no pattern can be found, the remaining points are encoded as
/// trivial single‑point TECs (empty translator set).
///
/// **Important**: After recursion, the `pattern` field of each top‑level TEC is
/// cleared (set to empty). The actual points of the pattern are moved into the
/// `sub_tecs` hierarchy. This is the standard behavior of RECURSIA: the pattern
/// is recursively decomposed and no longer stored at the top level.
///
/// # Arguments
/// * `dataset` - A reference to the full set of points `(tick, pitch)`. The algorithm works on
///   a copy of this set and does not modify the original data.
/// * `restrict_dpitch_zero` - If `true`, only translation vectors with zero pitch difference
///   are considered (purely temporal shifts). This restricts patterns to horizontal repetition.
/// * `sweepline_optimization` - Whether to use the sweepline optimized implementation.
///
/// # Returns
/// A `Vec` of `TranslationalEquivalence` objects that partition the input points (each point
/// belongs to exactly one TEC). Each TEC may have `sub_tecs` if recursion was applied later
/// (not used in this base function).
///
/// # Comparison details
/// When selecting the best TEC on a given iteration, the algorithm uses `tec_sort_key` with
/// the **remaining point set** (not the full dataset) to compute compactness and coverage.
/// This ensures that patterns are evaluated against the points that still need to be encoded.
///
/// # Complexity
/// In the worst case, building all TECs from the remaining points takes `O(k·n³)` time per
/// iteration (`n` = size of remaining set). The total number of iterations is at most the
/// number of points. Practical performance is acceptable for typical music datasets (up to a
/// few thousand points).
///
/// # Examples
/// ```
/// use nbslim::cosiatec_compress;
///
/// let points = vec![(0, 10), (1, 20), (2, 10), (3, 20)];
/// let tecs = cosiatec_compress(&points, true, true);
/// for tec in tecs {
///     println!("{}", tec.coverage().len());
/// }
/// ```
pub fn cosiatec_compress(
    dataset: &Vec<(u32, u32)>,
    restrict_dpitch_zero: bool,
    sweepline_optimization: bool
) -> Vec<TranslationalEquivalence> {
    let mut remaining: HashSet<(u32, u32)> = dataset.iter().copied().collect();
    let mut tec_list = Vec::new();

    while !remaining.is_empty() {
        // compute all TECs on the remaining points
        let mut pts_list: Vec<(u32, u32)> = remaining.iter().copied().collect();
        pts_list.sort();
        
        let all_tecs = if sweepline_optimization {
            build_tecs_from_mtps_sweepline(
                &pts_list, 
                restrict_dpitch_zero
            )
        } else {
            build_tecs_from_mtps(
                &pts_list, 
                restrict_dpitch_zero
            )
        };

        if all_tecs.is_empty() {
            // no more patterns -> output remaining as trivial TECs (single points)
            for p in remaining {
                let pattern = vec![p];
                let translators = HashSet::new();
                let tec = TranslationalEquivalence::new(pattern, translators, None);
                tec_list.push(tec);
            }
            break;
        }
        
        // select best TEC
        let best: TranslationalEquivalence = all_tecs.iter()
            .min_by_key(|tec| tec_sort_key(tec, &remaining)) // XXX: `remaining` OR `dataset`
            .unwrap()
            .clone();
        let best_coverage = best.coverage();
        tec_list.push(best);
        
        // remove covered points
        remaining = remaining.difference(&best_coverage).copied().collect();
    }

    tec_list
}