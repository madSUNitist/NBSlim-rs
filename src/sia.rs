use std::collections::HashMap;
use std::collections::HashSet;


/// Finds all maximal translatable patterns (MTPs) in a 2‑D point set using the SIA algorithm.
///
/// SIA (Structure Induction Algorithm) computes, for every non‑zero translation vector `v`,
/// the set of start points `p` such that both `p` and `p + v` belong to the dataset.
/// This set is the maximal translatable pattern for `v`. The algorithm groups all
/// ordered point pairs `(p, q)` by the vector `q - p`, then collects the starting points
/// for each vector.
///
/// # Arguments
/// * `dataset` - A reference to a vector of `(tick, pitch)` points with non‑negative coordinates.
/// * `restrict_dpitch_zero` - If `true`, only vectors with zero pitch difference are kept
///   (i.e., purely temporal translations). This restricts patterns to horizontal repetition.
///
/// # Returns
/// A vector of `(vector, pattern)` entries, where:
/// - `vector` is `(dtick, dpitch)` – the translation vector.
/// - `pattern` is a sorted list of points that form the MTP for that vector.
///
/// The zero vector `(0, 0)` is always excluded. Vectors that map fewer than two points
/// (i.e., pattern size < 2) are also omitted because a pattern must have at least one
/// translation to be translatable.
///
/// # Complexity
/// - Time: `O(n²)` in the worst case, where `n` is the number of points.
/// - Memory: `O(n²)` in the worst case (stores a set of start points for each distinct vector).
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