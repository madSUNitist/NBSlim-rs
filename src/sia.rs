//! SIA (Structure Induction Algorithm) – finds all maximal translatable patterns.

use std::collections::HashMap;
use std::collections::HashSet;


/// Finds all maximal translatable patterns (MTPs) in a 2‑D point set using the SIA algorithm.
///
/// SIA computes, for every non‑zero translation vector `v`, the set of start points `p`
/// such that both `p` and `p + v` belong to the dataset. This set is the maximal
/// translatable pattern for `v`. The algorithm groups all ordered point pairs `(p, q)`
/// by the vector `q - p`, then collects the starting points for each vector.
///
/// # Arguments
/// * `dataset` - A reference to a vector of `(tick, pitch)` points with non‑negative coordinates.
/// * `restrict_dpitch_zero` - If `true`, only vectors with zero pitch difference are kept
///   (i.e., purely temporal translations). This restricts patterns to horizontal repetition.
///
/// # Returns
/// A `HashMap` mapping each translation vector `(dtick, dpitch)` to a `Vec` of start points
/// that form the maximal translatable pattern for that vector. Vectors that have fewer than
/// two start points are omitted, as a pattern must have at least one translation.
///
/// # Notes
/// - The zero vector `(0, 0)` is always excluded.
/// - The returned start points for each vector are sorted by `(tick, pitch)`.
/// - Complexity: `O(n²)` time and memory in the worst case, where `n` is the number of points.
///
/// # Examples
/// ```
/// use nbslim::find_mtps;
///
/// let points = vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1)];
/// let mtps = find_mtps(&points, false);
/// // mtps contains patterns for horizontal and diagonal repetitions.
/// ```
pub fn find_mtps(
    dataset: &Vec<(u32, u32)>, 
    restrict_dpitch_zero: bool
) -> HashMap<(i32, i32), Vec<(u32, u32)>> {
    let points = dataset;
    let n = points.len();

    // Online grouping using HashMap to avoid storing all O(n²) pairs
    let mut groups: HashMap<(i32, i32), HashSet<(u32, u32)>> = HashMap::new();

    for i in 0..n {
        let (ti, pi) = points[i];
        for j in 0..i {            
            let (tj, pj) = points[j];
            // Compute vector (i32 range, differences may be negative)
            let dx = ti as i32 - tj as i32;
            let dy = pi as i32 - pj as i32;
            if restrict_dpitch_zero && dy != 0 {
                continue;
            }
            // Insert into group with pj as the starting point
            groups
                .entry((dx, dy))
                .or_insert_with(HashSet::new)
                .insert((tj, pj));
            groups
                .entry((-dx, -dy))
                .or_insert_with(HashSet::new)
                .insert((ti, pi));
        }
    }

    // Collect results, filtering out zero vector and groups with fewer than 2 start points
    let mut result = HashMap::new();
    for (vec, start_set) in groups {
        if vec == (0, 0) {
            continue;
        }
        if start_set.len() < 2 {
            continue;
        }
        // Convert to sorted Vec
        let mut start_points: Vec<_> = start_set.into_iter().collect();
        start_points.sort();                 // sort by (tick, pitch)
        result.insert(vec, start_points);
    }
    result
}