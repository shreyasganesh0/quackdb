//! Lesson 16: Hash Aggregation
//!
//! Hash-based aggregation using an open-addressing hash table. This is a
//! *pipeline breaker*: it accumulates all input during `execute`, then
//! produces grouped/aggregated output during `finalize`.
//!
//! **Key idea:** For each input chunk, evaluate group-by expressions to
//! form a group key, hash the key, and look up or create an entry in the
//! hash table. Update the aggregate states (count, sum, min, etc.) for
//! that group. After all input is consumed, iterate over the hash table
//! to produce result chunks.

use crate::chunk::DataChunk;
use crate::types::{LogicalType, ScalarValue};
use crate::vector::Vector;
use super::expression::Expression;
use super::pipeline::{OperatorResult, PhysicalOperator};
use std::collections::HashMap;

/// Supported aggregate function types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AggregateType {
    /// COUNT(expr) — counts non-NULL values.
    Count,
    /// SUM(expr) — sum of numeric values.
    Sum,
    /// AVG(expr) — average (implemented as sum/count).
    Avg,
    /// MIN(expr) — minimum value.
    Min,
    /// MAX(expr) — maximum value.
    Max,
    /// COUNT(*) — counts all rows including NULLs.
    CountStar,
}

/// Trait for aggregate function implementations.
///
/// Each aggregate function manages its own state through [`AggregateState`].
/// The lifecycle is: `init` -> repeated `update` calls -> `finalize`.
pub trait AggregateFunction {
    /// Create a fresh initial state for this aggregate.
    fn init(&self) -> AggregateState;

    /// Fold a new value into the running state.
    fn update(&self, state: &mut AggregateState, value: &ScalarValue);

    /// Extract the final scalar result from the accumulated state.
    fn finalize(&self, state: &AggregateState) -> ScalarValue;

    /// Merge two states (used for parallel / partitioned aggregation).
    fn merge(&self, state: &mut AggregateState, other: &AggregateState);

    /// The logical type of the aggregate's result.
    fn result_type(&self) -> LogicalType;
}

/// Mutable state held by an aggregate function during computation.
///
/// Fields are general-purpose so a single struct can serve all aggregate
/// types. Not every field is meaningful for every aggregate.
#[derive(Debug, Clone)]
pub struct AggregateState {
    /// Tracks the current result value (used by Min/Max).
    pub value: ScalarValue,
    /// Running row count (used by Count, Avg).
    pub count: u64,
    /// Running sum (used by Sum, Avg).
    pub sum: f64,
}

impl AggregateState {
    /// Create a new default aggregate state.
    pub fn new() -> Self {
        Self {
            value: ScalarValue::Null,
            count: 0,
            sum: 0.0,
        }
    }
}

/// Factory function: create an aggregate function implementation for the
/// given aggregate type and input type.
///
/// Returns a boxed trait object that implements the aggregate logic.
pub fn create_aggregate(agg_type: AggregateType, input_type: &LogicalType) -> Box<dyn AggregateFunction> {
    // Hint: match on agg_type and return the appropriate struct that
    // implements AggregateFunction. You may define private structs
    // (e.g., SumAggregate, CountAggregate) in this module.
    todo!()
}

/// Open-addressing hash table for group-by aggregation.
///
/// Groups are keyed by their serialized group-column values. Each group
/// entry holds a vector of [`AggregateState`]s, one per aggregate function.
pub struct AggregateHashTable {
    /// Mapping from serialized group key -> per-aggregate states.
    groups: HashMap<Vec<u8>, Vec<AggregateState>>,
    /// Types of the group-by columns.
    group_types: Vec<LogicalType>,
    /// Which aggregate functions to compute.
    agg_types: Vec<AggregateType>,
    /// Input types for each aggregate expression.
    agg_input_types: Vec<LogicalType>,
}

