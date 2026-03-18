# Lesson 26: Cost Optimizer

## What You're Building
A cost-based optimizer comprising three parts: column statistics and cardinality estimation, a cost model that assigns CPU/IO/network costs to plan nodes, and a join order optimizer that uses dynamic programming over relation subsets (DPsub) to find the cheapest join tree. Together, these let the database choose between physically equivalent plans by estimating which one is cheapest to execute.

## Rust Concepts You'll Need
- [Bitwise Operations](../concepts/bitwise_ops.md) -- `RelationSet` uses a `u64` bitmask to represent sets of relations; operations like union, intersection, and subset enumeration use `|`, `&`, `<<`, and `count_ones()`
- [Collections](../concepts/collections.md) -- `HashMap<RelationSet, (LogicalPlan, Cost)>` stores the DP table of best plans per subset
- [Type Conversions](../concepts/type_conversions.md) -- converting between row counts (`u64`), selectivity (`f64`), and cost components

## Key Patterns

### Bitmask for Set Representation
A u64 can represent a set of up to 64 elements. Bit `i` being set means element `i` is in the set. This is far more efficient than `HashSet<usize>` for small sets.

```rust
// Analogy: tracking which weekdays a shop is open (NOT the QuackDB solution)
struct Weekdays(u8); // 7 bits for Mon-Sun

impl Weekdays {
    fn includes(&self, day: usize) -> bool { self.0 & (1 << day) != 0 }
    fn union(&self, other: &Weekdays) -> Weekdays { Weekdays(self.0 | other.0) }
    fn count(&self) -> u32 { self.0.count_ones() }

    // Enumerate all non-empty subsets using Gosper's hack
    fn subsets(&self) -> Vec<Weekdays> {
        let mut result = Vec::new();
        let mut sub = self.0;
        while sub > 0 {
            result.push(Weekdays(sub));
            sub = (sub - 1) & self.0; // next subset
        }
        result
    }
}
```

The subset enumeration trick (`sub = (sub - 1) & full_set`) is critical for the DPsub algorithm. It iterates through all non-empty subsets of a bitmask in decreasing order.

### Dynamic Programming Over Subsets
DPsub builds optimal plans bottom-up. Start with single-relation sets (base cases). For each larger subset, try all ways to split it into two non-empty complementary parts, look up the best plan for each part, and keep the cheapest combined plan.

```rust
// Analogy: finding the cheapest way to parenthesize matrix multiplications
// For matrices M0, M1, M2, try (M0*M1)*M2 vs M0*(M1*M2)
// DPsub generalizes this to set partitions rather than contiguous ranges
fn best_split(set_size: usize) -> usize {
    // For each subset S of {0..n}:
    //   For each way to split S into (L, S\L):
    //     cost = dp[L] + dp[S\L] + combine_cost
    //     dp[S] = min over all splits
    0 // placeholder
}
```

### Cost Model with Weighted Components
A cost has CPU, IO, and network components. The `total()` function weights them (IO is 10x CPU, network is 100x CPU). Different physical operators have characteristic cost signatures -- a scan is mostly IO, a hash join is mostly CPU for the build side.

```rust
// Analogy: estimating delivery cost with fuel, tolls, and time
struct DeliveryCost { fuel: f64, tolls: f64, time_hours: f64 }

impl DeliveryCost {
    fn total(&self) -> f64 {
        self.fuel + self.tolls * 2.0 + self.time_hours * 50.0
    }
    fn add(&self, other: &DeliveryCost) -> DeliveryCost {
        DeliveryCost {
            fuel: self.fuel + other.fuel,
            tolls: self.tolls + other.tolls,
            time_hours: self.time_hours + other.time_hours,
        }
    }
}
```

## Step-by-Step Implementation Order
1. Start with `ColumnStatistics::new()` -- initialize with the given `total_count`, set `distinct_count` to `total_count`, `null_count` to 0, min/max to `None`, histogram to `None`.
2. Implement `equality_selectivity()` -- return `1.0 / distinct_count as f64`. Guard against division by zero.
3. Implement `selectivity()` -- for range predicates like `>` and `<`, use `(value - min) / (max - min)` or its complement. Clamp results to [0.0, 1.0].
4. Implement `CardinalityEstimator::estimate()` -- match on plan nodes. Scan returns the table's `row_count`. Filter multiplies the child's cardinality by the predicate's selectivity. Join multiplies both sides (or use a join selectivity heuristic).
5. Implement `CostModel` static methods -- `scan_cost` is proportional to rows (mostly IO), `sort_cost` is `n * log2(n)` (CPU), `hash_join_cost` is `build_rows` (CPU for build) + `probe_rows` (CPU for probe).
6. Implement `CostModel::estimate()` -- recursively sum child costs plus the cost of the current node.
7. Implement `RelationSet::subsets()` -- use the bitmask subset enumeration trick shown above.
8. Implement `JoinOrderOptimizer::dp_sub()` -- iterate over all subsets of the full relation set in order of increasing size. For each subset, try all splits into two complementary non-empty parts, compute the cost, and keep the minimum.
9. Watch out for the base case in DP -- single-relation subsets should map directly to their scan plans with scan cost.

## Reading the Tests
- **`test_column_statistics_selectivity`** checks that `equality_selectivity()` returns roughly `1/100 = 0.01` for 100 distinct values, and that `selectivity(">", 500.0)` for a [0, 1000] range returns roughly 0.5. This pins down your selectivity formulas.
- **`test_relation_set_subsets`** creates a set `0b111` (3 elements) and expects `subsets()` to return exactly 7 non-empty subsets. This confirms the bitmask enumeration produces all `2^n - 1` subsets.
- **`test_cost_model_sort`** verifies that sorting 10000 rows costs more than sorting 1000 rows, confirming your sort cost is monotonically increasing.

## What Comes Next
With query optimization complete, Part VII tackles **transactions and durability** —
how to make the database reliable under concurrent access and crashes. Lesson 27
implements MVCC (Multi-Version Concurrency Control) for snapshot isolation, reusing
the `DataChunk` and `ScalarValue` types you know well. Lesson 28 adds write-ahead
logging for crash recovery. These lessons shift focus from performance to correctness
guarantees.
