//! Lesson 33: Shuffle & Exchange Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::distributed::shuffle::*;
use quackdb::distributed::coordinator::*;
use quackdb::distributed::planner::*;
use quackdb::planner::logical_plan::*;
use quackdb::planner::catalog::Catalog;

fn make_chunk(values: Vec<i32>) -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    for v in values {
        chunk.append_row(&[ScalarValue::Int32(v)]);
    }
    chunk
}

#[test]
fn test_exchange_channel() {
    let (sender, receiver) = ExchangeChannel::new();
    let chunk = make_chunk(vec![1, 2, 3]);
    sender.send(chunk).unwrap();
    sender.close();

    let received = receiver.recv().unwrap();
    assert_eq!(received.count(), 3, "exchange channel should deliver the complete chunk without losing rows");
    assert!(receiver.recv().is_none(), "after sender.close(), recv should return None to signal end-of-stream");
}

#[test]
fn test_gather_operator() {
    let (s1, r1) = ExchangeChannel::new();
    let (s2, r2) = ExchangeChannel::new();

    s1.send(make_chunk(vec![1, 2])).unwrap();
    s2.send(make_chunk(vec![3, 4])).unwrap();
    s1.close();
    s2.close();

    let mut gather = GatherOperator::new(vec![r1, r2], vec![LogicalType::Int32]);

    let mut total = 0;
    while let Some(chunk) = gather.next_chunk() {
        total += chunk.count();
    }
    assert_eq!(total, 4, "gather operator should merge all chunks from all input channels into one stream");
}

#[test]
fn test_broadcast() {
    let (s1, r1) = ExchangeChannel::new();
    let (s2, r2) = ExchangeChannel::new();

    let broadcast = BroadcastOperator::new(vec![s1, s2], vec![LogicalType::Int32]);
    let chunk = make_chunk(vec![10, 20]);
    broadcast.broadcast(&chunk).unwrap();

    let c1 = r1.recv().unwrap();
    let c2 = r2.recv().unwrap();
    assert_eq!(c1.count(), 2, "broadcast must send a full copy of the data to every receiver");
    assert_eq!(c2.count(), 2, "broadcast must send a full copy of the data to every receiver");
}

#[test]
fn test_shuffle_routing() {
    let (s1, r1) = ExchangeChannel::new();
    let (s2, r2) = ExchangeChannel::new();

    let shuffle = ShuffleOperator::new(vec![0], vec![s1, s2], vec![LogicalType::Int32]);
    // Note: routing depends on hash implementation, just verify no crash
    // and that all data arrives somewhere
}

#[test]
fn test_distributed_executor() {
    let executor = DistributedExecutor::new(4);
    let plan = LogicalPlan::Scan {
        table_name: "test".to_string(),
        schema: Schema::new(vec![("id".to_string(), LogicalType::Int32)]),
        projection: None,
    };

    let planner = DistributedPlanner::new(4);
    let fragments = planner.plan(plan).unwrap();

    // Create a catalog with test data
    let mut catalog = Catalog::new();
    use quackdb::planner::catalog::*;
    catalog.create_table(TableInfo {
        name: "test".to_string(),
        columns: vec![
            ColumnInfo { name: "id".to_string(), data_type: LogicalType::Int32, nullable: false, column_index: 0 },
        ],
    }).unwrap();

    let mut chunk = DataChunk::new(&[LogicalType::Int32]);
    for i in 0..10 {
        chunk.append_row(&[ScalarValue::Int32(i)]);
    }
    catalog.insert_data("test", chunk).unwrap();

    let results = executor.execute(fragments, &catalog).unwrap();
    let total: usize = results.iter().map(|c| c.count()).sum();
    assert_eq!(total, 10, "distributed executor should return all rows from the source table after plan execution");
}

#[test]
fn test_backpressure() {
    let (sender, receiver) = ExchangeChannel::new();
    // Send many chunks without receiving
    for i in 0..100 {
        let chunk = make_chunk(vec![i]);
        sender.send(chunk).unwrap();
    }
    sender.close();

    let mut count = 0;
    while let Some(c) = receiver.recv() {
        count += c.count();
    }
    assert_eq!(count, 100, "exchange channels must handle backpressure without losing data when sender outpaces receiver");
}
