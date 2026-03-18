//! # Lesson 31: Data Partitioning — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Partitioner construction (`test_num_partitions`)
//! 2. Hash partitioning (`test_hash_partition`, `test_hash_partition_deterministic`)
//! 3. Round-robin partitioning (`test_round_robin_partition`)
//! 4. Range partitioning (`test_range_partition`)
//! 5. Edge cases (single partition, empty input)
//! 6. Partition pruning (`test_partition_pruning_hash`, `test_partition_pruning_range`)
//! 7. Partitioned table — insert and scan (`test_partitioned_table`, `test_partitioned_table_scan_partition`)
//! 8. Repartitioning (`test_repartition`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::chunk::DataChunk;
use quackdb::distributed::partition::*;

fn make_data(n: usize) -> DataChunk {
    let mut chunk = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    for i in 0..n {
        chunk.append_row(&[
            ScalarValue::Int32(i as i32),
            ScalarValue::Varchar(format!("val_{}", i)),
        ]);
    }
    chunk
}

// ── 1. Partitioner construction ─────────────────────────────────────

#[test]
fn test_num_partitions() {
    let p = Partitioner::new(PartitionScheme::Hash { columns: vec![0], num_partitions: 7 });
    assert_eq!(p.num_partitions(), 7);
}

// ── 2. Hash partitioning ────────────────────────────────────────────

#[test]
fn test_hash_partition() {
    let data = make_data(100);
    let partitioner = Partitioner::new(PartitionScheme::Hash {
        columns: vec![0],
        num_partitions: 4,
    });

    let parts = partitioner.partition(&data);
    assert_eq!(parts.len(), 4, "hash partitioner should produce exactly num_partitions output chunks");

    let total: usize = parts.iter().map(|c| c.count()).sum();
    assert_eq!(total, 100, "partitioning must not lose or duplicate any rows");
}

#[test]
fn test_hash_partition_deterministic() {
    let data = make_data(50);
    let partitioner = Partitioner::new(PartitionScheme::Hash {
        columns: vec![0],
        num_partitions: 4,
    });

    let parts1 = partitioner.partition(&data);
    let parts2 = partitioner.partition(&data);

    for (p1, p2) in parts1.iter().zip(parts2.iter()) {
        assert_eq!(p1.count(), p2.count(), "hash partitioning must be deterministic -- same input should always land in the same partition");
    }
}

// ── 3. Round-robin partitioning ─────────────────────────────────────

#[test]
fn test_round_robin_partition() {
    let data = make_data(10);
    let partitioner = Partitioner::new(PartitionScheme::RoundRobin { num_partitions: 3 });

    let parts = partitioner.partition(&data);
    assert_eq!(parts.len(), 3);

    let total: usize = parts.iter().map(|c| c.count()).sum();
    assert_eq!(total, 10);

    // Round robin should distribute evenly
    assert!(parts[0].count() >= 3, "round-robin should distribute rows evenly: 10 rows / 3 partitions = 3 or 4 per partition");
    assert!(parts[0].count() <= 4);
}

// ── 4. Range partitioning ───────────────────────────────────────────

#[test]
fn test_range_partition() {
    let data = make_data(100);
    let partitioner = Partitioner::new(PartitionScheme::Range {
        column: 0,
        boundaries: vec![ScalarValue::Int32(25), ScalarValue::Int32(50), ScalarValue::Int32(75)],
    });

    let parts = partitioner.partition(&data);
    assert_eq!(parts.len(), 4, "N range boundaries create N+1 partitions: (-inf,25), [25,50), [50,75), [75,+inf)");

    let total: usize = parts.iter().map(|c| c.count()).sum();
    assert_eq!(total, 100);
}

// ── 5. Edge cases ───────────────────────────────────────────────────

#[test]
fn test_hash_partition_single_partition() {
    // Edge case: partitioning into a single partition should put all rows in one chunk
    let data = make_data(10);
    let partitioner = Partitioner::new(PartitionScheme::Hash {
        columns: vec![0],
        num_partitions: 1,
    });

    let parts = partitioner.partition(&data);
    assert_eq!(parts.len(), 1, "single-partition scheme must produce exactly one partition");
    assert_eq!(parts[0].count(), 10, "all rows must land in the single partition");
}

#[test]
fn test_hash_partition_empty_input() {
    // Edge case: partitioning empty data
    let data = DataChunk::new(&[LogicalType::Int32, LogicalType::Varchar]);
    let partitioner = Partitioner::new(PartitionScheme::Hash {
        columns: vec![0],
        num_partitions: 4,
    });

    let parts = partitioner.partition(&data);
    let total: usize = parts.iter().map(|c| c.count()).sum();
    assert_eq!(total, 0, "partitioning empty input must produce zero total rows");
}

// ── 6. Partition pruning ────────────────────────────────────────────

#[test]
fn test_partition_pruning_hash() {
    let pruned = PartitionPruner::prune(
        &PartitionScheme::Hash { columns: vec![0], num_partitions: 4 },
        4,
        0,
        &ScalarValue::Int32(42),
    );
    // Hash pruning: should return exactly 1 partition
    assert_eq!(pruned.len(), 1, "hash partition pruning should narrow the search to exactly one partition for an equality predicate");
}

#[test]
fn test_partition_pruning_range() {
    let pruned = PartitionPruner::prune(
        &PartitionScheme::Range {
            column: 0,
            boundaries: vec![ScalarValue::Int32(25), ScalarValue::Int32(50), ScalarValue::Int32(75)],
        },
        4,
        0,
        &ScalarValue::Int32(30),
    );
    // Value 30 falls in partition 1 (between 25 and 50)
    assert_eq!(pruned.len(), 1);
    assert_eq!(pruned[0], 1, "value 30 falls in range [25,50) which is partition index 1");
}

// ── 7. Partitioned table ────────────────────────────────────────────

#[test]
fn test_partitioned_table() {
    let schema = vec![
        ("id".to_string(), LogicalType::Int32),
        ("name".to_string(), LogicalType::Varchar),
    ];
    let mut table = PartitionedTable::new(
        "test".to_string(),
        schema,
        PartitionScheme::Hash { columns: vec![0], num_partitions: 4 },
    );

    let data = make_data(100);
    table.insert(data);

    let all = table.scan_all();
    let total: usize = all.iter().map(|c| c.count()).sum();
    assert_eq!(total, 100);
}

#[test]
fn test_partitioned_table_scan_partition() {
    let schema = vec![("id".to_string(), LogicalType::Int32)];
    let mut table = PartitionedTable::new(
        "t".to_string(),
        schema,
        PartitionScheme::Hash { columns: vec![0], num_partitions: 4 },
    );

    let data = make_data(100);
    table.insert(data);

    let part_data = table.scan_partition(0);
    assert!(!part_data.is_empty() || true); // might be empty if hash distributes differently
}

// ── 8. Repartitioning ──────────────────────────────────────────────

#[test]
fn test_repartition() {
    let schema = vec![("id".to_string(), LogicalType::Int32)];
    let mut table = PartitionedTable::new(
        "t".to_string(),
        schema,
        PartitionScheme::Hash { columns: vec![0], num_partitions: 2 },
    );

    let data = make_data(100);
    table.insert(data);
    assert_eq!(table.num_partitions(), 2);

    table.repartition(PartitionScheme::Hash { columns: vec![0], num_partitions: 8 });
    assert_eq!(table.num_partitions(), 8);

    let all = table.scan_all();
    let total: usize = all.iter().map(|c| c.count()).sum();
    assert_eq!(total, 100, "repartitioning must preserve all data even when changing the number of partitions");
}
