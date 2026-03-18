//! Lesson 14: Pipeline Execution Model
//!
//! Push-based vectorized execution pipeline. Data flows from a [`DataSource`]
//! through a chain of [`PhysicalOperator`]s and into a [`DataSink`].
//!
//! **Key idea:** The pipeline pulls chunks from the source, then pushes each
//! chunk through operators in order. Some operators (filters, projections) are
//! streaming — they process one chunk at a time. Others (aggregates, sorts) are
//! *pipeline breakers* that accumulate all input before producing output via
//! [`PhysicalOperator::finalize`].

use crate::chunk::DataChunk;
use crate::types::LogicalType;

/// State of an operator during execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorState {
    /// The operator can accept more input.
    NeedInput,
    /// The operator has output ready to be consumed.
    HasOutput,
    /// The operator has finished producing output.
    Finished,
}

/// Result of an operator's `execute` call.
#[derive(Debug)]
pub enum OperatorResult {
    /// Produced a chunk of output (pass it downstream).
    Output(DataChunk),
    /// The operator absorbed the input but has nothing to emit yet.
    NeedMoreInput,
    /// The operator has no more output to produce.
    Finished,
}

/// A physical operator in the execution pipeline.
///
/// Implement this trait for each relational operator (filter, projection,
/// aggregate, join, sort, etc.). The pipeline calls `execute` for each
/// input chunk and `finalize` once the source is exhausted.
pub trait PhysicalOperator {
    /// Return the output column types this operator produces.
    fn output_schema(&self) -> Vec<LogicalType>;

    /// One-time initialization before execution begins.
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Process an input chunk and potentially produce output.
    ///
    /// Streaming operators return `Output(chunk)` immediately.
    /// Pipeline breakers return `NeedMoreInput` while accumulating state.
    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String>;

    /// Called after the last input chunk. Pipeline breakers should emit
    /// their accumulated results here.
    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        Ok(None)
    }

    /// Human-readable operator name for debugging and EXPLAIN output.
    fn name(&self) -> &str;
}

/// A source of data chunks (pull-based).
///
/// The pipeline repeatedly calls `next_chunk` until it returns `None`.
pub trait DataSource {
    /// Return the next chunk of data, or `None` when exhausted.
    fn next_chunk(&mut self) -> Result<Option<DataChunk>, String>;

    /// Return the column types this source produces.
    fn schema(&self) -> Vec<LogicalType>;
}

/// A sink that consumes data chunks produced by the pipeline.
pub trait DataSink {
    /// Consume a single chunk of output data.
    fn consume(&mut self, chunk: DataChunk) -> Result<(), String>;

    /// Called after the last chunk has been consumed.
    fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// An in-memory data source backed by pre-built chunks.
///
/// Useful for testing and for feeding materialized intermediate results
/// back into a new pipeline.
pub struct InMemorySource {
    chunks: Vec<DataChunk>,
    position: usize,
    types: Vec<LogicalType>,
}

impl InMemorySource {
    /// Create a new in-memory source from a vector of chunks and their column types.
    pub fn new(chunks: Vec<DataChunk>, types: Vec<LogicalType>) -> Self {
        // Hint: store chunks, set position to 0, store types.
        todo!()
    }
}

// Trait impl for the pull-based DataSource interface.
impl DataSource for InMemorySource {
    fn next_chunk(&mut self) -> Result<Option<DataChunk>, String> {
        // Hint: if position < chunks.len(), return chunks[position] and
        // increment position; otherwise return None.
        todo!()
    }

    fn schema(&self) -> Vec<LogicalType> {
        self.types.clone()
    }
}

/// A sink that collects all output chunks into a `Vec`.
pub struct CollectSink {
    chunks: Vec<DataChunk>,
}

impl CollectSink {
    /// Create a new empty collect sink.
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    /// Consume the sink and return all collected chunks.
    pub fn results(self) -> Vec<DataChunk> {
        self.chunks
    }
}

impl DataSink for CollectSink {
    fn consume(&mut self, chunk: DataChunk) -> Result<(), String> {
        // Hint: push the chunk onto self.chunks.
        todo!()
    }
}

/// A pipeline connecting source -> operators -> sink.
///
/// Data flows left-to-right: the source produces chunks, each operator
/// transforms them in sequence, and the final output goes to the sink.
pub struct Pipeline {
    source: Box<dyn DataSource>,
    operators: Vec<Box<dyn PhysicalOperator>>,
}

impl Pipeline {
    /// Create a new pipeline with the given data source and no operators.
    pub fn new(source: Box<dyn DataSource>) -> Self {
        Self {
            source,
            operators: Vec::new(),
        }
    }

    /// Append an operator to the end of the pipeline.
    pub fn add_operator(&mut self, op: Box<dyn PhysicalOperator>) {
        self.operators.push(op);
    }

    /// Execute the pipeline to completion, sending all results to `sink`.
    ///
    /// 1. Pull chunks from the source.
    /// 2. Push each chunk through every operator in order.
    /// 3. Send the final output to the sink.
    /// 4. After the source is exhausted, call `finalize` on each operator
    ///    and flush any remaining output through the rest of the chain.
    pub fn execute(self, sink: &mut dyn DataSink) -> Result<(), String> {
        // Hint: loop over source.next_chunk(). For each chunk, fold it
        // through operators via execute(). When an operator returns
        // Output(chunk), pass that chunk to the next operator.
        // After the source is drained, call finalize() on each operator
        // in order and push any produced chunks downstream.
        todo!()
    }
}

/// Convenience executor that runs a pipeline and collects results.
pub struct PipelineExecutor;

impl PipelineExecutor {
    /// Execute a pipeline, collecting all output chunks into a `Vec`.
    pub fn execute(pipeline: Pipeline) -> Result<Vec<DataChunk>, String> {
        // Hint: create a CollectSink, call pipeline.execute(&mut sink),
        // then return sink.results().
        todo!()
    }
}
