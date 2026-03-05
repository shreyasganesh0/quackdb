//! Lesson 26: Join Order Optimization (DPsub)

use crate::planner::logical_plan::LogicalPlan;
use super::statistics::TableStatistics;
use super::cost_model::{Cost, CostModel};
use std::collections::HashMap;

/// Represents a set of relations being joined.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelationSet {
    pub bits: u64,
}

impl RelationSet {
    pub fn singleton(index: usize) -> Self {
        Self { bits: 1u64 << index }
    }

    pub fn union(&self, other: &RelationSet) -> Self {
        Self { bits: self.bits | other.bits }
    }

    pub fn intersects(&self, other: &RelationSet) -> bool {
        self.bits & other.bits != 0
    }

    pub fn is_subset_of(&self, other: &RelationSet) -> bool {
        self.bits & other.bits == self.bits
    }

    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Iterate over all non-empty subsets of this set.
    pub fn subsets(&self) -> Vec<RelationSet> {
        todo!()
    }
}

/// A join edge connecting two relations.
#[derive(Debug, Clone)]
pub struct JoinEdge {
    pub left: RelationSet,
    pub right: RelationSet,
    pub condition: Option<crate::planner::logical_plan::LogicalExpr>,
}

/// Join order optimizer using dynamic programming (DPsub algorithm).
pub struct JoinOrderOptimizer;

impl JoinOrderOptimizer {
    /// Find the optimal join order for the given relations and join edges.
    pub fn optimize(
        relations: &[LogicalPlan],
        edges: &[JoinEdge],
        stats: &HashMap<String, TableStatistics>,
    ) -> Result<LogicalPlan, String> {
        todo!()
    }

    /// Enumerate all valid join orderings using DPsub.
    pub fn dp_sub(
        relations: &[LogicalPlan],
        edges: &[JoinEdge],
        stats: &HashMap<String, TableStatistics>,
    ) -> Result<HashMap<RelationSet, (LogicalPlan, Cost)>, String> {
        todo!()
    }
}
