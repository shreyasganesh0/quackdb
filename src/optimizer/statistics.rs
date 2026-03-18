//! Lesson 26: Column Statistics & Cardinality Estimation
//!
//! Maintains per-column statistics (min, max, distinct count, histograms) and
//! uses them to estimate how many rows each plan node will produce. Accurate
//! cardinality estimates are critical for the cost model and join ordering.

use crate::types::LogicalType;
use crate::planner::logical_plan::{LogicalPlan, LogicalExpr};

/// Statistics for a single column, used to estimate predicate selectivity.
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    /// Number of distinct (non-null) values in the column.
    pub distinct_count: u64,
    /// Number of NULL values.
    pub null_count: u64,
    /// Minimum value (if available), cast to f64 for uniform comparisons.
    pub min_value: Option<f64>,
    /// Maximum value (if available).
    pub max_value: Option<f64>,
    /// Total number of rows (including NULLs).
    pub total_count: u64,
    /// Optional equi-height histogram for more precise range estimates.
    pub histogram: Option<Vec<HistogramBucket>>,
}

/// A single equi-height histogram bucket.
///
/// Each bucket covers a contiguous value range and tracks row count and
/// distinct values within that range.
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: u64,
    pub distinct_count: u64,
}

impl ColumnStatistics {
    /// Create a new `ColumnStatistics` with the given total row count.
    ///
    /// All other fields should be initialised to sensible defaults (zero counts,
    /// `None` for min/max/histogram).
    pub fn new(total_count: u64) -> Self {
        todo!()
    }

    /// Estimate selectivity for a comparison predicate (`op` is one of
    /// `"="`, `"<"`, `">"`, `"<="`, `">="`, `"!="`).
    ///
    /// Returns a value in `[0.0, 1.0]` representing the fraction of rows
    /// expected to match.
    pub fn selectivity(&self, op: &str, value: f64) -> f64 {
        // Hint: for range predicates, use (value - min) / (max - min)
        // as the uniform-distribution estimate. Fall back to a default
        // (e.g., 0.1) when min/max are unknown.
        todo!()
    }

    /// Estimate selectivity for an equality predicate: `1.0 / distinct_count`.
    ///
    /// Returns a value in `[0.0, 1.0]`.
    pub fn equality_selectivity(&self) -> f64 {
        // Hint: guard against distinct_count == 0 to avoid division by zero.
        todo!()
    }
}

/// Table-level statistics aggregating column-level info.
#[derive(Debug, Clone)]
pub struct TableStatistics {
    /// Total number of rows in the table.
    pub row_count: u64,
    /// Per-column statistics, indexed by column position.
    pub columns: Vec<ColumnStatistics>,
}

/// Estimates output cardinality (row count) for logical plan nodes.
pub struct CardinalityEstimator;

impl CardinalityEstimator {
    /// Estimate the number of rows produced by `plan`.
    ///
    /// Looks up base-table statistics from `stats` and propagates through
    /// filters, joins, projections, and aggregations.
    pub fn estimate(plan: &LogicalPlan, stats: &std::collections::HashMap<String, TableStatistics>) -> u64 {
        // Hint: use pattern matching on LogicalPlan variants. For filters,
        // multiply parent cardinality by predicate selectivity.
        todo!()
    }

    /// Estimate the selectivity of `expr` given column-level statistics.
    ///
    /// For AND expressions, multiply selectivities; for OR, use the
    /// inclusion-exclusion formula: P(A) + P(B) - P(A)*P(B).
    pub fn estimate_selectivity(expr: &LogicalExpr, stats: &[ColumnStatistics]) -> f64 {
        todo!()
    }
}
