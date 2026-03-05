//! Lesson 17: Hash Join
//!
//! Hash join with build/probe phases supporting multiple join types.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};
use std::collections::HashMap;

/// Join types.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum JoinType {
    Inner,
    Left,
    Right,
    Full,
    Semi,
    Anti,
}

/// Hash table for the build side of a hash join.
pub struct JoinHashTable {
    /// Maps hash keys to lists of row indices.
    table: HashMap<Vec<u8>, Vec<usize>>,
    /// The build-side data.
    build_chunks: Vec<DataChunk>,
    /// Key column indices on the build side.
    build_keys: Vec<usize>,
    build_types: Vec<LogicalType>,
}

impl JoinHashTable {
    /// Create a new join hash table.
    pub fn new(build_keys: Vec<usize>, build_types: Vec<LogicalType>) -> Self {
        todo!()
    }

    /// Add a chunk to the build side.
    pub fn build(&mut self, chunk: DataChunk) -> Result<(), String> {
        todo!()
    }

    /// Probe the hash table with a chunk from the probe side.
    pub fn probe(
        &self,
        chunk: &DataChunk,
        probe_keys: &[usize],
        join_type: JoinType,
    ) -> Result<DataChunk, String> {
        todo!()
    }

    /// Number of rows in the build side.
    pub fn build_row_count(&self) -> usize {
        todo!()
    }
}

/// Hash join operator.
pub struct HashJoinOperator {
    join_type: JoinType,
    build_keys: Vec<usize>,
    probe_keys: Vec<usize>,
    hash_table: JoinHashTable,
    build_complete: bool,
    output_types: Vec<LogicalType>,
}

impl HashJoinOperator {
    pub fn new(
        join_type: JoinType,
        build_keys: Vec<usize>,
        probe_keys: Vec<usize>,
        build_types: Vec<LogicalType>,
        probe_types: Vec<LogicalType>,
    ) -> Self {
        todo!()
    }

    /// Feed a build-side chunk.
    pub fn add_build_chunk(&mut self, chunk: DataChunk) -> Result<(), String> {
        todo!()
    }

    /// Signal that the build phase is complete.
    pub fn finish_build(&mut self) {
        self.build_complete = true;
    }
}

impl PhysicalOperator for HashJoinOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        todo!()
    }

    fn name(&self) -> &str {
        "HashJoin"
    }
}
