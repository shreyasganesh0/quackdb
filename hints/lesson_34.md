# Lesson 34: Adaptive Execution

## What You're Building
An adaptive query execution layer that adjusts strategies at runtime based on observed data characteristics. The centerpiece is a Bloom filter -- a probabilistic data structure that can quickly test set membership with no false negatives but a controlled false positive rate. This enables runtime filter pushdown: after building one side of a join, construct a Bloom filter of join keys and use it to pre-filter the other side. The lesson also covers adaptive parallelism, which dynamically scales worker count based on runtime statistics like throughput and cardinality.

## Concept Recap
Building on Lesson 29 (Morsel Parallelism) and Lessons 11-12 (Hash Join): The Bloom filter is conceptually a lightweight version of the hash table you built for hash joins -- instead of storing full values, it stores just a few bits per entry. Adaptive parallelism extends the static worker count from Lesson 29's ParallelPipelineExecutor, adjusting it dynamically based on observed throughput. The runtime statistics (row counts, timing) connect back to the cost model from Lesson 26.

## Rust Concepts You'll Need
- [Bitwise Operations](../concepts/bitwise_ops.md) -- Bloom filters store bits in a Vec<u64>; you will set and test individual bits using shifts and masks
- [Collections](../concepts/collections.md) -- Vec<u64> as the bit array backing the Bloom filter; RuntimeStatistics as a plain data struct
- [Closures](../concepts/closures.md) -- hashing with multiple seeds, adaptive threshold functions

## Key Patterns

### Bloom Filter Bit Manipulation
A Bloom filter is a bit array where each insert sets K bits (from K hash functions) and each lookup checks if all K bits are set. The key operations are: computing which u64 word a bit lives in, and which bit within that word. Think of it like a hotel check-in board with colored pins -- to check in, you place pins at K specific positions. To check if someone is here, you look at those K positions. If all have pins, they are probably here. If any pin is missing, they are definitely not.

```rust
// Analogy: a simple attendance tracker using bits (NOT the QuackDB solution)
struct AttendanceBoard {
    bits: Vec<u64>,
    capacity: usize,
}

impl AttendanceBoard {
    fn new(num_seats: usize) -> Self {
        let num_words = (num_seats + 63) / 64;
        Self { bits: vec![0u64; num_words], capacity: num_seats }
    }

    fn mark_present(&mut self, seat: usize) {
        let word_idx = seat / 64;
        let bit_idx = seat % 64;
        self.bits[word_idx] |= 1u64 << bit_idx;
    }

    fn is_present(&self, seat: usize) -> bool {
        let word_idx = seat / 64;
        let bit_idx = seat % 64;
        (self.bits[word_idx] & (1u64 << bit_idx)) != 0
    }
}
```

### Multiple Hash Functions via Seeded Hashing
Bloom filters need K independent hash functions. A common trick is to compute two base hashes and derive K hashes as linear combinations: h_i = h1 + i * h2. This avoids implementing K separate hash functions while maintaining good distribution. It is like creating multiple fingerprints from a single document by highlighting different words each time.

```rust
// Analogy: generating multiple fingerprints for document deduplication (NOT the QuackDB solution)
fn multi_hash(data: &[u8], num_hashes: usize) -> Vec<u64> {
    let mut results = Vec::with_capacity(num_hashes);
    for seed in 0..num_hashes {
        let mut h: u64 = seed as u64;
        for &byte in data {
            h = h.wrapping_mul(6364136223846793005).wrapping_add(byte as u64);
        }
        results.push(h);
    }
    results
}
```

### Adaptive Strategy Switching
Monitor runtime metrics and switch strategies when thresholds are crossed. The key is defining clear decision boundaries. This is like a thermostat -- it monitors temperature and switches between heating and cooling based on thresholds, without human intervention.

```rust
// Analogy: adaptive video streaming quality (NOT the QuackDB solution)
struct StreamController {
    min_quality: usize,
    max_quality: usize,
    current: usize,
}

impl StreamController {
    fn adjust(&mut self, bandwidth_mbps: f64) -> usize {
        self.current = if bandwidth_mbps > 10.0 {
            self.max_quality
        } else if bandwidth_mbps > 2.0 {
            (self.min_quality + self.max_quality) / 2
        } else {
            self.min_quality
        };
        self.current
    }
}
```

