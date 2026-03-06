# Lesson 34: Adaptive Execution

## What You're Building
An adaptive query execution layer that adjusts strategies at runtime based on observed data characteristics. The centerpiece is a Bloom filter -- a probabilistic data structure that can quickly test set membership with no false negatives but a controlled false positive rate. This enables runtime filter pushdown: after building one side of a join, construct a Bloom filter of join keys and use it to pre-filter the other side. The lesson also covers adaptive parallelism, which dynamically scales worker count based on runtime statistics like throughput and cardinality.

## Rust Concepts You'll Need
- [Bitwise Operations](../concepts/bitwise_ops.md) -- Bloom filters store bits in a Vec<u64>; you will set and test individual bits using shifts and masks
- [Collections](../concepts/collections.md) -- Vec<u64> as the bit array backing the Bloom filter; RuntimeStatistics as a plain data struct
- [Closures](../concepts/closures.md) -- hashing with multiple seeds, adaptive threshold functions

## Key Patterns

### Bloom Filter Bit Manipulation
A Bloom filter is a bit array where each insert sets K bits (from K hash functions) and each lookup checks if all K bits are set. The key operations are: computing which u64 word a bit lives in, and which bit within that word.

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
Bloom filters need K independent hash functions. A common trick is to compute two base hashes and derive K hashes as linear combinations: h_i = h1 + i * h2.

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
Monitor runtime metrics and switch strategies when thresholds are crossed. The key is defining clear decision boundaries.

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

## Step-by-Step Implementation Order
1. Start with `BloomFilter::new()` -- calculate optimal num_bits from expected_items and false_positive_rate (formula: -n*ln(p) / (ln2)^2), and num_hashes (formula: (num_bits/n) * ln2); allocate Vec<u64> with (num_bits + 63) / 64 words
2. Implement `BloomFilter::hash()` -- produce a hash of the value bytes using the seed; a simple FNV or multiplicative hash seeded differently for each hash function works well
3. Implement `BloomFilter::insert()` -- for each of num_hashes seeds, compute hash, modulo num_bits to get bit position, set the bit in the appropriate u64 word
4. Implement `BloomFilter::might_contain()` -- same as insert but test bits instead of setting them; return false if any bit is unset
5. Implement `AdaptiveJoinOperator::new()` -- initialize with empty stats, no bloom filter, store threshold
6. Implement `AdaptiveJoinOperator::execute()` -- track rows in build_count; if build_count exceeds threshold, skip bloom filter construction; otherwise build one from the input keys
7. Implement `AdaptiveParallelism::new()` -- set current_workers to min_workers
8. Implement `AdaptiveParallelism::adjust()` -- examine stats (rows_processed, execution_time_us) to decide whether to scale up or down within bounds
9. Watch out for: bit indexing must handle the word/bit split correctly (divide by 64 for word index, modulo 64 for bit position); the hash function must be deterministic for the same seed+value pair

## Reading the Tests
- **`test_bloom_filter_false_positive_rate`** inserts 1000 values (0..1000), then tests 1000 non-inserted values (1000..2000). It asserts the false positive rate is under 5%. This verifies your bit sizing and hash functions produce a working probabilistic filter.
- **`test_adaptive_parallelism`** creates an AdaptiveParallelism(1, 8), feeds it high-throughput stats (1M rows), and asserts the returned worker count is between 1 and 8. The companion `test_adaptive_parallelism_scale_down` feeds low-volume stats and asserts workers <= 4, confirming your adjustment logic responds to workload characteristics.
