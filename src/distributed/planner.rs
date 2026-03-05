//! Lesson 32: Distributed Query Planning
//!
//! Insert exchange operators into query plans for distributed execution.

use crate::planner::logical_plan::LogicalPlan;

/// Exchange types for data movement between nodes.
#[derive(Debug, Clone)]
pub enum ExchangeType {
    Gather,
    Repartition { columns: Vec<usize>, num_partitions: usize },
    Broadcast,
}

/// A fragment of a distributed query plan.
#[derive(Debug, Clone)]
pub struct PlanFragment {
    pub plan: LogicalPlan,
    pub fragment_id: usize,
    pub exchange_input: Option<ExchangeType>,
    pub exchange_output: Option<ExchangeType>,
}

/// Distributed query planner.
pub struct DistributedPlanner {
    num_nodes: usize,
}

impl DistributedPlanner {
    pub fn new(num_nodes: usize) -> Self {
        Self { num_nodes }
    }

    /// Convert a logical plan into distributed plan fragments with exchanges.
    pub fn plan(&self, logical_plan: LogicalPlan) -> Result<Vec<PlanFragment>, String> {
        todo!()
    }

    /// Determine if an exchange is needed between two plan nodes.
    fn needs_exchange(&self, parent: &LogicalPlan, child: &LogicalPlan) -> Option<ExchangeType> {
        todo!()
    }
}

/// Build plan fragments from a logical plan.
pub struct FragmentBuilder {
    fragments: Vec<PlanFragment>,
    next_id: usize,
}

impl FragmentBuilder {
    pub fn new() -> Self {
        Self { fragments: Vec::new(), next_id: 0 }
    }

    pub fn add_fragment(&mut self, plan: LogicalPlan, exchange_in: Option<ExchangeType>, exchange_out: Option<ExchangeType>) -> usize {
        todo!()
    }

    pub fn build(self) -> Vec<PlanFragment> {
        self.fragments
    }
}

impl Default for FragmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