## Common Mistakes
- **Bit indexing off by one.** The word index is `bit_position / 64` and the bit within that word is `bit_position % 64`. Mixing these up or using the wrong modulus will set/check the wrong bits, causing both false negatives (fatal) and excessive false positives.
- **Using the wrong formula for optimal Bloom filter size.** The optimal number of bits is `-n * ln(p) / (ln2)^2` where n is expected items and p is the desired false positive rate. The optimal number of hash functions is `(num_bits / n) * ln(2)`. Getting these wrong will either waste memory or produce unacceptably high false positive rates.
- **Scaling workers beyond what the workload justifies.** Adaptive parallelism should scale down for small workloads where thread overhead exceeds the benefit. Use rows_processed and execution_time to compute throughput and make scaling decisions proportional to the work available.

## Step-by-Step Implementation Order
1. Start with `BloomFilter::new()` -- calculate optimal num_bits from expected_items and false_positive_rate (formula: -n*ln(p) / (ln2)^2), and num_hashes (formula: (num_bits/n) * ln2); allocate Vec<u64> with (num_bits + 63) / 64 words.
2. Implement `BloomFilter::hash()` -- produce a hash of the value bytes using the seed; a simple FNV or multiplicative hash seeded differently for each hash function works well.
3. Implement `BloomFilter::insert()` -- for each of num_hashes seeds, compute hash, modulo num_bits to get bit position, set the bit in the appropriate u64 word.
4. Implement `BloomFilter::might_contain()` -- same as insert but test bits instead of setting them; return false if any bit is unset.
5. Implement `RuntimeStatistics` with Default trait -- all fields start at 0.
6. Implement `AdaptiveJoinOperator::new()` -- initialize with empty stats, no bloom filter, store threshold.
7. Implement `AdaptiveJoinOperator::execute()` -- track rows in build_count; if build_count exceeds threshold, skip bloom filter construction; otherwise build one from the input keys.
8. Implement `AdaptiveParallelism::new()` -- set current_workers to min_workers.
9. Implement `AdaptiveParallelism::adjust()` -- examine stats (rows_processed, execution_time_us) to decide whether to scale up or down within bounds.

## Reading the Tests
- **`test_bloom_filter_basic`** inserts "hello" and "world", then checks that both are found. This is the simplest test verifying that inserted elements are always found (no false negatives). It is your first validation target.
- **`test_bloom_filter_empty`** checks that an empty Bloom filter does not report containing "anything". This verifies that a freshly constructed filter with no insertions has all bits unset.
- **`test_bloom_filter_false_positive_rate`** inserts 1000 values (0..1000), then tests 1000 non-inserted values (1000..2000). It asserts the false positive rate is under 5%. This verifies your bit sizing formula and hash functions produce a working probabilistic filter. If this fails, check your optimal size calculation.
- **`test_bloom_filter_runtime_pushdown`** inserts 100 values, then checks all 200 values (0..200). It expects at least 100 to pass (the inserted ones) but fewer than 200 (some non-inserted ones should be rejected). This simulates the runtime filter pushdown use case where the Bloom filter eliminates some but not all probe-side rows.
- **`test_runtime_statistics`** checks that `RuntimeStatistics::default()` initializes all fields to 0. This validates the Default trait implementation.
- **`test_adaptive_join_operator`** creates an AdaptiveJoinOperator with a threshold of 10000 and executes a small chunk. This is a smoke test verifying the operator can be constructed and executed without panicking.
- **`test_adaptive_parallelism`** creates an AdaptiveParallelism(1, 8), feeds it high-throughput stats (1M rows in 1ms), and asserts the returned worker count is between 1 and 8. This verifies that high throughput triggers scaling up.
- **`test_adaptive_parallelism_scale_down`** feeds low-volume stats (10 rows, 100us) and asserts workers <= 4. This confirms your adjustment logic responds to workload characteristics by scaling down for small workloads, avoiding wasteful thread overhead.
