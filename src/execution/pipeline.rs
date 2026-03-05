//! Lesson 14: Pipeline Execution Model
//!
//! Push-based vectorized execution pipeline.

use crate::chunk::DataChunk;
use crate::types::LogicalType;

/// State of an operator during execution.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum OperatorState {
    /// Need more input.
    NeedInput,
    /// Has output available.
    HasOutput,
    /// Done producing output.
    Finished,
}

/// Result of an operator's execute call.
#[derive(Debug)]
pub enum OperatorResult {
    /// Produced a chunk of output.
    Output(DataChunk),
    /// Need more input to produce output.
    NeedMoreInput,
    /// No more output to produce.
    Finished,
}

/// A physical operator in the execution pipeline.
pub trait PhysicalOperator {
    /// Get the output schema of this operator.
    fn output_schema(&self) -> Vec<LogicalType>;

    /// Initialize the operator.
    fn init(&mut self) -> Result<(), String> {
        Ok(())
    }

    /// Process an input chunk and potentially produce output.
    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String>;

    /// Signal that no more input will arrive. May produce final output.
    fn finalize(&mut self) -> Result<Option<DataChunk>, String> {
        Ok(None)
    }

    /// Get the name of this operator for debugging.
    fn name(&self) -> &str;
}

/// A source of data chunks.
pub trait DataSource {
    /// Get the next chunk of data. Returns None when exhausted.
    fn next_chunk(&mut self) -> Result<Option<DataChunk>, String>;

    /// Get the output schema.
    fn schema(&self) -> Vec<LogicalType>;
}

/// A sink that consumes data chunks.
pub trait DataSink {
    /// Consume a chunk of data.
    fn consume(&mut self, chunk: DataChunk) -> Result<(), String>;

    /// Signal that no more data will arrive.
    fn finalize(&mut self) -> Result<(), String> {
        Ok(())
    }
}

/// An in-memory data source from pre-built chunks.
pub struct InMemorySource {
    chunks: Vec<DataChunk>,
    position: usize,
    types: Vec<LogicalType>,
}

impl InMemorySource {
    pub fn new(chunks: Vec<DataChunk>, types: Vec<LogicalType>) -> Self {
        todo!()
    }
}

impl DataSource for InMemorySource {
    fn next_chunk(&mut self) -> Result<Option<DataChunk>, String> {
        todo!()
    }

    fn schema(&self) -> Vec<LogicalType> {
        self.types.clone()
    }
}

/// A sink that collects all chunks.
pub struct CollectSink {
    chunks: Vec<DataChunk>,
}

impl CollectSink {
    pub fn new() -> Self {
        Self { chunks: Vec::new() }
    }

    pub fn results(self) -> Vec<DataChunk> {
        self.chunks
    }
}

impl DataSink for CollectSink {
    fn consume(&mut self, chunk: DataChunk) -> Result<(), String> {
        todo!()
    }
}

/// A pipeline connecting a source -> operators -> sink.
pub struct Pipeline {
    source: Box<dyn DataSource>,
    operators: Vec<Box<dyn PhysicalOperator>>,
}

impl Pipeline {
    pub fn new(source: Box<dyn DataSource>) -> Self {
        Self {
            source,
            operators: Vec::new(),
        }
    }

    /// Add an operator to the pipeline.
    pub fn add_operator(&mut self, op: Box<dyn PhysicalOperator>) {
        self.operators.push(op);
    }

    /// Execute the pipeline, sending results to the sink.
    pub fn execute(self, sink: &mut dyn DataSink) -> Result<(), String> {
        todo!()
    }
}

/// Executor that runs a pipeline to completion.
pub struct PipelineExecutor;

impl PipelineExecutor {
    /// Execute a pipeline, collecting results.
    pub fn execute(pipeline: Pipeline) -> Result<Vec<DataChunk>, String> {
        todo!()
    }
}
