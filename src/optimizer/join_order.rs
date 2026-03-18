//! Lesson 26: Join Order Optimization (DPsub)
//!
//! Finds the optimal join order for multi-way joins using the DPsub dynamic
//! programming algorithm. The key idea: enumerate all subsets of relations,
//! computing the cheapest join tree for each subset bottom-up.

use crate::planner::logical_plan::LogicalPlan;
use super::statistics::TableStatistics;
use super::cost_model::{Cost, CostModel};
use std::collections::HashMap;

/// A bitmask representing a set of relations in the join graph.
///
/// Each bit position corresponds to a relation index. For example, bit 0
/// is relation 0, bit 1 is relation 1, and `bits = 0b101` is {0, 2}.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct RelationSet {
    pub bits: u64,
}

impl RelationSet {
    /// Create a set containing only the relation at `index`.
    pub fn singleton(index: usize) -> Self {
        // Bit-shift to set exactly one bit.
        Self { bits: 1u64 << index }
    }

    /// Return the union of two relation sets (bitwise OR).
    pub fn union(&self, other: &RelationSet) -> Self {
        Self { bits: self.bits | other.bits }
    }

    /// Return `true` if the two sets share at least one relation.
    pub fn intersects(&self, other: &RelationSet) -> bool {
        self.bits & other.bits != 0
    }

    /// Return `true` if `self` is a subset of `other`.
    pub fn is_subset_of(&self, other: &RelationSet) -> bool {
        self.bits & other.bits == self.bits
    }

    /// Number of relations in the set (population count).
    pub fn count(&self) -> u32 {
        self.bits.count_ones()
    }

    /// Iterate over all non-empty proper subsets of this set.
    ///
    /// This is the inner loop of DPsub. Use the bit-manipulation trick:
    /// `sub = (sub - 1) & self.bits` to enumerate subsets in descending order.
    pub fn subsets(&self) -> Vec<RelationSet> {
        todo!()
    }
}

/// An edge in the join graph connecting two groups of relations.
#[derive(Debug, Clone)]
pub struct JoinEdge {
    /// Relations referenced by the left side of the join predicate.
    pub left: RelationSet,
    /// Relations referenced by the right side of the join predicate.
    pub right: RelationSet,
    /// The join condition (e.g., `t1.id = t2.id`). `None` means cross join.
    pub condition: Option<crate::planner::logical_plan::LogicalExpr>,
}

/// Join order optimizer using the DPsub algorithm.
///
/// DPsub explores all possible binary join trees for up to ~20 relations
/// (limited by the 64-bit bitmask). For each subset of relations, it picks
/// the split (left subset, right subset) with the lowest combined cost.
pub struct JoinOrderOptimizer;

impl JoinOrderOptimizer {
    /// Find the cheapest join order and return the resulting physical plan.
    pub fn optimize(
        relations: &[LogicalPlan],
        edges: &[JoinEdge],
        stats: &HashMap<String, TableStatistics>,
    ) -> Result<LogicalPlan, String> {
        // Hint: call `dp_sub` to build the cost table, then extract the plan
        // for the full relation set.
        todo!()
    }

    /// Build the DP table mapping each relation subset to its best plan and cost.
    ///
    /// Algorithm outline:
    /// 1. Initialise singletons with their scan costs.
    /// 2. For each subset size 2..=N, enumerate splits via `subsets()`.
    /// 3. For each valid split that has a corresponding join edge, compute the
    ///    combined cost and keep the minimum.
    pub fn dp_sub(
        relations: &[LogicalPlan],
        edges: &[JoinEdge],
        stats: &HashMap<String, TableStatistics>,
    ) -> Result<HashMap<RelationSet, (LogicalPlan, Cost)>, String> {
        todo!()
    }
}
