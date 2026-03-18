# Lesson 26: Cost Optimizer

## What You're Building
A cost-based optimizer comprising three parts: column statistics and cardinality estimation, a cost model that assigns CPU/IO/network costs to plan nodes, and a join order optimizer that uses dynamic programming over relation subsets (DPsub) to find the cheapest join tree. Together, these let the database choose between physically equivalent plans by estimating which one is cheapest to execute.

> **Unified Concept:** This lesson has three layers, but they are a pipeline: statistics feed into the cost model, the cost model feeds into join ordering. You do not need to understand all three at once. Start with statistics (just counting things), then cost model (just arithmetic on those counts), then join ordering (just picking the cheapest combination). Each layer only uses the one before it.

## Concept Recap
Building on Lesson 25: The rule optimizer rewrites plans using structural patterns (e.g., "push filters down"). The cost optimizer goes further -- it enumerates multiple valid plans (especially join orderings) and picks the cheapest one using statistics. The `LogicalPlan` nodes and `Schema` from the planner are what the cardinality estimator and cost model analyze.

## Rust Concepts You'll Need
- [Bitwise Operations](../concepts/bitwise_ops.md) -- `RelationSet` uses a `u64` bitmask to represent sets of relations; operations like union, intersection, and subset enumeration use `|`, `&`, `<<`, and `count_ones()`
- [Collections](../concepts/collections.md) -- `HashMap<RelationSet, (LogicalPlan, Cost)>` stores the DP table of best plans per subset
- [Type Conversions](../concepts/type_conversions.md) -- converting between row counts (`u64`), selectivity (`f64`), and cost components

## Key Patterns

### Bitmask for Set Representation
A u64 can represent a set of up to 64 elements. Bit `i` being set means element `i` is in the set. This is far more efficient than `HashSet<usize>` for small sets. Think of it like a row of light switches -- each switch represents a table, and "on" means that table is included in the subset.

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
DPsub builds optimal plans bottom-up. Start with single-relation sets (base cases). For each larger subset, try all ways to split it into two non-empty complementary parts, look up the best plan for each part, and keep the cheapest combined plan. This is like planning a road trip -- you figure out the best route between every pair of cities first, then combine those into longer journeys.

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
A cost has CPU, IO, and network components. The `total()` function weights them (IO is 10x CPU, network is 100x CPU). Different physical operators have characteristic cost signatures -- a scan is mostly IO, a hash join is mostly CPU for the build side. This is like estimating a project budget with labor, materials, and shipping as separate line items.

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

## Common Mistakes
- **Division by zero in selectivity calculations.** If `distinct_count` is 0 or `max_value == min_value`, your formulas will panic or produce `NaN`. Always guard against these edge cases.
- **Forgetting the base case in DPsub.** Single-relation subsets must be seeded into the DP table with their scan plans and scan costs before you start iterating over larger subsets. Without base cases, every lookup will fail.
- **Iterating subsets in the wrong order.** DPsub must process subsets from smallest to largest so that when you split a subset, both halves are already in the table. Iterate by increasing `count_ones()`.

## Step-by-Step Implementation Order
1. Start with `ColumnStatistics::new()` -- initialize with the given `total_count`, set `distinct_count` to `total_count`, `null_count` to 0, min/max to `None`, histogram to `None`.
2. Implement `equality_selectivity()` -- return `1.0 / distinct_count as f64`. Guard against division by zero.
3. Implement `selectivity()` -- for range predicates like `>` and `<`, use `(value - min) / (max - min)` or its complement. Clamp results to [0.0, 1.0].
4. Implement `CardinalityEstimator::estimate()` -- match on plan nodes. Scan returns the table's `row_count`. Filter multiplies the child's cardinality by the predicate's selectivity. Join multiplies both sides (or use a join selectivity heuristic).
5. Implement `CostModel` static methods -- `scan_cost` is proportional to rows (mostly IO), `sort_cost` is `n * log2(n)` (CPU), `hash_join_cost` is `build_rows` (CPU for build) + `probe_rows` (CPU for probe), `merge_join_cost` is linear in both inputs.
6. Implement `Cost::zero()`, `Cost::add()`, and `Cost::total()` -- zero is the identity element, add sums each component, total applies the weights.
7. Implement `CostModel::estimate()` -- recursively sum child costs plus the cost of the current node.
8. Implement `RelationSet` methods -- `singleton`, `union`, `count`, `is_subset_of`, and critically `subsets()` using the bitmask subset enumeration trick.
9. Implement `JoinOrderOptimizer::optimize()` -- iterate over all subsets of the full relation set in order of increasing size. For each subset, try all splits into two complementary non-empty parts, compute the cost, and keep the minimum.

## Reading the Tests
- **`test_column_statistics_new`** creates a `ColumnStatistics` with 1000 rows and checks that `total_count` is 1000. This is a simple constructor sanity check confirming your initialization is correct.
- **`test_column_statistics_selectivity`** checks that `equality_selectivity()` returns roughly `1/100 = 0.01` for 100 distinct values, and that `selectivity(">", 500.0)` for a [0, 1000] range returns roughly 0.5, and `selectivity("<", 100.0)` returns roughly 0.1. This pins down your selectivity formulas and validates the range calculation.
- **`test_cardinality_estimation_scan`** creates a scan on "users" with 1000 rows and expects the estimator to return 1000. This confirms your scan base case uses the table statistics directly.
- **`test_cardinality_estimation_filter`** adds an equality filter on a table with 100 distinct values out of 1000 rows. It expects the result to be less than 1000 but greater than 0. This validates that your filter estimation multiplies by selectivity.
- **`test_cost_model_scan`** and **`test_cost_model_sort`** verify that scans and sorts produce non-zero costs, and that sorting 10000 rows costs more than sorting 1000 rows. This confirms your cost formulas are monotonically increasing with input size.
- **`test_cost_model_hash_join`** checks that `hash_join_cost(1000, 10000)` produces a non-zero total. This validates the build+probe cost formula.
- **`test_cost_addition`** and **`test_cost_zero`** test the Cost arithmetic: adding two costs sums each component, and `Cost::zero()` returns all zeros. These are the building blocks for recursive cost estimation.
- **`test_relation_set`** tests basic set operations (singleton, union, count, is_subset_of). **`test_relation_set_subsets`** creates set `0b111` (3 elements) and expects exactly 7 non-empty subsets. This confirms the bitmask enumeration produces all `2^n - 1` subsets.
- **`test_join_order_two_tables`** and **`test_join_order_four_tables`** run the full DPsub optimizer on 2- and 4-table joins respectively, expecting a valid plan to be returned. The 4-table test with a chain of join edges is the real stress test for your DP logic.
- **`test_merge_join_cost_cheaper`** checks that merge join cost is positive for equal-sized inputs. This validates that you have a separate cost formula for merge joins.

## What Comes Next
With query optimization complete, Part VII tackles **transactions and durability** --
how to make the database reliable under concurrent access and crashes. Lesson 27
implements MVCC (Multi-Version Concurrency Control) for snapshot isolation, reusing
the `DataChunk` and `ScalarValue` types you know well. Lesson 28 adds write-ahead
logging for crash recovery. These lessons shift focus from performance to correctness
guarantees.
