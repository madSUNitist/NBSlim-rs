//! Translational Equivalence Class (TEC) – the core data structure for pattern encoding.

use std::fmt;
use std::collections::HashSet;

/// A translational equivalence class (TEC) consisting of a pattern (a set of points)
/// and a set of non‑zero translation vectors that map the pattern onto other
/// occurrences within the same dataset.
///
/// The pattern is stored as a sorted `Vec<(u32, u32)>` to ensure deterministic ordering.
/// Translators are `(i32, i32)` vectors, possibly negative in either dimension.
/// Sub‑TECs are used by recursive compression algorithms (RECURSIA) to encode
/// nested patterns.
///
/// # Examples
/// ```
/// use std::collections::HashSet;
/// use nbslim::TranslationalEquivalence;
///
/// let pattern = vec![(0, 0), (1, 0)];
/// let mut translators = HashSet::new();
/// translators.insert((1, 0)); // translation by one tick
/// let tec = TranslationalEquivalence::new(pattern, translators, None);
/// assert_eq!(tec.coverage().len(), 3); // pattern + translated copy
/// ```
#[derive(Debug, Clone, Eq)]
pub struct TranslationalEquivalence {
    /// Sorted list of points `(tick, pitch)` forming the pattern.
    pub pattern: Vec<(u32, u32)>,
    /// Set of non‑zero translation vectors `(Δtick, Δpitch)` that map `pattern`
    /// onto other patterns in the dataset.
    pub translators: HashSet<(i32, i32)>,
    /// Optional sub‑TECs obtained after recursive compression of the pattern.
    pub sub_tecs: Vec<TranslationalEquivalence>,
}

impl PartialEq for TranslationalEquivalence {
    fn eq(&self, other: &Self) -> bool {
        self.pattern == other.pattern && self.translators == other.translators && self.sub_tecs == other.sub_tecs
    }
}
impl TranslationalEquivalence {
    /// Creates a new TEC. The pattern is automatically sorted.
    ///
    /// # Arguments
    /// * `pattern` - The points forming the pattern (will be sorted).
    /// * `translators` - Non‑zero translation vectors.
    /// * `sub_tecs` - Optional recursively compressed sub‑TECs for the pattern.
    pub fn new(
        pattern: Vec<(u32, u32)>,
        translators: HashSet<(i32, i32)>,
        sub_tecs: Option<Vec<TranslationalEquivalence>>,
    ) -> Self {
        let mut pattern_sorted = pattern;
        pattern_sorted.sort();
        TranslationalEquivalence {
            pattern: pattern_sorted,
            translators,
            sub_tecs: sub_tecs.unwrap_or_else(Vec::new),
        }
    }

    /// Returns the set of all points that belong to any occurrence in this TEC.
    ///
    /// The coverage includes:
    /// - The pattern itself
    /// - All translated copies: `pattern + v` for each `v` in `translators`
    /// - If `sub_tecs` are present, their coverage is also included (merged).
    ///
    /// # Examples
    /// ```
    /// # use std::collections::HashSet;
    /// # use nbslim::TranslationalEquivalence;
    /// let pattern = vec![(0, 0)];
    /// let mut translators = HashSet::new();
    /// translators.insert((2, 0));
    /// let tec = TranslationalEquivalence::new(pattern, translators, None);
    /// assert_eq!(tec.coverage().len(), 2); // points (0,0) and (2,0)
    /// ```
    pub fn coverage(&self) -> HashSet<(u32, u32)> {
        let mut sub_cov: HashSet<(u32, u32)> = self.pattern.iter().copied().collect();
        for sub in &self.sub_tecs {
            sub_cov.extend(sub.coverage());
        }

        let mut cov = sub_cov.clone();
        for &(dx, dy) in &self.translators {
            for &(x, y) in &sub_cov {
                cov.insert(((x as i32 + dx) as u32, (y as i32 + dy) as u32));
            }
        }
        cov
    }

