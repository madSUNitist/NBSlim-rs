use std::collections::{HashMap, HashSet};

use crate::tec::TranslationalEquivalence;


const PITCH_BITS: u32 = 14;
const OFFSET: u32 = 1200;

/// Converts a list of MIDI-like note events into 2D points for compression,
/// along with a mapping from each point back to its original note data.
///
/// The y-coordinate is encoded as:
///   `(instrument << PITCH_BITS) | (pitch_in_cents + OFFSET)`
/// This packs instrument ID and pitch offset into a single `u32` value.
/// The x-coordinate is simply the tick (time) value.
///
/// # Arguments
/// * `notes` - A slice of tuples where each tuple represents a note event:
///   `(tick, layer, instrument, key, velocity, panning, pitch)`
///
///   - `tick`:      time in ticks (`usize`)
///   - `layer`:     layer index (`usize`)
///   - `instrument`: instrument ID (`usize`)
///   - `key`:       MIDI key number (0–127, `usize`)
///   - `velocity`:  note velocity (`usize`)
///   - `panning`:   panning value (`i64`)
///   - `pitch`:     pitch bend offset in cents (`i64`)
///
/// # Returns
/// A tuple `(points, mapping)` where:
/// - `points`: `Vec<(u32, u32)>` – the generated 2D points `(tick, encoded_y)`.
/// - `mapping`: `HashMap<(u32, u32), (usize, usize, usize, usize, usize, i64, i64)>`
///   that maps each point back to the original note data.
pub fn notes_to_points(
    notes: &Vec<(usize, usize, usize, usize, usize, i64, i64)>
) -> (Vec<(u32, u32)>, HashMap<(u32, u32), (usize, usize, usize, usize, usize, i64, i64)>) {
    let mut points: Vec<_> = Vec::new();
    let mut mapping = HashMap::new();

    for &(tick, layer, instrument, key, velocity, panning, pitch) in notes {
        let pitch_cents = ((key * 100) as i64 + pitch) as u32;
        let pitch_off = pitch_cents + OFFSET;
        let encoded_y = ((instrument as u32) << PITCH_BITS) | pitch_off;
        let point = (tick as u32, encoded_y);
        points.push(point);
        mapping.insert(point, (tick, layer, instrument, key, velocity, panning, pitch));
    }

    (points, mapping)
}

/// Merge all TECs that satisfy the filter into a single TEC containing all their coverage points.
///
/// The merged TEC has no translators (i.e., it is not a true TEC) and is used only for
/// reconstruction. It helps avoid many tiny TECs that would each occupy a small layer block.
pub fn merge_tecs(
    tecs: Vec<TranslationalEquivalence>, 
    filter: fn(&TranslationalEquivalence) -> bool
) -> Vec<TranslationalEquivalence> {
    let mut to_merge = Vec::new();
    let mut to_keep  = Vec::new();
    for tec in tecs {
        if filter(&tec) {
            to_merge.push(tec);
        } else {
            to_keep.push(tec);
        }
    }

    if to_merge.is_empty() {
        return to_keep;
    }

    // Collect all points from the TECs to be merged
    let mut merged_points = HashSet::new();
    for tec in to_merge {
        merged_points.extend(tec.coverage());
    }
    
    if merged_points.is_empty() {
        // No points (shouldn't happen) – just return empty Vec
        return vec![]
    }

    // Sort points to get a deterministic pattern
    
    let mut sorted_points = Vec::from_iter(merged_points);
    sorted_points.sort();
    let merged_tec = TranslationalEquivalence::new(
        sorted_points,
        HashSet::new(), 
        None
    );
    
    to_keep.push(merged_tec);
    to_keep
}


/// Calculate compression statistics for a list of TECs.
/// 
/// Returns a tuple: (original_count, encoded_units, compression_ratio):
pub fn compression_stats(
    tecs: &Vec<TranslationalEquivalence>, 
    original_points: &Vec<(u32, u32)>
) -> (usize, usize, f64) {
    fn _count_units(tecs: &Vec<TranslationalEquivalence>) -> usize {
        let mut units = 0usize;
        for tec in tecs {
            units += tec.translators.len();
            units += if tec.sub_tecs.is_empty() {
                tec.pattern.len()
            } else {
                _count_units(&tec.sub_tecs)
            };
        }

        units
    }

    let original = original_points.len();
    let encoded = _count_units(tecs);
    let ratio = if encoded > 0 {
        original as f64 / encoded as f64
    } else {
        0.0
    };

    (original, encoded, ratio)
}

/// Reconstructs the original set of points from a list of TECs.
///
/// The reconstruction takes the coverage (pattern + all translated copies)
/// of each TEC, merges them, and returns the sorted list of unique points.
/// This is the inverse operation of a lossless compression pipeline that
/// decomposes a point set into TECs.
///
/// # Arguments
/// * `tecs` - A vector of `TranslationalEquivalence` objects, typically produced
///            by a compression algorithm (e.g., COSIATEC, SIATECCompress).
///
/// # Returns
/// A sorted `Vec<(u32, u32)>` containing all points that were covered by the
/// input TECs, with duplicates removed.
pub fn rebuild(tecs: Vec<TranslationalEquivalence>) -> Vec<(u32, u32)> {
    let mut all_points = HashSet::new();
    for tec in tecs {
        all_points.extend(tec.coverage());
    }
    let mut result: Vec<_> = all_points.into_iter().collect();
    result.sort();
    result
}