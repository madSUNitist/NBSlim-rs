//! RECURSIA: recursive compression of TEC patterns.

use crate::cosiatec::cosiatec_compress;
use crate::tec::TranslationalEquivalence;


/// RECURSIA (Recursive SIA) applied to the COSIATEC compression algorithm.
///
/// This function first obtains a standard COSIATEC cover of the dataset, then
/// recursively compresses the `pattern` of each resulting TEC if the pattern
/// contains at least `min_pattern_size` points. The recursive compression uses
/// the same algorithm (RECURSIA‑COSIATEC) and stores the result in the TEC’s
/// `sub_tecs` field. After recursion, the original `pattern` is cleared because
/// the compressed representation is now held in `sub_tecs`.
///
/// The recursion continues until patterns shrink below the size threshold.
/// This hierarchical encoding can reveal nested repetitions and improve
/// overall compression factor.
///
/// # Arguments
/// * `dataset` - A reference to the set of points `(tick, pitch)` to compress.
/// * `restrict_dpitch_zero` - If `true`, only purely temporal translations are allowed.
/// * `min_pattern_size` - Minimum number of points a pattern must contain to
///   be considered for recursion. Patterns smaller than this are left unchanged
///   (their `sub_tecs` remains empty).
///
/// # Returns
/// A vector of top‑level `TranslationalEquivalence` objects. Each TEC may have
/// a non‑empty `sub_tecs` field containing the recursively compressed version
/// of its pattern. The original pattern points are cleared in such cases.
///
/// # Examples
/// ```
/// use nbslim::recursive_cosiatec_compress;
///
/// let points = vec![(0, 0), (1, 1), (2, 0), (3, 1)];
/// let tecs = recursive_cosiatec_compress(&points, true, 2);
/// // Some TECs may have non‑empty sub_tecs.
/// ```
pub fn recursive_cosiatec_compress(
    dataset: &Vec<(u32, u32)>,
    restrict_dpitch_zero: bool,
    min_pattern_size: usize,
) -> Vec<TranslationalEquivalence> {
    // 1. Obtain the top‑level COSIATEC cover (without recursion)
    let mut tecs = cosiatec_compress(dataset, restrict_dpitch_zero);

    // 2. Recursively compress the pattern of each TEC
    for tec in &mut tecs {
        if tec.pattern.len() >= min_pattern_size {
            tec.sub_tecs = recursive_cosiatec_compress(
                &tec.pattern, 
                restrict_dpitch_zero, 
                min_pattern_size
            );
            tec.pattern.clear();
        } else {
            tec.sub_tecs = vec![];
        }
    }
    tecs
}
