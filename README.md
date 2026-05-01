English | [简体中文](README.zh.md)

# NBSlim

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/nbslim.svg)](https://crates.io/crates/nbslim)
[![Rust](https://img.shields.io/badge/rust-1.95.0%2B-blue.svg)](https://www.rust-lang.org)

Rust implementation of SIA, SIATEC, COSIATEC, and RecurSIA – algorithms for compressing 2D point sets by discovering **translational equivalence classes (TECs)**. Designed for compressing Note Block Studio (.nbs) music files, but works on any set of points in the plane.

This crate is the core algorithm implementation extracted from the [NBSlim](https://github.com/madSUNitist/NBSlim) Python package, released independently for Rust projects.

## Algorithms

- **SIA** – finds all maximal translatable patterns from a point set.
- **SIATEC** – builds translational equivalence classes from SIA results.
- **COSIATEC** – greedy lossless compression: repeatedly extract the best TEC (highest compression ratio) and remove its covered points.
- **RecurSIA** – recursive version of COSIATEC that compresses patterns of each TEC, capturing nested repetitions.

All algorithms are implemented with `O(n²)` time complexity and use online HashMap aggregation to avoid storing all point pairs, making them memory efficient for large inputs.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
nbslim = "0.1.1"
```

## Usage

### Basic compression with COSIATEC

```rust
use nbslim::{cosiatec_compress, TranslationalEquivalence};

let points = vec![(0, 100), (1, 200), (2, 100), (3, 200)];
let tecs = cosiatec_compress(&points, true);  // restrict_dpitch_zero = true
for tec in &tecs {
    println!("{}", tec.summary(0));
}
```

### Recursive compression (RecurSIA)

```rust
use nbslim::recursive_cosiatec_compress;

let tecs = recursive_cosiatec_compress(&points, true, 2);
```

### Working with TECs

```rust
use std::collections::HashSet;

// Coverage set (all points that belong to any occurrence of this TEC)
let coverage: HashSet<(u32, u32)> = tec.coverage();

// Compression ratio (coverage size / encoding units)
let ratio = tec.compression_ratio();

// Compactness relative to the full dataset
let dataset_points: HashSet<(u32, u32)> = points.iter().copied().collect();
let compactness = tec.compactness(&dataset_points);
```

### Reconstructing original points from a list of TECs

```rust
use nbslim::rebuild;

let original_points = rebuild(&tecs);
assert_eq!(original_points, points);
```

### Merge small TECs

```rust
use nbslim::merge_tecs;

// Merge all TECs with coverage size <= 10 into a single TEC
let merged = merge_tecs(tecs, |t| t.coverage().len() <= 10);
```

### Compression statistics

```rust
use nbslim::compression_stats;

let (original_count, encoded_units, ratio) = compression_stats(&tecs, &points);
println!("Original: {}, Encoded units: {}, Ratio: {:.2}", original_count, encoded_units, ratio);
```

## References

1. Meredith, D., Lemström, K., & Wiggins, G. A. (2002). *Algorithms for discovering repeated patterns in multidimensional representations of polyphonic music.*
2. Meredith, D. (2013). *COSIATEC and SIATECCompress: Pattern discovery by geometric compression.*
3. Meredith, D. (2019). *RecurSIA-RRT: Recursive translatable point-set pattern discovery with removal of redundant translators.* arXiv:1906.12286.
