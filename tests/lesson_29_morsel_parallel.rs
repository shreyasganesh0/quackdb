//! # Lesson 29: Morsel-Driven Parallelism — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Morsel queue creation (`test_morsel_queue_creation`)
//! 2. Morsel queue consumption (`test_morsel_queue_consumption`)
//! 3. Parallel collector (`test_parallel_collector`)
//! 4. Edge cases (empty queue, single-chunk queue)
//! 5. Thread-safe queue consumption (`test_morsel_queue_thread_safe`)
//! 6. Parallel scan+filter (`test_parallel_scan_filter`)
//! 7. Deterministic results (`test_parallel_deterministic_results`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::parallel::morsel::*;
use quackdb::parallel::scheduler::*;
use quackdb::execution::pipeline::*;
use std::sync::Arc;

fn make_chunks(n: usize, rows_per_chunk: usize) -> Vec<DataChunk> {
    (0..n).map(|chunk_idx| {
        let mut chunk = DataChunk::new(&[LogicalType::Int32]);
        for i in 0..rows_per_chunk {
            chunk.append_row(&[ScalarValue::Int32((chunk_idx * rows_per_chunk + i) as i32)]);
        }
        chunk
    }).collect()
}

// ── 1. Morsel queue creation ────────────────────────────────────────

#[test]
fn test_morsel_queue_creation() {
    let chunks = make_chunks(4, 100);
    let queue = MorselQueue::new(chunks);
    assert_eq!(queue.total(), 4, "morsel queue should track the total number of chunks enqueued");
    assert_eq!(queue.remaining(), 4);
}

// ── 2. Morsel queue consumption ─────────────────────────────────────

#[test]
fn test_morsel_queue_consumption() {
    let chunks = make_chunks(3, 10);
    let queue = MorselQueue::new(chunks);

    let m1 = queue.take().unwrap();
    assert_eq!(m1.chunk.count(), 10);
    assert_eq!(queue.remaining(), 2);

    let m2 = queue.take().unwrap();
    let m3 = queue.take().unwrap();
    assert!(queue.take().is_none(), "exhausted morsel queue should return None to signal workers to stop");
    assert_eq!(queue.remaining(), 0);
}

// ── 3. Parallel collector ───────────────────────────────────────────

#[test]
fn test_parallel_collector() {
    let collector = ParallelCollector::new();
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);
    collector.push(chunk);

    let results = collector.into_results();
    assert_eq!(results.len(), 1, "ParallelCollector should gather chunks from all threads into one result set");
}

// ── 4. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_morsel_queue_empty() {
    // Edge case: queue with no chunks
    let queue = MorselQueue::new(vec![]);
    assert_eq!(queue.total(), 0, "empty queue must report total 0");
    assert_eq!(queue.remaining(), 0);
    assert!(queue.take().is_none(), "taking from an empty queue must return None immediately");
}

#[test]
fn test_morsel_queue_single_chunk() {
    // Edge case: queue with exactly one chunk
    let chunks = make_chunks(1, 5);
    let queue = MorselQueue::new(chunks);
    assert_eq!(queue.total(), 1);

    let m = queue.take().unwrap();
    assert_eq!(m.chunk.count(), 5);
    assert!(queue.take().is_none(), "single-chunk queue should be exhausted after one take");
}

// ── 5. Thread-safe queue consumption ────────────────────────────────

#[test]
fn test_morsel_queue_thread_safe() {
    let chunks = make_chunks(100, 10);
    let queue = Arc::new(MorselQueue::new(chunks));
    let collector = Arc::new(ParallelCollector::new());

    // Spawn 4 worker threads that compete for morsels
    let mut handles = Vec::new();
    for _ in 0..4 {
        let q = Arc::clone(&queue);
        let c = Arc::clone(&collector);
        handles.push(std::thread::spawn(move || {
            while let Some(morsel) = q.take() {
                c.push(morsel.chunk);
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let results = collector.into_results();
    let total_rows: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total_rows, 1000, "morsel-driven parallelism must process every row exactly once across all worker threads");
}

// ── 6. Parallel scan+filter ─────────────────────────────────────────

#[test]
fn test_parallel_scan_filter() {
    let chunks = make_chunks(10, 100);
    let queue = Arc::new(MorselQueue::new(chunks));
    let collector = Arc::new(ParallelCollector::new());

    let executor = ParallelPipelineExecutor::new(4);

    struct FilterGt50;
    impl PhysicalOperator for FilterGt50 {
        fn output_schema(&self) -> Vec<LogicalType> { vec![LogicalType::Int32] }
        fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
            let mut output = DataChunk::new(&[LogicalType::Int32]);
            for i in 0..input.count() {
                if let ScalarValue::Int32(v) = input.column(0).get_value(i) {
                    if v > 50 {
                        output.append_row(&[ScalarValue::Int32(v)]);
                    }
                }
            }
            if output.count() > 0 {
                Ok(OperatorResult::Output(output))
            } else {
                Ok(OperatorResult::NeedMoreInput)
            }
        }
        fn name(&self) -> &str { "FilterGt50" }
    }

    executor.execute(
        queue,
        || Box::new(FilterGt50),
        collector.clone(),
    ).unwrap();

    let results = Arc::try_unwrap(collector).unwrap().into_results();
    let total_rows: usize = results.iter().map(|c| c.count()).sum();
    // Values 0..1000, those > 50 = 949
    assert_eq!(total_rows, 949, "parallel filter should produce the same result as sequential -- values 51..999 = 949 rows");
}

// ── 7. Deterministic results ────────────────────────────────────────

#[test]
fn test_parallel_deterministic_results() {
    // Run twice and verify same total
    for _ in 0..2 {
        let chunks = make_chunks(8, 50);
        let queue = Arc::new(MorselQueue::new(chunks));
        let collector = Arc::new(ParallelCollector::new());

        let executor = ParallelPipelineExecutor::new(4);
        struct Passthrough;
        impl PhysicalOperator for Passthrough {
            fn output_schema(&self) -> Vec<LogicalType> { vec![LogicalType::Int32] }
            fn execute(&mut self, input: &DataChunk) -> Result<OperatorResult, String> {
                Ok(OperatorResult::Output(input.slice(0, input.count())))
            }
            fn name(&self) -> &str { "Passthrough" }
        }

        executor.execute(queue, || Box::new(Passthrough), collector.clone()).unwrap();
        let results = Arc::try_unwrap(collector).unwrap().into_results();
        let total: usize = results.iter().map(|c| c.count()).sum();
        assert_eq!(total, 400, "parallel passthrough must yield deterministic row counts regardless of thread scheduling");
    }
}
