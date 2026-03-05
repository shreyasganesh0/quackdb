//! Lesson 26: Cost Model

use crate::planner::logical_plan::LogicalPlan;
use super::statistics::TableStatistics;
use std::collections::HashMap;

/// Cost of executing a plan node.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Cost {
    pub cpu: f64,
    pub io: f64,
    pub network: f64,
}

impl Cost {
    pub fn zero() -> Self {
        Cost { cpu: 0.0, io: 0.0, network: 0.0 }
    }

    pub fn total(&self) -> f64 {
        self.cpu + self.io * 10.0 + self.network * 100.0
    }

    pub fn add(&self, other: &Cost) -> Cost {
        Cost {
            cpu: self.cpu + other.cpu,
            io: self.io + other.io,
            network: self.network + other.network,
        }
    }
}

/// Cost model for estimating execution cost.
pub struct CostModel;

impl CostModel {
    /// Estimate the cost of a logical plan.
    pub fn estimate(plan: &LogicalPlan, stats: &HashMap<String, TableStatistics>) -> Cost {
        todo!()
    }

    /// Cost of a hash join.
    pub fn hash_join_cost(build_rows: u64, probe_rows: u64) -> Cost {
        todo!()
    }

    /// Cost of a merge join (inputs already sorted).
    pub fn merge_join_cost(left_rows: u64, right_rows: u64) -> Cost {
        todo!()
    }

    /// Cost of a table scan.
    pub fn scan_cost(rows: u64) -> Cost {
        todo!()
    }

    /// Cost of a sort.
    pub fn sort_cost(rows: u64) -> Cost {
        todo!()
    }
}
