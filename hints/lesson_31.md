# Lesson 31: Partitioning

## What You're Building
A data partitioning system that distributes rows across multiple partitions using hash, range, or round-robin strategies. Partitioning is fundamental to distributed databases -- it determines how data is spread across nodes, enables parallel scans, and when combined with partition pruning, allows the engine to skip irrelevant partitions entirely. This lesson also introduces a partition pruner that eliminates partitions based on query predicates.

## Rust Concepts You'll Need
- [Collections](../concepts/collections.md) -- Vec<Vec<DataChunk>> for multi-level partition storage, HashMap-like routing of rows to partition buckets
- [Iterators](../concepts/iterators.md) -- iterating over rows to classify them, flat_map/chain for merging partitions during scan_all
- [Enums and Matching](../concepts/enums_and_matching.md) -- PartitionScheme is an enum with data-carrying variants; match on it to select the routing logic

## Key Patterns

### Hash-Based Routing
Compute a hash of the partition key, then use modulo to assign rows to buckets. This guarantees all rows with the same key land in the same partition.

```rust
// Analogy: routing log entries to file shards (NOT the QuackDB solution)
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn route_to_shard<T: Hash>(item: &T, num_shards: usize) -> usize {
    let mut hasher = DefaultHasher::new();
    item.hash(&mut hasher);
    (hasher.finish() as usize) % num_shards
}

fn partition_logs(logs: &[String], num_shards: usize) -> Vec<Vec<&String>> {
    let mut shards = vec![Vec::new(); num_shards];
    for log in logs {
        let shard = route_to_shard(log, num_shards);
        shards[shard].push(log);
    }
    shards
}
```

### Range Partitioning with Boundaries
Given sorted boundary values [25, 50, 75], values are routed: <25 goes to partition 0, 25..50 to partition 1, 50..75 to partition 2, >=75 to partition 3. This creates N+1 partitions for N boundaries.

```rust
// Analogy: assigning grades by score ranges (NOT the QuackDB solution)
fn assign_grade(score: i32, boundaries: &[i32]) -> usize {
    for (i, &boundary) in boundaries.iter().enumerate() {
        if score < boundary {
            return i;
        }
    }
    boundaries.len() // last bucket
}

let boundaries = vec![60, 70, 80, 90];
assert_eq!(assign_grade(55, &boundaries), 0); // F
assert_eq!(assign_grade(75, &boundaries), 2); // B
assert_eq!(assign_grade(95, &boundaries), 4); // A+
```

## Step-by-Step Implementation Order
1. Start with `Partitioner::num_partitions()` -- match on the scheme: Hash and RoundRobin carry num_partitions directly; Range has boundaries.len() + 1 partitions
2. Implement `partition_for_row()` -- extract the partition key value(s) from the chunk at the given row, then route: hash the value and modulo for Hash, compare against boundaries for Range, use a counter modulo for RoundRobin
3. Implement `partition()` -- create `num_partitions` empty DataChunks, iterate over rows calling `partition_for_row`, append each row to the correct partition chunk
4. Implement `PartitionedTable::new()` -- initialize with the right number of empty partition vecs
5. Implement `insert()` -- create a Partitioner from the scheme, call partition(), and push resulting chunks into the corresponding partition vecs
6. Implement `scan_all()` -- flatten all partitions into a single Vec<DataChunk>
7. Implement `scan_partition()` -- return a slice of the requested partition's chunks
8. Implement `repartition()` -- collect all data via scan_all, update the scheme, re-insert everything
9. Implement `PartitionPruner::prune()` -- for Hash, hash the value and return only the matching partition; for Range, find which partition the value falls into; for RoundRobin, return all partitions (cannot prune)
10. Watch out for: Range partitioning with N boundaries produces N+1 partitions; hashing ScalarValue requires consistent hashing across calls

## Reading the Tests
- **`test_partition_pruning_range`** creates range boundaries [25, 50, 75] and prunes for value 30. It expects exactly one partition (index 1, between 25 and 50). This confirms your range pruner correctly identifies which partition contains a given value.
- **`test_repartition`** inserts 100 rows into a 2-partition table, then repartitions to 8 partitions and verifies total row count is still 100. This ensures repartition collects all data and redistributes without loss.
