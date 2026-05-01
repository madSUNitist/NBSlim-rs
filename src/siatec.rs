//! SIATEC – builds translational equivalence classes (TECs) from SIA patterns.

use std::collections::HashSet;

use crate::sia::find_mtps;
use crate::tec::TranslationalEquivalence;

/// Builds translational equivalence classes (TECs) for every maximal translatable pattern (MTP)
/// found in the dataset by the SIA algorithm.
///
/// For each MTP (represented by a translation vector `v` and its pattern `P`), this function
/// determines all translation vectors `w` such that `P + w` is entirely contained in the dataset.
/// These `w`s become the TEC’s translator set. The TEC is then stored as `⟨P, translators⟩`.
///
/// # Arguments
/// * `dataset` - A reference to the vector of points `(tick, pitch)`.
/// * `restrict_dpitch_zero` - If `true`, only translation vectors with zero pitch difference
///   are accepted (purely temporal shifts).
///
/// # Returns
/// A vector of `TranslationalEquivalence` objects, one per distinct MTP (i.e., per non‑zero
/// translation vector `v`). Patterns that have no translators (other than the trivial zero vector)
/// are omitted.
///
/// # Algorithmic notes
/// - For each MTP pattern `P`, candidate translators are all vectors from the first pattern point
///   `p0` to any point in the dataset. Only those that map the whole `P` into the dataset are kept.
/// - Complexity: `O(m · n)` where `m` is the number of MTPs and `n` is the dataset size.
///   In the worst case `m = O(n²)`.
///
/// # Examples
/// ```
/// use nbslim::build_tecs_from_mtps;
///
/// let points = vec![(0, 0), (1, 0), (2, 0), (3, 0)];
/// let tecs = build_tecs_from_mtps(&points, false);
/// // The pattern [0,0] and [1,0] may form a TEC with translator (1,0).
/// ```
pub fn build_tecs_from_mtps(dataset: &Vec<(u32, u32)>, restrict_dpitch_zero: bool) -> Vec<TranslationalEquivalence> {
    // Store dataset as i64 HashSet for easy containment checks
    let points_set: HashSet<(u32, u32)> = dataset.iter().copied().collect();
    let mtps = find_mtps(dataset, restrict_dpitch_zero); // returns Vec<((i32,i32), Vec<(u32,u32)>)>
    let mut tecs = Vec::new();

    for (v, start_pts) in mtps {
        if v == (0, 0) {
            continue;
        }
        // pattern is the set of points that can be translated by v
        let pattern = start_pts; // already sorted and deduped by sia
        if pattern.is_empty() {
            continue;
        }

        let p0 = pattern[0]; // (u32, u32)
        // Collect all candidate translation vectors w = q - p0  (signed differences)
        let candidates: HashSet<(i32, i32)> = points_set
            .iter()
            .map(|&(qx, qy)| ((qx as i64 - p0.0 as i64) as i32, (qy as i64 - p0.1 as i64) as i32))
            .collect();

        let mut translators = HashSet::<(i32, i32)>::new();
        for w in candidates {
            if w == (0, 0) {
                continue;
            }
            if restrict_dpitch_zero && w.1 != 0 {
                continue;
            }
            // Check if pattern + w is fully contained in dataset
            let ok = pattern.iter().all(|&(px, py)| {
                let tx = (px as i32 + w.0) as u32;
                let ty = (py as i32 + w.1) as u32;
                points_set.contains(&(tx, ty))
            });
            if ok {
                translators.insert(w);
            }
        }

        if !translators.is_empty() {
            let tec = TranslationalEquivalence::new(pattern, translators, None);
            tecs.push(tec);
        }
    }

    tecs
}