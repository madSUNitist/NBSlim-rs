[English](README.md) | 简体中文

# NBSlim

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/nbslim.svg)](https://crates.io/crates/nbslim)
[![Rust](https://img.shields.io/badge/rust-1.95.0%2B-blue.svg)](https://www.rust-lang.org)

SIA、SIATEC、COSIATEC 和 RecurSIA 算法的 Rust 实现，通过发现**平移等价类（TEC）**来压缩二维点集。专为压缩 Note Block Studio (.nbs) 音乐文件设计，但也适用于平面上的任意点集。

这个 crate 是从 [NBSlim](https://github.com/madSUNitist/NBSlim) Python 包中提取的核心算法实现，作为独立的 Rust crate 发布。

## Algorithms

- **SIA** – 从点集中找出所有最大可平移模式。
- **SIATEC** – 根据 SIA 的结果构建平移等价类。
- **COSIATEC** – 贪心无损压缩：重复提取最佳 TEC（压缩比最高），并移除其覆盖的点。
- **RecurSIA** – COSIATEC 的递归版本，对每个 TEC 的模式进一步压缩，捕捉嵌套重复。

所有算法均实现为 O(n²) 时间复杂度，并使用在线 HashMap 聚合避免存储所有点对，从而在大规模输入下保持内存高效。

## Installation

```toml
[dependencies]
nbslim = "0.1.0"
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
