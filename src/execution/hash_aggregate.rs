//! Lesson 16: Hash Aggregation
//!
//! Hash-based aggregation with open-addressing hash table.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};
use crate::vector::Vector;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};
use std::collections::HashMap;

/// Aggregate function types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregateType {
    Count,
    Sum,
    Avg,
    Min,
    Max,
    CountStar,
}

/// Trait for aggregate function implementations.
pub trait AggregateFunction {
    /// Initialize the aggregate state.
    fn init(&self) -> AggregateState;
    /// Update the state with a new value.
    fn update(&self, state: &mut AggregateState, value: &ScalarValue);
    /// Get the final result.
    fn finalize(&self, state: &AggregateState) -> ScalarValue;
    /// Merge two states (for parallel aggregation).
    fn merge(&self, state: &mut AggregateState, other: &AggregateState);
    /// The result type.
    fn result_type(&self) -> LogicalType;
}

/// State held by an aggregate function during computation.
#[derive(Debug, Clone)]
pub struct AggregateState {
    pub value: ScalarValue,
    pub count: u64,
    pub sum: f64,
}

impl AggregateState {
    pub fn new() -> Self {
        todo!()
    }
}

/// Create an aggregate function implementation.
pub fn create_aggregate(agg_type: AggregateType, input_type: &LogicalType) -> Box<dyn AggregateFunction> {
    todo!()
}

/// Open-addressing hash table for aggregation.
pub struct AggregateHashTable {
    /// Mapping from group key to aggregate states.
    groups: HashMap<Vec<u8>, Vec<AggregateState>>,
    group_types: Vec<LogicalType>,
    agg_types: Vec<AggregateType>,
    agg_input_types: Vec<LogicalType>,
}

impl AggregateHashTable {
    /// Create a new aggregate hash table.
    pub fn new(
        group_types: Vec<LogicalType>,
        agg_types: Vec<AggregateType>,
        agg_input_types: Vec<LogicalType>,
    ) -> Self {
        todo!()
    }

    /// Add a chunk of data to the hash table.
    pub fn add_chunk(
        &mut self,
        group_columns: &[usize],
        agg_columns: &[usize],
        chunk: &DataChunk,
    ) -> Result<(), String> {
        todo!()
    }

    /// Produce the final aggregated result as data chunks.
    pub fn finalize(&self) -> Result<Vec<DataChunk>, String> {
        todo!()
    }

    /// Number of groups in the hash table.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

/// Hash aggregate operator for pipeline execution.
pub struct HashAggregateOperator {
    group_exprs: Vec<Expression>,
    agg_types: Vec<AggregateType>,
    agg_exprs: Vec<Expression>,
    hash_table: Option<AggregateHashTable>,
    output_types: Vec<LogicalType>,
    finalized: bool,
}

impl HashAggregateOperator {
    pub fn new(
        group_exprs: Vec<Expression>,
        agg_types: Vec<AggregateType>,
        agg_exprs: Vec<Expression>,
        group_types: Vec<LogicalType>,
        agg_input_types: Vec<LogicalType>,
    ) -> Self {
        todo!()
    }
}

impl PhysicalOperator for HashAggregateOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "HashAggregate"
    }
}
