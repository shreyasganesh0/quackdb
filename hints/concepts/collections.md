# Collections

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md), [generics](./generics.md)

## Quick Reference
- `Vec::new()` or `vec![1, 2, 3]` -- growable array, like Python's `list`
- `HashMap::new()` + `.insert(k, v)` -- hash table, like Python's `dict`
- `.get(&key)` returns `Option<&V>` -- safe lookup, no panic
- `.entry(key).or_insert(default)` -- insert-or-update pattern (great for aggregations)
- `Vec::with_capacity(n)` -- pre-allocate to avoid repeated reallocations

## Common Compiler Errors

**`error[E0502]: cannot borrow 'v' as mutable because it is also borrowed as immutable`**
You took a reference to a Vec element and then tried to push/modify the Vec.
Fix: clone the value (for Copy types this is automatic), or restructure so the borrow ends before mutation.

**`error[E0277]: the trait bound 'K: Eq' is not satisfied`** (or `Hash`)
You tried to use a type as a HashMap key without the required traits.
Fix: add `#[derive(Eq, Hash, PartialEq)]` to the key type, or use a type that already implements them.

**`error[E0599]: no method named 'entry' found for struct 'Vec'`**
You called a HashMap method on a Vec.
Fix: `entry()` is a `HashMap` method. For Vec, use indexing or `.push()`.

## When You'll Use This
- **Lesson 10 (Buffer Pool):** `HashMap` for the page table mapping page IDs to frame indices
- **Lesson 16 (Hash Aggregate):** `HashMap<Vec<u8>, Vec<AggregateState>>` maps group keys to state
- **Lesson 17 (Hash Join):** `HashMap<Vec<u8>, Vec<usize>>` maps key bytes to row indices
- **Lesson 19 (External Sort):** `Vec` backs both the heap and sorted runs
- **Lesson 23 (Binder & Catalog):** `HashMap<String, TableInfo>` for the catalog
- **Lesson 26 (Cost Optimizer):** `HashMap<RelationSet, (LogicalPlan, Cost)>` for the DP table
- **Lesson 27 (MVCC):** `HashMap<TxnId, Transaction>` for tracking transactions
- **Lesson 30 (Window Functions):** heavy use of Vec for buffering and computing results
- **Lesson 31 (Partitioning):** `Vec<Vec<DataChunk>>` for partition storage
- **Lesson 34 (Adaptive Execution):** `Vec<u64>` as the Bloom filter bit array

## What This Is

Rust's standard library provides growable, heap-allocated collection types that serve the same roles as collections in other languages. The two most commonly used are `Vec<T>` (a growable array) and `HashMap<K, V>` (a hash table). If you know Python, `Vec` is like `list` and `HashMap` is like `dict`. In JavaScript terms, `Vec` is similar to `Array` and `HashMap` to `Map`. In C++, they correspond to `std::vector` and `std::unordered_map`.

The main difference from garbage-collected languages is that Rust collections own their elements. When a `Vec` is dropped, all elements inside it are dropped too. You cannot hold a reference into a `Vec` while simultaneously pushing new elements, because a push might reallocate the internal buffer and invalidate the reference. This is enforced at compile time by the borrow checker and prevents the class of iterator-invalidation bugs that plague C++ code.

Rust collections are generic: `Vec<T>` can hold any type `T`, and `HashMap<K, V>` requires that keys implement `Eq + Hash`. The compiler monomorphizes these -- it generates specialized code for each concrete type you use -- so there is no runtime overhead compared to hand-written type-specific containers. You can also pre-allocate capacity with methods like `Vec::with_capacity` and `HashMap::with_capacity` to avoid repeated reallocations when you know the approximate size in advance.

## Syntax

```rust
use std::collections::HashMap;

fn main() {
    // --- Vec<T> ---

    // Create and populate
    let mut scores: Vec<i32> = Vec::new();
    scores.push(90);
    scores.push(85);
    scores.push(92);

    // Create with initial values using the vec! macro
    let names = vec!["Alice", "Bob", "Carol"];

    // Pre-allocate capacity (avoids reallocations)
    let mut buffer: Vec<u8> = Vec::with_capacity(4096);

    // Access by index (panics if out of bounds)
    let first = scores[0]; // 90

    // Safe access with .get() returning Option<&T>
    let maybe = scores.get(10); // None

    // Length and capacity
    println!("len={}, capacity={}", scores.len(), scores.capacity());

    // --- HashMap<K, V> ---

    // Create and insert
    let mut ages: HashMap<String, u32> = HashMap::new();
    ages.insert("Alice".to_string(), 30);
    ages.insert("Bob".to_string(), 25);

    // Lookup with .get() -- returns Option<&V>
    if let Some(age) = ages.get("Alice") {
        println!("Alice is {age}");
    }

    // Check for key existence
    let has_carol = ages.contains_key("Carol"); // false

    // Iterate over key-value pairs
    for (name, age) in &ages {
        println!("{name}: {age}");
    }
}
```

## Common Patterns

### Building a Column Store with Vecs

In a columnar database, each column is stored as a separate vector. This
pattern uses `Vec::with_capacity` for efficient bulk loading.

```rust
struct ColumnStore {
    ids: Vec<u64>,
    temperatures: Vec<f64>,
    timestamps: Vec<u64>,
}

impl ColumnStore {
    fn with_capacity(n: usize) -> Self {
        ColumnStore {
            ids: Vec::with_capacity(n),
            temperatures: Vec::with_capacity(n),
            timestamps: Vec::with_capacity(n),
        }
    }

    fn push_row(&mut self, id: u64, temp: f64, ts: u64) {
        self.ids.push(id);
        self.temperatures.push(temp);
        self.timestamps.push(ts);
    }

    fn len(&self) -> usize {
        self.ids.len()
    }
}

fn main() {
    let mut store = ColumnStore::with_capacity(1000);
    for i in 0..1000 {
        store.push_row(i, 20.0 + (i as f64) * 0.01, 1700000000 + i);
    }
    println!("loaded {} rows", store.len());
}
```

