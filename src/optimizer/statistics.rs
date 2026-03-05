//! Lesson 26: Column Statistics & Cardinality Estimation

use crate::types::LogicalType;
use crate::planner::logical_plan::{LogicalPlan, LogicalExpr};

/// Statistics for a single column.
#[derive(Debug, Clone)]
pub struct ColumnStatistics {
    pub distinct_count: u64,
    pub null_count: u64,
    pub min_value: Option<f64>,
    pub max_value: Option<f64>,
    pub total_count: u64,
    /// Simple equi-height histogram buckets.
    pub histogram: Option<Vec<HistogramBucket>>,
}

/// A histogram bucket.
#[derive(Debug, Clone)]
pub struct HistogramBucket {
    pub lower_bound: f64,
    pub upper_bound: f64,
    pub count: u64,
    pub distinct_count: u64,
}

impl ColumnStatistics {
    pub fn new(total_count: u64) -> Self {
        todo!()
    }

    /// Estimate selectivity for a simple predicate.
    pub fn selectivity(&self, op: &str, value: f64) -> f64 {
        todo!()
    }

    /// Estimate selectivity for an equality predicate.
    pub fn equality_selectivity(&self) -> f64 {
        todo!()
    }
}

/// Table-level statistics.
#[derive(Debug, Clone)]
pub struct TableStatistics {
    pub row_count: u64,
    pub columns: Vec<ColumnStatistics>,
}

/// Estimate the cardinality (output row count) of a logical plan node.
pub struct CardinalityEstimator;

impl CardinalityEstimator {
    /// Estimate the number of rows produced by a plan node.
    pub fn estimate(plan: &LogicalPlan, stats: &std::collections::HashMap<String, TableStatistics>) -> u64 {
        todo!()
    }

    /// Estimate selectivity of a filter expression.
    pub fn estimate_selectivity(expr: &LogicalExpr, stats: &[ColumnStatistics]) -> f64 {
        todo!()
    }
}
