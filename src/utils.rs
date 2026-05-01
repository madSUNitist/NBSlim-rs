//! Utility functions for converting between note events and points, merging TECs,
//! computing compression statistics, and reconstructing original data.

use std::collections::{HashMap, HashSet};

use crate::tec::TranslationalEquivalence;


const PITCH_BITS: u32 = 14;
const OFFSET: i16 = 1200;

/// Converts a list of MIDI‑like note events into 2D points for compression,
/// along with a mapping from each point back to **all** original note data
/// that share that exact point (i.e., a multi‑set).
///
/// The y‑coordinate is encoded as:
///   `(instrument << PITCH_BITS) | (pitch_in_cents + OFFSET)`
/// This packs instrument ID and pitch offset into a single `u32` value.
/// The x‑coordinate is simply the tick (time) value.
///
/// # Arguments
/// * `notes` - A slice of tuples representing note events:
///   `(tick, layer, instrument, key, velocity, panning, pitch)`
///
/// # Returns
/// A tuple `(points, mapping)` where:
/// - `points`: `Vec<(u32, u32)>` – the generated 2D points `(tick, encoded_y)`,
///   with possible duplicates.
/// - `mapping`: `HashMap<(u32, u32), Vec<NoteData>>` mapping a point to all
///   original notes that produced it, in the order encountered.
///
/// # Examples
/// ```
/// use nbslim::utils::notes_to_points;
///
/// let notes = vec![(0, 0, 1, 60, 100, 0, 0)];
/// let (points, mapping) = notes_to_points(&notes);
/// assert_eq!(points, vec![(0, 23584)]);
/// assert_eq!(mapping.get(&(0, 23584)).unwrap()[0], (0, 0, 1, 60, 100, 0, 0));
/// ```
pub fn notes_to_points(
    notes: &Vec<(u32, u8, u8, u8, u8, i8, i16)>,
) -> (
    Vec<(u32, u32)>,
    HashMap<(u32, u32), Vec<(u32, u8, u8, u8, u8, i8, i16)>>,
) {
    let mut points = Vec::new();
    let mut mapping = HashMap::new();

    for &(tick, layer, instrument, key, velocity, panning, pitch) in notes {
        let pitch_cents = (key as i16) * 100 + pitch;
        let pitch_off = pitch_cents + OFFSET;
        let encoded_y = ((instrument as u32) << PITCH_BITS) | (pitch_off as u32);
        let point = (tick as u32, encoded_y);
        points.push(point);
        mapping
            .entry(point)
            .or_insert_with(Vec::new)
            .push((tick, layer, instrument, key, velocity, panning, pitch));
    }

    (points, mapping)
}

/// Merges all TECs that satisfy a predicate into a single TEC containing all their coverage points.
///
/// The merged TEC has no translators (i.e., it is not a true TEC) and is used only for
/// reconstruction. It helps avoid many tiny TECs that would each occupy a small layer block.
///
/// # Arguments
/// * `tecs` - A vector of TECs to process.
/// * `filter` - A function that returns `true` for TECs that should be merged.
///
/// # Returns
/// A new vector of TECs where the filtered ones have been replaced by a single merged TEC
/// (if any were merged), and all others are kept unchanged.
///
/// # Examples
/// ```
/// # use nbslim::TranslationalEquivalence;
/// # use nbslim::utils::merge_tecs;
/// # use std::collections::HashSet;
/// let pattern = vec![(0, 0)];
/// let tec = TranslationalEquivalence::new(pattern, HashSet::new(), None);
/// let merged = merge_tecs(vec![tec], |t| t.coverage().len() == 1);
/// assert_eq!(merged.len(), 1);
/// ```
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


/// Calculates compression statistics for a list of TECs.
///
/// # Returns
/// A tuple `(original_count, encoded_units, compression_ratio)` where:
/// - `original_count` is the total number of points in the original dataset.
/// - `encoded_units` is the number of units used in the compressed representation
///   (pattern points + translators, or recursively the sum of sub‑TEC units).
/// - `compression_ratio` is `original_count / encoded_units`. A ratio greater than 1
///   indicates compression.
///
/// # Examples
/// ```
/// # use nbslim::TranslationalEquivalence;
/// # use nbslim::utils::compression_stats;
/// # use std::collections::HashSet;
/// let points = vec![(0, 0), (1, 0)];
/// let tecs = vec![TranslationalEquivalence::new(vec![(0, 0)], HashSet::new(), None)];
/// let (orig, enc, ratio) = compression_stats(&tecs, &points);
/// assert_eq!(orig, 2);
/// ```
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

/// Reconstructs the original set of points from a list of TECs, then maps
/// each point back to its original note data using a pre‑built mapping.
///
/// The reconstruction first computes the coverage (pattern + all translated copies)
/// of each TEC, merges them, and returns the **sorted unique** points.
/// Then, for each such point, it retrieves all notes associated with it
/// from the mapping (in the order they were originally inserted) and flattens
/// them into the result vector.
///
/// **Important:** This function assumes that the total number of times a point
/// appears in the compressed representation is exactly the number of notes
/// stored in the mapping for that point. Because the compression process does
/// not preserve duplicate points, the final note order is deterministic but
/// may not match the original order across different points. For exact order
/// preservation, you must maintain an auxiliary index sequence.
///
/// # Arguments
/// * `tecs` - A vector of `TranslationalEquivalence` objects, typically produced
///            by a compression algorithm.
/// * `mapping` - The mapping from a point to all original notes that produced it,
///               as returned by `notes_to_points`.
///
/// # Returns
/// A `Vec` of original note tuples, in the order: first all notes belonging to
/// the smallest `(tick, y)` point (sorted), then the next point, etc. Notes that
/// shared the exact same point are returned in the insertion order recorded in
/// the mapping.
///
/// # Examples
/// ```
/// # use nbslim::TranslationalEquivalence;
/// # use nbslim::utils::{notes_to_points, points_to_notes};
/// # use std::collections::HashSet;
/// let notes = vec![(0, 0, 1, 60, 100, 0, 0)];
/// let (points, mapping) = notes_to_points(&notes);
/// let tecs = vec![TranslationalEquivalence::new(points.clone(), HashSet::new(), None)];
/// let reconstructed = points_to_notes(&tecs, &mapping);
/// assert_eq!(reconstructed, notes);
/// ```
pub fn points_to_notes(
    tecs: &Vec<TranslationalEquivalence>,
    mapping: &HashMap<(u32, u32), Vec<(u32, u8, u8, u8, u8, i8, i16)>>,
) -> Vec<(u32, u8, u8, u8, u8, i8, i16)> {
    // 1. Rebuild the unique points covered by all TECs (sorted)
    let mut all_points = HashSet::new();
    for tec in tecs {
        all_points.extend(tec.coverage());
    }

    // 2. For each unique point, collect all original notes from the mapping
    let mut result = Vec::new();
    for point in all_points {
        if let Some(notes_at_point) = mapping.get(&point) {
            result.extend(notes_at_point.iter().cloned());
        }
        // If a point from coverage is not present in mapping (should never happen),
        // we simply skip it – it indicates an inconsistency.
    }
    result.sort();
    result
}