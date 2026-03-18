//! # Lesson 14: Pipeline Execution Model — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Source basics (`test_inmemory_source`)
//! 2. Sink basics (`test_collect_sink`)
//! 3. Empty pipeline (`test_pipeline_empty_source`)
//! 4. Single operator pipeline (`test_pipeline_passthrough`, `test_pipeline_single_operator`)
//! 5. Edge cases (single-element input, empty chunk)
//! 6. Chained operators (`test_pipeline_chain`)
//! 7. Multiple chunks (`test_pipeline_multiple_chunks`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::pipeline::*;

struct PassthroughOperator;

impl PhysicalOperator for PassthroughOperator {
    fn output_schema(&self) -> Vec<LogicalType> { vec![LogicalType::Int32] }
    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        Ok(OperatorResult::Output(input.slice(0, input.count())))
    }
    fn name(&self) -> &str { "Passthrough" }
}

struct DoubleOperator;

impl PhysicalOperator for DoubleOperator {
    fn output_schema(&self) -> Vec<LogicalType> { vec![LogicalType::Int32] }
    fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
        let mut output = DataChunk::new(&[LogicalType::Int32]);
        for i in 0..input.count() {
            if let ScalarValue::Int32(v) = input.column(0).get_value(i) {
                output.append_row(&[ScalarValue::Int32(v * 2)]);
            }
        }
        Ok(OperatorResult::Output(output))
    }
    fn name(&self) -> &str { "Double" }
}

fn make_test_source(data: Vec<Vec<i32>>) -> InMemorySource {
    let chunks: Vec<DataChunk> = data.into_iter().map(|vals| {
        let mut chunk = DataChunk::new(&[LogicalType::Int32]);
        for v in vals {
            chunk.append_row(&[ScalarValue::Int32(v)]);
        }
        chunk
    }).collect();
    InMemorySource::new(chunks, vec![LogicalType::Int32])
}

// ── 1. Source basics ────────────────────────────────────────────────

#[test]
fn test_inmemory_source() {
    let mut source = make_test_source(vec![vec![1, 2], vec![3]]);
    let chunk1 = source.next_chunk().unwrap().unwrap();
    assert_eq!(chunk1.count(), 2);
    let chunk2 = source.next_chunk().unwrap().unwrap();
    assert_eq!(chunk2.count(), 1);
    let chunk3 = source.next_chunk().unwrap();
    assert!(chunk3.is_none(), "source should return None after all chunks are consumed, signaling end-of-data");
}

// ── 2. Sink basics ─────────────────────────────────────────────────

#[test]
fn test_collect_sink() {
    let mut sink = CollectSink::new();
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);
    sink.consume(chunk).unwrap();

    let results = sink.results();
    assert_eq!(results.len(), 1, "sink should accumulate each consumed chunk");
    assert_eq!(results[0].column(0).get_value(0), ScalarValue::Int32(42));
}

// ── 3. Empty pipeline ──────────────────────────────────────────────

#[test]
fn test_pipeline_empty_source() {
    let source = make_test_source(vec![]);
    let pipeline = Pipeline::new(Box::new(source));
    let results = PipelineExecutor::execute(pipeline).unwrap();
    assert!(results.is_empty(), "a pipeline with no input chunks should produce no output");
}

// ── 4. Single operator pipeline ─────────────────────────────────────

#[test]
fn test_pipeline_passthrough() {
    let source = make_test_source(vec![vec![1, 2, 3]]);
    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(PassthroughOperator));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 3);
}

#[test]
fn test_pipeline_single_operator() {
    let source = make_test_source(vec![vec![1, 2, 3]]);
    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(DoubleOperator));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    assert!(!results.is_empty());
    let chunk = &results[0];
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(2), "operator should transform each row: 1*2=2");
    assert_eq!(chunk.column(0).get_value(1), ScalarValue::Int32(4));
    assert_eq!(chunk.column(0).get_value(2), ScalarValue::Int32(6));
}

// ── 5. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_pipeline_single_element() {
    // Edge case: pipeline with exactly one row
    let source = make_test_source(vec![vec![42]]);
    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(DoubleOperator));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 1, "single-element input must produce single-element output");
    assert_eq!(results[0].column(0).get_value(0), ScalarValue::Int32(84), "42 * 2 = 84");
}

// ── 6. Chained operators ────────────────────────────────────────────

#[test]
fn test_pipeline_chain() {
    let source = make_test_source(vec![vec![5, 10]]);
    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(DoubleOperator)); // *2
    pipeline.add_operator(Box::new(DoubleOperator)); // *4

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let chunk = &results[0];
    assert_eq!(chunk.column(0).get_value(0), ScalarValue::Int32(20), "chained operators compose: 5*2*2=20, data flows through each stage");
    assert_eq!(chunk.column(0).get_value(1), ScalarValue::Int32(40));
}

// ── 7. Multiple chunks ─────────────────────────────────────────────

#[test]
fn test_pipeline_multiple_chunks() {
    let source = make_test_source(vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(PassthroughOperator));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 6, "pipeline must process all chunks from the source, not just the first");
}
