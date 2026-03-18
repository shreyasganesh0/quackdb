//! Lesson 17: Hash Join
//!
//! Hash join with build/probe phases supporting multiple join types.
//! The build side is fully materialized into a hash table keyed by the
//! join columns. Then the probe side is streamed through, looking up
//! matches in the hash table.
//!
//! **Key idea:** Two-phase execution:
//! 1. **Build phase** — insert all rows from the smaller relation into
//!    a hash table keyed by the join column(s).
//! 2. **Probe phase** — for each row in the larger relation, hash the
//!    join key and look up matching build-side rows.

use crate::chunk::DataChunk;
use crate::types::LogicalType;
use super::pipeline::{OperatorResult, PhysicalOperator};
use std::collections::HashMap;

/// Supported join types.
///
/// The join type affects which unmatched rows appear in the output:
/// - `Inner`: only matched pairs.
/// - `Left`/`Right`: all rows from one side, NULLs for non-matches.
/// - `Full`: all rows from both sides.
/// - `Semi`: rows from the left that have at least one match (no right columns).
/// - `Anti`: rows from the left that have no match.
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
///
/// Keys are serialized join-column values; values are lists of row indices
/// pointing back into the stored `build_chunks`.
pub struct JoinHashTable {
    /// Maps serialized key bytes -> list of (chunk_index, row_index) pairs
    /// or flat row indices into the concatenated build data.
    table: HashMap<Vec<u8>, Vec<usize>>,
    /// All chunks from the build side, stored for later row retrieval.
    build_chunks: Vec<DataChunk>,
    /// Column indices in the build chunks that form the join key.
    build_keys: Vec<usize>,
    /// Column types of the build side.
    build_types: Vec<LogicalType>,
}

impl JoinHashTable {
    /// Create a new empty join hash table.
    pub fn new(build_keys: Vec<usize>, build_types: Vec<LogicalType>) -> Self {
        // Hint: initialize table and build_chunks as empty.
        todo!()
    }

    /// Insert a chunk into the build side of the hash table.
    ///
    /// For each row, serialize the key columns, hash them, and record
    /// the row index in the hash table.
    pub fn build(&mut self, chunk: DataChunk) -> Result<(), String> {
        // Hint: iterate over rows in chunk. For each row, extract the
        // key columns, serialize to bytes, and push the row index into
        // self.table entry. Then store the chunk in build_chunks.
        todo!()
    }

    /// Probe the hash table with a chunk from the probe side.
    ///
    /// For each probe row, serialize its key columns, look up matching
    /// build rows, and produce an output chunk with combined columns
    /// according to the join type.
    pub fn probe(
        &self,
        chunk: &DataChunk,
        probe_keys: &[usize],
        join_type: JoinType,
    ) -> Result<DataChunk, String> {
        // Hint: for each probe row, hash the key and look up in self.table.
        // For Inner joins, emit one output row per match.
        // For Left joins, emit a NULL-padded row if no match.
        // For Semi/Anti, emit the probe row based on match existence.
        todo!()
    }

    /// Return the total number of rows in the build side.
    pub fn build_row_count(&self) -> usize {
        // Hint: sum the row counts of all build_chunks.
        todo!()
    }
}

/// Hash join operator for pipeline execution.
///
/// During the build phase, chunks are added via `add_build_chunk`. Once
/// `finish_build` is called, subsequent `execute` calls probe the hash
/// table with input chunks from the probe side.
pub struct HashJoinOperator {
    /// The type of join to perform.
    join_type: JoinType,
    /// Column indices forming the join key on the build side.
    build_keys: Vec<usize>,
    /// Column indices forming the join key on the probe side.
    probe_keys: Vec<usize>,
    /// The build-side hash table.
    hash_table: JoinHashTable,
    /// Whether the build phase is complete.
    build_complete: bool,
    /// Output column types (build columns ++ probe columns, or subset for semi/anti).
    output_types: Vec<LogicalType>,
}

impl HashJoinOperator {
    /// Create a new hash join operator.
    ///
    /// `build_types` and `probe_types` describe the schemas of the two
    /// input sides. The output schema is their concatenation (for inner/outer
    /// joins) or just the probe schema (for semi/anti joins).
    pub fn new(
        join_type: JoinType,
        build_keys: Vec<usize>,
        probe_keys: Vec<usize>,
        build_types: Vec<LogicalType>,
        probe_types: Vec<LogicalType>,
    ) -> Self {
        // Hint: compute output_types based on join_type. For Semi/Anti,
        // output only probe columns. For others, concatenate build + probe.
        todo!()
    }

    /// Feed a build-side chunk into the hash table.
    ///
    /// Must be called before `finish_build`.
    pub fn add_build_chunk(&mut self, chunk: DataChunk) -> Result<(), String> {
        // Hint: delegate to self.hash_table.build(chunk).
        todo!()
    }

    /// Signal that the build phase is complete. After this call, `execute`
    /// will probe the hash table with incoming chunks.
    pub fn finish_build(&mut self) {
        self.build_complete = true;
    }
}

// Trait impl: probe-side streaming after build is complete.
impl PhysicalOperator for HashJoinOperator {
    fn output_schema(&self) -> Vec<LogicalType> {
        self.output_types.clone()
    }

    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        // Hint: assert build_complete is true. Then call
        // self.hash_table.probe(input, &self.probe_keys, self.join_type)
        // and wrap the result in OperatorResult::Output.
        todo!()
    }

    fn name(&self) -> &str {
        "HashJoin"
    }
}