    /// Recursive compression ratio: coverage size / total encoding units.
    ///
    /// For a leaf TEC (no `sub_tecs`), total units = `|pattern| + |translators|`.
    /// For a non‑leaf TEC, total units = `|translators| + sum(units of sub_tecs)`.
    ///
    /// # Examples
    /// ```
    /// # use nbslim::TranslationalEquivalence;
    /// # use std::collections::HashSet;
    /// let pattern = vec![(0, 0), (1, 0)];
    /// let mut translators = HashSet::new();
    /// translators.insert((2, 0));
    /// let tec = TranslationalEquivalence::new(pattern, translators, None);
    /// // coverage size = 3, units = 2 (pattern) + 1 (translator) = 3, ratio = 1.0
    /// assert!((tec.compression_ratio() - 1.0).abs() < 1e-6);
    /// ```
    pub fn compression_ratio(&self) -> f64 {
        fn total_encoding_units(tec: &TranslationalEquivalence) -> usize {
            if tec.sub_tecs.is_empty() {
                tec.pattern.len() + tec.translators.len()
            } else {
                let mut units = tec.sub_tecs.len();
                for sub in &tec.sub_tecs {
                    units += total_encoding_units(sub);
                }
                units
            }
        }

        let cov_len = self.coverage().len();
        let total_units = total_encoding_units(&self);
        return cov_len as f64 / total_units as f64;
    }

    /// Compactness: the proportion of points inside the pattern’s bounding box that belong to the pattern.
    ///
    /// More precisely: `|pattern| / (number of dataset points lying inside the pattern’s axis‑aligned bounding box)`.
    ///
    /// If sub‑TECs exist, the pattern points from the entire subtree are merged first.
    /// Higher compactness means the pattern occupies a relatively dense region.
    ///
    /// # Arguments
    /// * `points` - The full dataset points (as a `HashSet`), used to count points inside the bounding box.
    ///
    /// # Returns
    /// A value in `[0, 1]`. Returns `0.0` if the pattern is empty or the bounding box contains no points.
    pub fn compactness(&self, points: &HashSet<(u32, u32)>) -> f64 {
        let pattern_set = self.coverage();
        if pattern_set.is_empty() {
            return 0.0;
        }
        
        let xs: Vec<u32> = pattern_set.iter().map(|p| p.0).collect();
        let ys: Vec<u32> = pattern_set.iter().map(|p| p.1).collect();
        let min_x = *xs.iter().min().unwrap();
        let max_x = *xs.iter().max().unwrap();
        let min_y = *ys.iter().min().unwrap();
        let max_y = *ys.iter().max().unwrap();
        
        let mut count = 0;
        for &(x, y) in points {
            if x >= min_x && x <= max_x && y >= min_y && y <= max_y {
                count += 1;
            }
        }
        if count == 0 {
            0.0
        } else {
            self.pattern.len() as f64 / count as f64
        }
    }

    /// Returns a human‑readable summary of the TEC, optionally with indentation.
    pub fn summary(&self, indent: usize) -> String {
        let mut lines: Vec<String> = Vec::new();
        let spaces = " ".repeat(indent);
        if self.pattern.is_empty() {
            lines.push(format!("{}translators={:?}", spaces, self.translators))
        } else {
            lines.push(format!("{}pattern={:?}, translators={:?}", spaces, self.pattern, self.translators))
        }
        lines.push(format!("{}  coverage count: {}", spaces, self.coverage().len()));
        lines.push(format!("{}  compression ratio: {}", spaces, self.compression_ratio()));
        if !self.sub_tecs.is_empty() {
            lines.push(format!("{}  sub-tecs:", spaces));
            for sub in &self.sub_tecs {
                let sub_summary = sub.summary(indent + 2);
                lines.push(sub_summary);
            }
        }

        return lines.join("\n");
    }
}

