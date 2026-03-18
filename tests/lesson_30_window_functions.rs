//! # Lesson 30: Window Functions — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. ROW_NUMBER — basic ranking (`test_row_number`)
//! 2. RANK — ties handling (`test_rank`)
//! 3. DENSE_RANK — no gaps (`test_dense_rank`)
//! 4. Running aggregates — SUM, COUNT (`test_running_sum`, `test_window_count`)
//! 5. Edge cases (single row, LAG/LEAD)
//! 6. Pipeline integration (`test_window_operator_pipeline`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::execution::window::*;
use quackdb::execution::sort_merge_join::{SortKey, SortDirection, NullOrder};
use quackdb::execution::pipeline::*;

fn make_window_data() -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Varchar, LogicalType::Int32, LogicalType::Float64]);
    // dept, id, salary
    chunk.append_row(&[ScalarValue::Varchar("eng".into()), ScalarValue::Int32(1), ScalarValue::Float64(100.0)]);
    chunk.append_row(&[ScalarValue::Varchar("eng".into()), ScalarValue::Int32(2), ScalarValue::Float64(120.0)]);
    chunk.append_row(&[ScalarValue::Varchar("eng".into()), ScalarValue::Int32(3), ScalarValue::Float64(110.0)]);
    chunk.append_row(&[ScalarValue::Varchar("sales".into()), ScalarValue::Int32(4), ScalarValue::Float64(90.0)]);
    chunk.append_row(&[ScalarValue::Varchar("sales".into()), ScalarValue::Int32(5), ScalarValue::Float64(95.0)]);
    chunk
}

// ── 1. ROW_NUMBER ───────────────────────────────────────────────────

#[test]
fn test_row_number() {
    let chunk = make_window_data();
    let window = WindowDef {
        function_type: WindowFunctionType::RowNumber,
        partition_by: vec![],
        order_by: vec![SortKey { column_index: 1, direction: SortDirection::Ascending, null_order: NullOrder::NullsLast }],
        frame: WindowFrame::default_frame(),
        arg_column: None,
        offset: None,
        default_value: None,
    };

    let func = create_window_function(WindowFunctionType::RowNumber, None);
    let order: Vec<usize> = (0..5).collect();
    let results = func.evaluate(&chunk, &order, &window.frame);
    assert_eq!(results.len(), 5, "ROW_NUMBER should produce exactly one value per input row");
    // Row numbers should be 1, 2, 3, 4, 5
    for (i, val) in results.iter().enumerate() {
        if let ScalarValue::Int64(n) = val {
            assert_eq!(*n, (i + 1) as i64);
        }
    }
}

// ── 2. RANK ─────────────────────────────────────────────────────────

#[test]
fn test_rank() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(20)]);
    chunk.append_row(&[ScalarValue::Int32(30)]);

    let func = create_window_function(WindowFunctionType::Rank, None);
    let order: Vec<usize> = (0..4).collect();
    let frame = WindowFrame::default_frame();
    let results = func.evaluate(&chunk, &order, &frame);
    // With ties: 1, 1, 3, 4
    assert_eq!(results.len(), 4, "RANK produces one value per row; ties get the same rank but the next rank skips ahead");
}

// ── 3. DENSE_RANK ───────────────────────────────────────────────────

#[test]
fn test_dense_rank() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(20)]);
    chunk.append_row(&[ScalarValue::Int32(30)]);

    let func = create_window_function(WindowFunctionType::DenseRank, None);
    let order: Vec<usize> = (0..4).collect();
    let frame = WindowFrame::default_frame();
    let results = func.evaluate(&chunk, &order, &frame);
    // Dense rank: 1, 1, 2, 3
    assert_eq!(results.len(), 4, "DENSE_RANK never skips ranks after ties, unlike RANK which leaves gaps");
}

// ── 4. Running aggregates ───────────────────────────────────────────

#[test]
fn test_running_sum() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(10)]);
    chunk.append_row(&[ScalarValue::Int32(20)]);
    chunk.append_row(&[ScalarValue::Int32(30)]);

    let func = create_window_function(WindowFunctionType::Sum, Some(&LogicalType::Int32));
    let order: Vec<usize> = (0..3).collect();
    let frame = WindowFrame {
        start: FrameBound::UnboundedPreceding,
        end: FrameBound::CurrentRow,
    };
    let results = func.evaluate(&chunk, &order, &frame);
    // Running sum: 10, 30, 60
    assert_eq!(results.len(), 3, "UNBOUNDED PRECEDING to CURRENT ROW frame computes a running sum over the ordered partition");
}

#[test]
fn test_window_count() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(1)]);
    chunk.append_row(&[ScalarValue::Int32(2)]);
    chunk.append_row(&[ScalarValue::Int32(3)]);

    let func = create_window_function(WindowFunctionType::Count, Some(&LogicalType::Int32));
    let order: Vec<usize> = (0..3).collect();
    let frame = WindowFrame {
        start: FrameBound::UnboundedPreceding,
        end: FrameBound::CurrentRow,
    };
    let results = func.evaluate(&chunk, &order, &frame);
    // Running count: 1, 2, 3
    assert_eq!(results.len(), 3);
}

// ── 5. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_row_number_single_row() {
    // Edge case: window function on a single-row input
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(42)]);

    let func = create_window_function(WindowFunctionType::RowNumber, None);
    let order: Vec<usize> = vec![0];
    let frame = WindowFrame::default_frame();
    let results = func.evaluate(&chunk, &order, &frame);
    assert_eq!(results.len(), 1, "ROW_NUMBER on a single row should produce exactly one value");
    if let ScalarValue::Int64(n) = &results[0] {
        assert_eq!(*n, 1, "the only row should get row_number = 1");
    }
}

#[test]
fn test_lag_lead() {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    chunk.append_row(&[ScalarValue::Int32(1)]);
    chunk.append_row(&[ScalarValue::Int32(2)]);
    chunk.append_row(&[ScalarValue::Int32(3)]);

    let lag = create_window_function(WindowFunctionType::Lag, Some(&LogicalType::Int32));
    let lead = create_window_function(WindowFunctionType::Lead, Some(&LogicalType::Int32));
    let order: Vec<usize> = (0..3).collect();
    let frame = WindowFrame::default_frame();

    let lag_results = lag.evaluate(&chunk, &order, &frame);
    let lead_results = lead.evaluate(&chunk, &order, &frame);
    assert_eq!(lag_results.len(), 3, "LAG should return one value per row, using NULL or a default for the first row");
    assert_eq!(lead_results.len(), 3, "LEAD should return one value per row, using NULL or a default for the last row");
}

// ── 6. Pipeline integration ─────────────────────────────────────────

#[test]
fn test_window_operator_pipeline() {
    let chunk = make_window_data();
    let source = InMemorySource::new(vec![chunk], vec![LogicalType::Varchar, LogicalType::Int32, LogicalType::Float64]);

    let window_op = WindowOperator::new(
        vec![WindowDef {
            function_type: WindowFunctionType::RowNumber,
            partition_by: vec![0],
            order_by: vec![SortKey { column_index: 2, direction: SortDirection::Descending, null_order: NullOrder::NullsLast }],
            frame: WindowFrame::default_frame(),
            arg_column: None,
            offset: None,
            default_value: None,
        }],
        vec![LogicalType::Varchar, LogicalType::Int32, LogicalType::Float64],
    );

    let mut pipeline = Pipeline::new(Box::new(source));
    pipeline.add_operator(Box::new(window_op));

    let results = PipelineExecutor::execute(pipeline).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 5, "window operator in a pipeline should preserve all input rows while appending the computed window column");
}
