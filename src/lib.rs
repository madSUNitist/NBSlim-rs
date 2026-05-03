//! # NBSlim – Geometric compression algorithms for 2D point sets
//!
//! This crate implements SIA, SIATEC, COSIATEC, and RecurSIA – algorithms that discover
//! translational equivalence classes (TECs) in planar point sets. It is designed primarily
//! for compressing Note Block Studio (`.nbs`) music files but works on any set of points
//! in the plane.
//!
//! # Algorithms
//!
//! - **SIA** – finds all maximal translatable patterns from a point set.
//! - **SIATEC** – builds translational equivalence classes from SIA results.
//! - **COSIATEC** – greedy lossless compression: repeatedly extracts the best TEC and
//!   removes its covered points.
//! - **RecurSIA** – recursive version of COSIATEC that compresses patterns of each TEC
//!   to capture nested repetitions.
//!
//! # Examples
//!
//! Basic compression:
//!
//! ```
//! use nbslim::cosiatec_compress;
//!
//! let points = vec![(0, 100), (1, 200), (2, 100), (3, 200)];
//! let tecs = cosiatec_compress(&points, true, true);
//! assert!(tecs.len() > 0);
//! ```

mod sia;
mod tec;
mod siatec;
mod cosiatec;
mod recursia;
mod sweepline;

pub mod utils;

pub use tec::TranslationalEquivalence;
pub use sia::find_mtps;
pub use siatec::build_tecs_from_mtps;
pub use cosiatec::cosiatec_compress;
pub use recursia::recursive_cosiatec_compress;
pub use sweepline::build_tecs_from_mtps as build_tecs_from_mtps_sweepline;


#[cfg(test)]
mod test {
    use std::collections::HashSet;

    use crate::{
        TranslationalEquivalence, 
        cosiatec_compress, 
        recursive_cosiatec_compress, 
        utils::{compression_stats, notes_to_points, points_to_notes}
    };

    const NOTES: &str = include_str!("../tests/fixtures/notes.json");
    const DATASET: &str = include_str!("../tests/fixtures/dataset.json");
    
    fn load_test_data() -> (Vec<(u32, u8, u8, u8, u8, i8, i16)>, Vec<(u32, u32)>) {
        let parsed_notes: Vec<[i64; 7]> = serde_json::from_str(NOTES).unwrap();
        let parsed_dataset: Vec<[u32; 2]> = serde_json::from_str(DATASET).unwrap();
        (
            parsed_notes
                .into_iter()
                .map(|[a, b, c, d, e, f, g]| (a as u32, b as u8, c as u8, d as u8, e as u8, f as i8, g as i16))
                .collect(), 
            parsed_dataset.into_iter().map(|[a, b]| (a, b)).collect()
        )
    }
    
    fn overall_coverage(tecs: Vec<TranslationalEquivalence>) -> HashSet<(u32, u32)> {
        let mut all_points = HashSet::new();
        for tec in tecs {
            all_points.extend(tec.coverage());
        }
        all_points
    }

    #[test]
    fn test_recursive_cosiatec_compress() {
        let (mut notes, data) = load_test_data();
        let (points, mapping) = notes_to_points(&notes);
        assert_eq!(points, data);

        let tecs = recursive_cosiatec_compress(&data, true, 2, true);
        assert!(compression_stats(&tecs, &data).2 > 1.0);

        let rebuilt = points_to_notes(&tecs, &mapping);
        notes.sort();
        assert_eq!(rebuilt, notes);

        let dataset: HashSet<_> = data.into_iter().collect();
        let coverage: HashSet<_> = overall_coverage(tecs);
        assert_eq!(dataset, coverage);
    }

    #[test]
    fn test_cosiatec_compress() {
        let (mut notes, data) = load_test_data();
        let (points, mapping) = notes_to_points(&notes);
        assert_eq!(points, data);

        let tecs = cosiatec_compress(&data, true, true);
        assert!(compression_stats(&tecs, &data).2 > 1.0);

        let rebuilt = points_to_notes(&tecs, &mapping);
        notes.sort();
        assert_eq!(rebuilt, notes);

        let dataset: HashSet<_> = data.into_iter().collect();
        let coverage: HashSet<_> = overall_coverage(tecs);
        assert_eq!(dataset, coverage);
    }
}