impl fmt::Display for TranslationalEquivalence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f, "TEC(pattern={:?}, translators={:?}, sub_tecs=[{}])", 
            self.pattern, 
            self.translators, 
            Vec::from_iter(self.sub_tecs.iter().map(|tec| format!("{}", tec))).join(", ")
        )
    }
}


/// A comparable key that implements the multi‑level ordering rules defined in
/// the ISBETTERTEC function from the literature.
///
/// Lower key value corresponds to a **better** TEC according to:
/// 1. Higher compression ratio
/// 2. Higher compactness
/// 3. Larger coverage size
/// 4. Larger pattern size
/// 5. Smaller temporal width (duration)
/// 6. Smaller bounding‑box area
///
/// This struct is used with `sort_by_key` and `min_by_key` to select optimal TECs
/// in algorithms like `COSIATEC` and `SIATECCompress`.
pub(crate) struct SortKey {
    cr: f64,      // f64 do not implement `Eq`
    comp: f64,    // f64 do not implement `Eq`
    cov_len: usize, 
    pat_len: usize, 
    width: u32, 
    area: u64
}

impl SortKey {
    pub fn new(cr: f64, comp: f64, cov_len: usize, pat_len: usize, width: u32, area: u64) -> Self {
        debug_assert!(cr.is_finite(), "compression ratio must be finite");
        debug_assert!(comp.is_finite(), "compactness must be finite");
        Self { cr, comp, cov_len, pat_len, width, area }
    }
}

impl Eq for SortKey {}
impl PartialEq for SortKey {
    fn eq(&self, other: &Self) -> bool {
        self.cr == other.cr && 
        self.comp == other.comp && 
        self.cov_len == other.cov_len && 
        self.pat_len == other.pat_len && 
        self.width == other.width && 
        self.area == other.area
    }
}
impl PartialOrd for SortKey {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        // For descending fields, store negative. 
        match (-self.cr).partial_cmp(&(-other.cr)) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match (-self.comp).partial_cmp(&(-other.comp)) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match (-(self.cov_len as i64)).partial_cmp(&(-(other.cov_len as i64))) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        match (-(self.pat_len as i64)).partial_cmp(&(-(other.pat_len as i64))) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        // For ascending fields, store positive.
        match self.width.partial_cmp(&other.width) {
            Some(core::cmp::Ordering::Equal) => {}
            ord => return ord,
        }
        self.area.partial_cmp(&other.area)
    }
}
impl Ord for SortKey {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // partial_cmp is guaranteed to return Some because all comparands are finite.
        self.partial_cmp(other).unwrap()
    }
}

/// Constructs a `SortKey` for a given TEC with respect to a dataset.
/// The key follows the ISBETTERTEC multi‑level comparison rules and can be used
/// to sort TECs from best to worst.
///
/// # Arguments
/// * `tec` - The TEC to evaluate.
/// * `dataset_points` - The set of points in the dataset (used for compactness calculation).
///
/// # Returns
/// A `SortKey` where `key1 < key2` means TEC1 is better than TEC2.
pub fn tec_sort_key(tec: &TranslationalEquivalence, dataset_points: &HashSet<(u32, u32)>) -> SortKey {
    let cr = tec.compression_ratio();
    let comp = tec.compactness(dataset_points);
    let cov_len = tec.coverage().len();
    let pat_len = tec.pattern.len();
    let width = (
        tec.pattern
            .iter()
            .map(|x| x.0 as i32)
            .max().unwrap() - 
        tec.pattern
            .iter()
            .map(|x| x.0 as i32)
            .min().unwrap()
    ) as u32;
    let x_range = width;
    let y_range = (
        tec.pattern
            .iter()
            .map(|x| x.1 as i32)
            .max().unwrap() - 
        tec.pattern
            .iter()
            .map(|x| x.1 as i32)
            .min().unwrap()
    ) as u32;
    let area = x_range as u64 * y_range as u64;
    SortKey::new(cr, comp, cov_len, pat_len, width, area)
}