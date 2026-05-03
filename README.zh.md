# NBSlim

[![License](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT)
[![Crates.io](https://img.shields.io/crates/v/nbslim.svg)](https://crates.io/crates/nbslim)
[![Rust](https://img.shields.io/badge/rust-1.95.0%2B-blue.svg)](https://www.rust-lang.org)

Rust 实现的 SIA、SIATEC、COSIATEC 和 RecurSIA 算法，用于通过发现**平移等价类 (TEC)** 来压缩二维点集。专为压缩 Note Block Studio (.nbs) 音乐文件设计，但也适用于平面上的任意点集。

本 crate 是从 [NBSlim](https://github.com/madSUNitist/NBSlim) Python 包中提取的核心算法实现，独立发布供 Rust 项目使用。

## 算法

- **SIA** – 从点集中找出所有最大可平移模式（`O(n²)`）。
- **SIATEC** – 从 SIA 的结果构建平移等价类（`O(m·n)`，最坏情况 `O(n³)`）。
- **COSIATEC** – 贪心无损压缩：反复提取最佳 TEC（压缩比最高），并移除其覆盖的点。
- **RecurSIA** – COSIATEC 的递归版本，压缩每个 TEC 的模式，捕捉嵌套重复。

## 安装

在 `Cargo.toml` 中添加：

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

## NBS 文件集成（可选）

本 crate 提供了 `utils::notes_to_points` 和 `utils::points_to_notes` 函数，用于在 NBS 音符事件与算法使用的二维点表示之间进行转换，将乐器和音高打包到单个坐标中。详情请参阅[文档](https://docs.rs/nbslim)。

## References

1. Meredith, D., Lemström, K., & Wiggins, G. A. (2002). *Algorithms for discovering repeated patterns in multidimensional representations of polyphonic music.*
2. Meredith, D. (2013). *COSIATEC and SIATECCompress: Pattern discovery by geometric compression.*
3. Meredith, D. (2019). *RecurSIA-RRT: Recursive translatable point-set pattern discovery with removal of redundant translators.* arXiv:1906.12286.
