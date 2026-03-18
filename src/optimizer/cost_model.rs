//! Lesson 26: Cost Model
//!
//! Assigns a multi-dimensional cost (CPU, I/O, network) to each plan node.
//! The optimizer uses these costs to choose between alternative plans, e.g.,
//! hash join vs. merge join or different join orderings.

use crate::planner::logical_plan::LogicalPlan;
use super::statistics::TableStatistics;
use std::collections::HashMap;

/// Multi-dimensional cost of executing a plan node.
///
/// The `total()` method combines dimensions with fixed weight multipliers,
/// reflecting the relative latency of each resource.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cost {
    /// CPU cost in abstract units (proportional to row count).
    pub cpu: f64,
    /// I/O cost -- weighted 10x CPU in `total()`.
    pub io: f64,
    /// Network cost -- weighted 100x CPU in `total()`.
    pub network: f64,
}

impl Cost {
    /// A zero-cost starting point for accumulation.
    pub fn zero() -> Self {
        Cost { cpu: 0.0, io: 0.0, network: 0.0 }
    }

    /// Compute a single scalar cost by weighting each dimension.
    ///
    /// Formula: `cpu + io * 10 + network * 100`.
    pub fn total(&self) -> f64 {
        self.cpu + self.io * 10.0 + self.network * 100.0
    }

    /// Element-wise addition of two cost vectors.
    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            cpu: self.cpu + other.cpu,
            io: self.io + other.io,
            network: self.network + other.network,
        }
    }
}

/// Estimates execution cost for various physical operators.
///
/// All methods are stateless and derive costs from row counts and table statistics.
pub struct CostModel;

impl CostModel {
    /// Estimate the total cost of executing a logical plan by recursively
    /// costing each node and summing.
    pub fn estimate(plan: &LogicalPlan, stats: &HashMap<String, TableStatistics>) -> Cost {
        // Hint: pattern-match on the LogicalPlan variant, compute the node's
        // own cost, then recurse into children and sum with `Cost::add`.
        todo!()
    }

    /// Cost of a hash join: build-side creates a hash table, probe-side looks up.
    ///
    /// CPU cost ~ `build_rows` (build) + `probe_rows` (probe).
    pub fn hash_join_cost(build_rows: u64, probe_rows: u64) -> Cost {
        todo!()
    }

    /// Cost of a merge join where both inputs are already sorted.
    ///
    /// CPU cost ~ `left_rows + right_rows` (single pass over both).
    pub fn merge_join_cost(left_rows: u64, right_rows: u64) -> Cost {
        todo!()
    }

    /// Cost of a sequential table scan.
    ///
    /// Dominated by I/O; CPU cost is linear in `rows`.
    pub fn scan_cost(rows: u64) -> Cost {
        todo!()
    }

    /// Cost of an in-memory sort (O(n log n) CPU).
    pub fn sort_cost(rows: u64) -> Cost {
        // Hint: cpu ~ rows * (rows as f64).log2()
        todo!()
    }
}