impl AggregateHashTable {
    /// Create a new empty aggregate hash table.
    pub fn new(
        group_types: Vec<LogicalType>,
        agg_types: Vec<AggregateType>,
        agg_input_types: Vec<LogicalType>,
    ) -> Self {
        Self {
            groups: HashMap::new(),
            group_types,
            agg_types,
            agg_input_types,
        }
    }

    /// Add a chunk of data to the hash table.
    ///
    /// For each row: serialize the group columns into a key, look up or
    /// create the group entry, then update each aggregate state with the
    /// corresponding aggregate column value.
    pub fn add_chunk(
        &mut self,
        group_columns: &[usize],
        agg_columns: &[usize],
        chunk: &DataChunk,
    ) -> Result<(), String> {
        // Hint: iterate over rows in the chunk. For each row, build a
        // byte key from group_columns, then entry-or-insert into
        // self.groups and call update on each aggregate state.
        todo!()
    }

    /// Produce the final aggregated result as data chunks.
    ///
    /// Iterates over all groups, finalizes each aggregate state, and
    /// assembles the group keys + aggregate results into output chunks.
    pub fn finalize(&self) -> Result<Vec<DataChunk>, String> {
        // Hint: for each (key, states) in self.groups, deserialize the
        // group key back into scalar values, call finalize on each
        // aggregate state, and append a row to the output.
        todo!()
    }

    /// Return the number of distinct groups accumulated so far.
    pub fn group_count(&self) -> usize {
        self.groups.len()
    }
}

/// Hash aggregate operator for pipeline execution.
///
/// This is a pipeline breaker: `execute` absorbs input into the hash table,
/// and `finalize` emits the grouped results.
pub struct HashAggregateOperator {
    /// Expressions evaluated to compute group keys.
    group_exprs: Vec<Expression>,
    /// Which aggregate functions to apply.
    agg_types: Vec<AggregateType>,
    /// Expressions evaluated to compute aggregate inputs.
    agg_exprs: Vec<Expression>,
    /// The underlying hash table (created on first use).
    hash_table: Option<AggregateHashTable>,
    /// Output column types: group types followed by aggregate result types.
    output_types: Vec<LogicalType>,
    /// Whether finalize has already been called.
    finalized: bool,
}

impl HashAggregateOperator {
    /// Create a new hash aggregate operator.
    ///
    /// `group_exprs` define the GROUP BY key; `agg_exprs` are the inputs
    /// to the aggregate functions listed in `agg_types`.
    pub fn new(
        group_exprs: Vec<Expression>,
        agg_types: Vec<AggregateType>,
        agg_exprs: Vec<Expression>,
        group_types: Vec<LogicalType>,
        agg_input_types: Vec<LogicalType>,
    ) -> Self {
        let mut output_types = group_types.clone();
        for (agg_type, input_type) in agg_types.iter().zip(agg_input_types.iter()) {
            let result_type = match agg_type {
                AggregateType::Count | AggregateType::CountStar => LogicalType::Int64,
                AggregateType::Sum => match input_type {
                    LogicalType::Float32 | LogicalType::Float64 => LogicalType::Float64,
                    _ => LogicalType::Int64,
                },
                AggregateType::Avg => LogicalType::Float64,
                AggregateType::Min | AggregateType::Max => input_type.clone(),
            };
            output_types.push(result_type);
        }
        Self {
            group_exprs,
            agg_types,
            agg_exprs,
            hash_table: None,
            output_types,
            finalized: false,
        }
    }
}

// Trait impl: pipeline breaker — accumulates during execute, emits during finalize.
impl PhysicalOperator for HashAggregateOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: evaluate group_exprs and agg_exprs against the input chunk,
        // then call hash_table.add_chunk(). Always return NeedMoreInput
        // because aggregation is a pipeline breaker.
        todo!()
    }

    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: call hash_table.finalize() to get the grouped results.
        // Return the first chunk (or None if empty).
        todo!()
    }

    fn name(&self) -> &str {
        "HashAggregate"
    }
}
