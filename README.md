# NBSlim

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/nbslim.svg)](https://crates.io/crates/nbslim)
[![Rust](https://img.shields.io/badge/rust-1.95.0%2B-blue.svg)](https://www.rust-lang.org)

Rust implementation of SIA, SIATEC, COSIATEC, and RecurSIA – algorithms for compressing 2D point sets by discovering **translational equivalence classes (TECs)**. Designed for compressing Note Block Studio (.nbs) music files, but works on any set of points in the plane.

This crate is the core algorithm implementation extracted from the [NBSlim](https://github.com/madSUNitist/NBSlim) Python package, released independently for Rust projects.

## Algorithms

- **SIA** – finds all maximal translatable patterns from a point set (`O(n²)`).
- **SIATEC** – builds translational equivalence classes from SIA results (`O(m·n)`, worst-case `O(n³)`).
- **COSIATEC** – greedy lossless compression: repeatedly extract the best TEC (highest compression ratio) and remove its covered points.
- **RecurSIA** – recursive version of COSIATEC that compresses patterns of each TEC, capturing nested repetitions.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nbslim = "0.1.4"
```

## Usage

### Basic compression with COSIATEC

```rust
use nbslim::cosiatec_compress;

let points = vec![(0, 100), (1, 200), (2, 100), (3, 200)];
// restrict_dpitch_zero: only horizontal translations (Δtick, 0)
// sweepline_optimization: use faster sweepline exact matching (default true)
let tecs = cosiatec_compress(&points, true, true);
for tec in &tecs {
    println!("{}", tec.summary(0));
}
```

### Recursive compression (RecurSIA)

```rust
use nbslim::recursive_cosiatec_compress;

let tecs = recursive_cosiatec_compress(&points, true, 2, true);
// Patterns smaller than 2 points are not recursed.
```

### Working with TECs

```rust
use std::collections::HashSet;
use nbslim::TranslationalEquivalence;

// Coverage set (all points that belong to any occurrence of this TEC)
let coverage: HashSet<(u32, u32)> = tec.coverage();

// Compression ratio (coverage size / encoding units)
let ratio = tec.compression_ratio();

// Compactness relative to the full dataset
let dataset_points: HashSet<(u32, u32)> = points.iter().copied().collect();
let compactness = tec.compactness(&dataset_points);
```

### Reconstructing original points from TECs

```rust
use std::collections::HashSet;

let mut all_points = HashSet::new();
for tec in &tecs {
    all_points.extend(tec.coverage());
}
let mut reconstructed: Vec<(u32, u32)> = all_points.into_iter().collect();
reconstructed.sort();
assert_eq!(reconstructed, points);
```

### Merge small TECs (using a static predicate)

```rust
use nbslim::merge_tecs;

fn is_small(tec: &TranslationalEquivalence) -> bool {
    tec.coverage().len() <= 10
}

let merged = merge_tecs(tecs, is_small);
```

### Compression statistics

```rust
use nbslim::compression_stats;

let (original_count, encoded_units, ratio) = compression_stats(&tecs, &points);
println!("Original: {}, Encoded units: {}, Ratio: {:.2}", original_count, encoded_units, ratio);
```

### Building TECs directly from SIA patterns

```rust
use nbslim::{build_tecs_from_mtps, find_mtps};

let mtps = find_mtps(&points, false);
let tecs = build_tecs_from_mtps(&points, false);
```

## NBS file integration (optional)

The crate includes `utils::notes_to_points` and `utils::points_to_notes` to convert between NBS note events and the 2D point representation used by the algorithms, packing instrument and pitch into a single coordinate. See the [documentation](https://docs.rs/nbslim) for details.

## References

1. Meredith, D., Lemström, K., & Wiggins, G. A. (2002). *Algorithms for discovering repeated patterns in multidimensional representations of polyphonic music.*
2. Meredith, D. (2013). *COSIATEC and SIATECCompress: Pattern discovery by geometric compression.*
3. Meredith, D. (2019). *RecurSIA-RRT: Recursive translatable point-set pattern discovery with removal of redundant translators.* arXiv:1906.12286.