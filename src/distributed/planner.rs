//! Lesson 32: Distributed Query Planning
//!
//! Transforms a single-node logical plan into a set of plan fragments
//! connected by exchange operators. Each fragment runs on a separate node;
//! exchanges handle data movement (shuffle, broadcast, gather) between nodes.

use crate::planner::logical_plan::LogicalPlan;

/// Types of data exchange between distributed plan fragments.
#[derive(Debug, Clone)]
pub enum ExchangeType {
    /// All partitions send data to a single coordinator node.
    Gather,
    /// Repartition data by hashing the specified columns.
    Repartition { columns: Vec<usize>, num_partitions: usize },
    /// Send a full copy of the data to every node (used for small dimension tables).
    Broadcast,
}

/// A fragment of a distributed query plan that executes on a single node.
///
/// Fragments are connected by exchanges: `exchange_input` describes how this
/// fragment receives data; `exchange_output` describes how it sends data.
#[derive(Debug, Clone)]
pub struct PlanFragment {
    /// The local logical plan to execute on this node.
    pub plan: LogicalPlan,
    /// Unique identifier for this fragment.
    pub fragment_id: usize,
    /// How this fragment receives input from upstream fragments.
    pub exchange_input: Option<ExchangeType>,
    /// How this fragment sends output to downstream fragments.
    pub exchange_output: Option<ExchangeType>,
}

/// Converts a single-node logical plan into distributed plan fragments.
pub struct DistributedPlanner {
    num_nodes: usize,
}

impl DistributedPlanner {
    /// Create a planner targeting the given number of cluster nodes.
    pub fn new(num_nodes: usize) -> Self {
        Self { num_nodes }
    }

    /// Split a logical plan into fragments with exchange operators inserted
    /// at boundaries that require data movement.
    ///
    /// Walk the plan tree top-down; at each node, check if an exchange is
    /// needed between parent and child.
    pub fn plan(&self, logical_plan: LogicalPlan) -> Result<Vec<PlanFragment>, String> {
        // Hint: use a FragmentBuilder. For joins, check whether both sides
        // are co-partitioned on the join key; if not, insert a Repartition.
        todo!()
    }

    /// Determine whether an exchange is needed between `parent` and `child`.
    ///
    /// Returns `None` if data can stay local, or `Some(ExchangeType)` describing
    /// the required data movement.
    fn needs_exchange(&self, parent: &LogicalPlan, child: &LogicalPlan) -> Option<ExchangeType> {
        // Hint: joins need repartitioning unless both sides are already
        // partitioned on the join key. Final result needs Gather.
        todo!()
    }
}

/// Helper that incrementally builds plan fragments with auto-assigned IDs.
pub struct FragmentBuilder {
    fragments: Vec<PlanFragment>,
    next_id: usize,
}

impl FragmentBuilder {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self { fragments: Vec::new(), next_id: 0 }
    }

    /// Add a fragment and return its assigned ID.
    pub fn add_fragment(&mut self, plan: LogicalPlan, exchange_in: Option<ExchangeType>, exchange_out: Option<ExchangeType>) -> usize {
        // Hint: create a PlanFragment with `next_id`, push it, increment next_id.
        todo!()
    }

    /// Consume the builder and return all fragments.
    pub fn build(self) -> Vec<PlanFragment> {
        self.fragments
    }
}

impl Default for FragmentBuilder {
    fn default() -> Self {
        Self::new()
    }
}
