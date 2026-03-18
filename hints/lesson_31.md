# Lesson 31: Partitioning

## What You're Building
A data partitioning system that distributes rows across multiple partitions using hash, range, or round-robin strategies. Partitioning is fundamental to distributed databases -- it determines how data is spread across nodes, enables parallel scans, and when combined with partition pruning, allows the engine to skip irrelevant partitions entirely. This lesson also introduces a partition pruner that eliminates partitions based on query predicates.

## Concept Recap
Building on Lesson 29 (Morsel Parallelism): In Lesson 29 you divided DataChunks into morsels for parallel processing on a single machine. Partitioning extends this idea across multiple machines -- instead of splitting work among threads, you split data among nodes. The hash-based routing here is conceptually similar to the hash tables you built in Lessons 11-12 for aggregation and joins, but applied to data placement rather than lookups.

## Rust Concepts You'll Need
- [Collections](../concepts/collections.md) -- Vec<Vec<DataChunk>> for multi-level partition storage, HashMap-like routing of rows to partition buckets
- [Iterators](../concepts/iterators.md) -- iterating over rows to classify them, flat_map/chain for merging partitions during scan_all
- [Enums and Matching](../concepts/enums_and_matching.md) -- PartitionScheme is an enum with data-carrying variants; match on it to select the routing logic

## Key Patterns

### Hash-Based Routing
Compute a hash of the partition key, then use modulo to assign rows to buckets. This guarantees all rows with the same key land in the same partition. It works like mail sorting at a post office -- the zip code (hash) determines which delivery route (partition) the letter goes to.

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
Given sorted boundary values [25, 50, 75], values are routed: <25 goes to partition 0, 25..50 to partition 1, 50..75 to partition 2, >=75 to partition 3. This creates N+1 partitions for N boundaries. Think of it like sorting books onto shelves by page count -- shelf labels mark the cutoff points.

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

### Partition Pruning
When a query has a predicate like `WHERE id = 42`, you can determine which partition contains that value without scanning all partitions. For hash partitioning, hash the value and compute the partition. For range partitioning, binary-search the boundaries. For round-robin, pruning is impossible because there is no data-dependent routing. This is like knowing which filing cabinet drawer to open based on the first letter of a name.

```rust
// Analogy: finding the right drawer in an alphabetical filing cabinet (NOT the QuackDB solution)
fn find_drawer(name: &str, drawer_labels: &[char]) -> usize {
    let first_char = name.chars().next().unwrap_or('a');
    for (i, &label) in drawer_labels.iter().enumerate() {
        if first_char < label { return i; }
    }
    drawer_labels.len()
}
```

## Common Mistakes
- **Off-by-one in range partitioning.** N boundaries produce N+1 partitions, not N. The last partition catches all values >= the highest boundary. If you create only N partitions, the last bucket of data will be lost.
- **Non-deterministic hashing across calls.** If your hash function produces different results for the same value on different calls, the same row could end up in different partitions on insert vs. pruning lookup. Use a consistent hasher.
- **Forgetting to handle empty partitions.** After partitioning, some buckets may be empty. Your scan and repartition logic must handle empty DataChunks gracefully.

## Step-by-Step Implementation Order
1. Start with `Partitioner::num_partitions()` -- match on the scheme: Hash and RoundRobin carry num_partitions directly; Range has boundaries.len() + 1 partitions.
2. Implement `partition_for_row()` -- extract the partition key value(s) from the chunk at the given row, then route: hash the value and modulo for Hash, compare against boundaries for Range, use a counter modulo for RoundRobin.
3. Implement `partition()` -- create `num_partitions` empty DataChunks, iterate over rows calling `partition_for_row`, append each row to the correct partition chunk.
4. Implement `PartitionedTable::new()` -- initialize with the right number of empty partition vecs.
5. Implement `insert()` -- create a Partitioner from the scheme, call partition(), and push resulting chunks into the corresponding partition vecs.
6. Implement `scan_all()` -- flatten all partitions into a single Vec<DataChunk>.
7. Implement `scan_partition()` -- return chunks from the requested partition.
8. Implement `repartition()` -- collect all data via scan_all, update the scheme, re-insert everything.
9. Implement `PartitionPruner::prune()` -- for Hash, hash the value and return only the matching partition; for Range, find which partition the value falls into; for RoundRobin, return all partitions (cannot prune).

## Reading the Tests
- **`test_hash_partition`** partitions 100 rows into 4 hash buckets and checks that exactly 4 partitions are returned with a total of 100 rows. This validates basic hash routing without row loss or duplication.
- **`test_hash_partition_deterministic`** partitions the same 50 rows twice and checks that each partition has the same count both times. This ensures your hash function is deterministic -- critical for partition pruning to work correctly.
- **`test_round_robin_partition`** partitions 10 rows into 3 buckets and checks that each bucket has 3 or 4 rows (even distribution). This validates the round-robin strategy distributes evenly.
- **`test_range_partition`** uses boundaries [25, 50, 75] on 100 rows (0..99) and checks that 4 partitions are created with a total of 100 rows. This confirms N boundaries produce N+1 partitions.
- **`test_partitioned_table`** creates a hash-partitioned table, inserts 100 rows, and scans all partitions to verify the total is 100. This is the end-to-end integration test for PartitionedTable.
- **`test_partitioned_table_scan_partition`** inserts data and then scans a single partition (partition 0). This tests that individual partition access works.
- **`test_partition_pruning_hash`** prunes for value 42 in a 4-partition hash scheme and expects exactly 1 partition returned. This validates that hash pruning narrows to a single bucket.
- **`test_partition_pruning_range`** prunes for value 30 with boundaries [25, 50, 75] and expects partition index 1 (the [25, 50) range). This tests the boundary comparison logic.
- **`test_repartition`** inserts 100 rows into a 2-partition table, repartitions to 8 partitions, and verifies total row count is still 100 and num_partitions is 8. This ensures repartition collects all data and redistributes without loss.
- **`test_num_partitions`** checks that a hash partitioner with num_partitions=7 reports 7. This is a simple getter validation.