### The Entry API for Aggregations

The `entry()` API on `HashMap` is Rust's elegant solution for the
"insert-or-update" pattern. It avoids redundant lookups and is perfect for
building aggregation hash tables.

```rust
use std::collections::HashMap;

fn word_frequency(text: &str) -> HashMap<String, usize> {
    let mut freq: HashMap<String, usize> = HashMap::new();
    for word in text.split_whitespace() {
        // entry() returns a view into the slot for this key.
        // or_insert(0) inserts 0 if the key is absent, then returns &mut usize.
        *freq.entry(word.to_lowercase()).or_insert(0) += 1;
    }
    freq
}

fn main() {
    let text = "the quick brown fox jumps over the lazy fox";
    let freq = word_frequency(text);
    assert_eq!(freq["the"], 2);
    assert_eq!(freq["fox"], 2);
    println!("{freq:?}");
}
```

### Draining and Retaining Elements

`Vec::retain` and `Vec::drain` let you filter or remove elements efficiently.
This is useful when pruning stale entries from caches or buffers.

```rust
fn main() {
    // retain: keep only elements matching a predicate (in-place filter)
    let mut values = vec![1, 2, 3, 4, 5, 6, 7, 8];
    values.retain(|&x| x % 2 == 0);
    assert_eq!(values, vec![2, 4, 6, 8]);

    // drain: remove a range and get an iterator over removed elements
    let mut log_buffer = vec!["msg1", "msg2", "msg3", "msg4", "msg5"];
    let flushed: Vec<&str> = log_buffer.drain(..3).collect();
    assert_eq!(flushed, vec!["msg1", "msg2", "msg3"]);
    assert_eq!(log_buffer, vec!["msg4", "msg5"]);

    // extend: append elements from an iterator
    let mut all = vec![1, 2, 3];
    all.extend([4, 5, 6]);
    assert_eq!(all, vec![1, 2, 3, 4, 5, 6]);
}
```

## Gotchas

**1. Borrowing a Vec element while mutating the Vec is not allowed.**
The borrow checker prevents iterator invalidation bugs that silently corrupt
data in C++:
```rust
let mut v = vec![1, 2, 3];
// let first = &v[0];
// v.push(4);          // ERROR: cannot borrow `v` as mutable because it is
//                      //        also borrowed as immutable (via `first`)
// println!("{first}");

// Fix: clone the value, or restructure so the borrow ends before the push.
let first = v[0]; // Copy the i32 (no borrow kept)
v.push(4);        // OK now
```

**2. `HashMap` iteration order is non-deterministic.**
Unlike Python 3.7+ dicts which preserve insertion order, Rust's `HashMap` does
not guarantee any order. If you need ordered iteration, use `BTreeMap` instead:
```rust
use std::collections::BTreeMap;
let mut ordered = BTreeMap::new();
ordered.insert("b", 2);
ordered.insert("a", 1);
// Iteration is always in key order: ("a", 1), ("b", 2)
```

**3. `.remove()` on Vec is O(n); use `.swap_remove()` when order doesn't matter.**
`Vec::remove(i)` shifts all subsequent elements left. If you do not need to
preserve order, `swap_remove` is O(1) -- it swaps the removed element with
the last element:
```rust
let mut v = vec![10, 20, 30, 40];
v.swap_remove(1);  // removes 20 by swapping with 40
assert_eq!(v, vec![10, 40, 30]); // order changed, but O(1)
```

## Related Concepts

- [Ownership and Borrowing](./ownership_and_borrowing.md) -- Vec/HashMap own their elements; borrowing rules apply
- [Generics](./generics.md) -- `Vec<T>` and `HashMap<K, V>` are generic types
- [Iterators](./iterators.md) -- `.iter()`, `.into_iter()`, `.drain()` produce iterators over collections
- [Trait Bounds](./trait_bounds.md) -- HashMap keys require `Eq + Hash`

## Quick Reference

### Vec<T>

| Operation | Method | Time Complexity |
|---|---|---|
| Create empty | `Vec::new()` | O(1) |
| Create with capacity | `Vec::with_capacity(n)` | O(1) |
| From literal | `vec![1, 2, 3]` | O(n) |
| Append | `.push(val)` | Amortized O(1) |
| Pop last | `.pop()` -> `Option<T>` | O(1) |
| Index | `v[i]` or `.get(i)` | O(1) |
| Length | `.len()` | O(1) |
| Remove at index | `.remove(i)` | O(n) |
| Remove unordered | `.swap_remove(i)` | O(1) |
| Filter in-place | `.retain(\|x\| pred)` | O(n) |
| Sort | `.sort()` | O(n log n) |
| Extend | `.extend(iter)` | O(k) |
| Clear | `.clear()` | O(n) drops |

### HashMap<K, V>

| Operation | Method | Time Complexity |
|---|---|---|
| Create | `HashMap::new()` | O(1) |
| Insert | `.insert(k, v)` | Amortized O(1) |
| Lookup | `.get(&k)` -> `Option<&V>` | O(1) avg |
| Contains | `.contains_key(&k)` | O(1) avg |
| Remove | `.remove(&k)` -> `Option<V>` | O(1) avg |
| Entry API | `.entry(k).or_insert(v)` | O(1) avg |
| Length | `.len()` | O(1) |
| Iterate | `for (k, v) in &map` | O(n) |
