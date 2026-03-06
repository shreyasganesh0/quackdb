# Lesson 16: Hash Aggregation

## What You're Building
A hash-based aggregation engine that groups rows by key columns and computes aggregate functions (SUM, COUNT, AVG, MIN, MAX) per group. The AggregateFunction trait defines the lifecycle of an aggregate computation. The AggregateHashTable maps serialized group keys to vectors of aggregate states. The HashAggregateOperator buffers all input during execution, then produces grouped results during finalization. This is how databases handle GROUP BY clauses efficiently.

## Rust Concepts You'll Need
- [Trait Objects](../concepts/trait_objects.md) -- `create_aggregate` returns `Box<dyn AggregateFunction>`, allowing polymorphic dispatch over different aggregate implementations
- [Collections](../concepts/collections.md) -- `HashMap<Vec<u8>, Vec<AggregateState>>` maps byte-serialized group keys to per-group state
- [Closures](../concepts/closures.md) -- useful when iterating over rows and applying aggregate updates

## Key Patterns

### Trait Object Factory
A factory function returns different concrete types behind a common trait object, selected at runtime by an enum discriminant.

```rust
// Analogy: a shape area calculator factory (NOT the QuackDB solution)
trait AreaCalc {
    fn init(&self) -> f64;
    fn add(&self, state: &mut f64, value: f64);
    fn result(&self, state: &f64) -> f64;
}

struct SumArea;
impl AreaCalc for SumArea {
    fn init(&self) -> f64 { 0.0 }
    fn add(&self, state: &mut f64, value: f64) { *state += value; }
    fn result(&self, state: &f64) -> f64 { *state }
}

struct MaxArea;
impl AreaCalc for MaxArea {
    fn init(&self) -> f64 { f64::NEG_INFINITY }
    fn add(&self, state: &mut f64, value: f64) { if value > *state { *state = value; } }
    fn result(&self, state: &f64) -> f64 { *state }
}

enum CalcType { Sum, Max }

fn create_calc(ct: CalcType) -> Box<dyn AreaCalc> {
    match ct {
        CalcType::Sum => Box::new(SumArea),
        CalcType::Max => Box::new(MaxArea),
    }
}
```

### Hash Table with Byte-Serialized Keys
When group keys can be multiple columns of varying types, serialize them into a `Vec<u8>` for use as HashMap keys. This avoids needing a custom hash for every combination of types.

```rust
// Analogy: caching computed results by composite key (NOT the QuackDB solution)
use std::collections::HashMap;

fn serialize_key(parts: &[&str]) -> Vec<u8> {
    let mut key = Vec::new();
    for part in parts {
        key.extend_from_slice(&(part.len() as u32).to_le_bytes());
        key.extend_from_slice(part.as_bytes());
    }
    key
}

let mut cache: HashMap<Vec<u8>, f64> = HashMap::new();
let key = serialize_key(&["region_a", "2025"]);
cache.entry(key).or_insert(0.0);
```

## Step-by-Step Implementation Order
1. Start with `AggregateState::new()` -- initialize value to Null, count to 0, sum to 0.0
2. Implement `create_aggregate()` -- match on AggregateType and return the appropriate `Box<dyn AggregateFunction>`; each variant needs its own struct implementing the trait
3. Implement the AggregateFunction trait for each variant -- COUNT increments count, SUM adds to sum, AVG tracks both sum and count, MIN/MAX compare values; skip NULL inputs in update
4. Implement `AggregateHashTable::new()` -- store types and initialize an empty HashMap
5. Implement `add_chunk()` -- for each row, serialize the group columns into a `Vec<u8>` key, look up or create the entry, then call update on each aggregate state
6. Implement `finalize()` -- iterate over all groups, call finalize on each aggregate state, and assemble the results into DataChunks
7. Watch out for global aggregation (empty group_types) -- use a single empty key so there is exactly one group

## Reading the Tests
- **`test_aggregate_sum`** creates a hash table with one group column (Int32) and one aggregate (Sum on Int64). It calls `add_chunk` with explicit group and aggregate column indices `(&[0], &[1])`. The assertion checks that group 1 sums to 90 and group 2 sums to 60. This reveals that `add_chunk` must use column indices to extract the right data.
- **`test_aggregate_global`** passes empty group columns `(&[], &[0, 0])` and asserts `group_count() == 1`. This confirms your implementation must handle the "no GROUP BY" case as a single global group.
