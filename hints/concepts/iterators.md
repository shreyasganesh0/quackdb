# Iterators

> **Prerequisites:** [collections](./collections.md)

## What This Is

Iterators in Rust are a zero-cost abstraction for processing sequences of values. Any type that implements the `Iterator` trait provides a `.next()` method that returns `Some(value)` until the sequence is exhausted, then returns `None`. What makes Rust iterators powerful is **chaining**: you compose a pipeline of transformations (`.map()`, `.filter()`, `.take()`, etc.) and the compiler fuses them into a single loop with no intermediate allocations. The result runs as fast as hand-written loop code.

If you know Python, Rust iterators are similar to generator expressions and the `itertools` module, but they are evaluated lazily by default and compiled to optimized machine code rather than interpreted. In JavaScript, they are analogous to array methods like `.map()` and `.filter()`, except Rust iterators are lazy -- nothing happens until you consume the iterator with `.collect()`, `.for_each()`, a `for` loop, or another terminal operation. In C++, they fill the role of `<algorithm>` functions combined with C++20 ranges.

A critical distinction: calling `.iter()` on a collection produces an iterator of **references** (`&T`). Calling `.into_iter()` consumes the collection and yields **owned values** (`T`). Calling `.iter_mut()` produces **mutable references** (`&mut T`). Choosing the right one depends on whether you need to read, consume, or modify the elements.

## Syntax

```rust
fn main() {
    let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

    // Basic for loop -- uses IntoIterator implicitly
    for n in &numbers {
        print!("{n} ");
    }
    println!();

    // Chained iterator: filter even numbers, square them, collect into a new Vec
    let result: Vec<i32> = numbers.iter()
        .filter(|&&n| n % 2 == 0)       // keep even numbers
        .map(|&n| n * n)                 // square each
        .collect();                       // gather into Vec<i32>
    assert_eq!(result, vec![4, 16, 36, 64, 100]);

    // enumerate: get (index, value) pairs
    for (i, val) in numbers.iter().enumerate() {
        if i < 3 {
            println!("index {i} => {val}");
        }
    }

    // zip: combine two iterators element-wise
    let keys = vec!["a", "b", "c"];
    let vals = vec![1, 2, 3];
    let pairs: Vec<(&&str, &i32)> = keys.iter().zip(vals.iter()).collect();
    println!("{pairs:?}");

    // sum, min, max -- terminal operations
    let total: i32 = numbers.iter().sum();
    let smallest = numbers.iter().min(); // Option<&i32>
    println!("sum={total}, min={smallest:?}");
}
```

## Common Patterns

### Transform and Aggregate a Data Column

A common analytical workload is scanning a column, applying a transformation,
and computing an aggregate -- all without allocating intermediate storage.

```rust
fn average_of_positives(values: &[f64]) -> Option<f64> {
    let (sum, count) = values.iter()
        .filter(|&&v| v > 0.0)
        .fold((0.0, 0usize), |(sum, count), &v| (sum + v, count + 1));

    if count > 0 {
        Some(sum / count as f64)
    } else {
        None
    }
}

fn main() {
    let temperatures = vec![-5.0, 12.3, -1.2, 25.7, 0.0, 18.9, -3.3, 31.1];
    match average_of_positives(&temperatures) {
        Some(avg) => println!("avg of positives: {avg:.2}"),
        None => println!("no positive values"),
    }
    // Output: avg of positives: 22.00
}
```

### Building Lookup Structures from Iterators

Iterators can collect into any collection that implements `FromIterator`,
including `HashMap`. This replaces the dict-comprehension pattern from Python.

```rust
use std::collections::HashMap;

struct SensorReading {
    sensor_id: u32,
    value: f64,
}

fn build_latest_readings(data: &[SensorReading]) -> HashMap<u32, f64> {
    // The last occurrence for each sensor_id wins
    data.iter()
        .map(|r| (r.sensor_id, r.value))
        .collect()
}

fn main() {
    let readings = vec![
        SensorReading { sensor_id: 1, value: 22.5 },
        SensorReading { sensor_id: 2, value: 18.3 },
        SensorReading { sensor_id: 1, value: 23.1 }, // overwrites sensor 1
    ];
    let latest = build_latest_readings(&readings);
    println!("sensor 1: {}", latest[&1]); // 23.1
    println!("sensor 2: {}", latest[&2]); // 18.3
}
```

### Chaining, Flat-Mapping, and Windowing

Complex pipelines can flatten nested structures and look at sliding windows
of data, all lazily evaluated.

```rust
fn main() {
    // flat_map: flatten one-to-many relationships
    let sentences = vec!["hello world", "foo bar baz"];
    let words: Vec<&str> = sentences.iter()
        .flat_map(|s| s.split_whitespace())
        .collect();
    assert_eq!(words, vec!["hello", "world", "foo", "bar", "baz"]);

    // windows: sliding window over a slice (not an iterator adapter, but on slices)
    let data = [10, 20, 15, 30, 25];
    let moving_avg: Vec<f64> = data.windows(3)
        .map(|w| w.iter().sum::<i32>() as f64 / 3.0)
        .collect();
    println!("moving averages: {moving_avg:?}");
    // [15.0, 21.666..., 23.333...]

    // chain: concatenate two iterators
    let first_half = [1, 2, 3];
    let second_half = [4, 5, 6];
    let combined: Vec<i32> = first_half.iter()
        .chain(second_half.iter())
        .copied()
        .collect();
    assert_eq!(combined, vec![1, 2, 3, 4, 5, 6]);

    // take and skip
    let first_three: Vec<i32> = (0..100).take(3).collect();
    let skipped: Vec<i32> = (0..10).skip(7).collect();
    assert_eq!(first_three, vec![0, 1, 2]);
    assert_eq!(skipped, vec![7, 8, 9]);
}
```

## Gotchas

**1. Iterators are lazy -- nothing happens until you consume them.**
This is the single most common surprise for beginners. Calling `.map()` or
`.filter()` alone does nothing; the compiler will even warn you:
```rust
let v = vec![1, 2, 3];
v.iter().map(|x| x * 2);  // WARNING: unused `Map` -- this does nothing!

// You must consume the iterator:
let doubled: Vec<i32> = v.iter().map(|x| x * 2).collect();
// Or use for_each for side effects:
v.iter().for_each(|x| println!("{x}"));
```

**2. `.iter()` vs `.into_iter()` vs `.iter_mut()` confusion.**
Choosing the wrong variant leads to type errors or unintended moves:
```rust
let names = vec!["Alice".to_string(), "Bob".to_string()];

// .iter() borrows: yields &String
for name in names.iter() {
    println!("{name}");
}
println!("names still usable: {names:?}"); // OK

// .into_iter() consumes: yields String (owned)
for name in names.into_iter() {
    println!("{name}");
}
// println!("{names:?}"); // ERROR: names has been moved!
```

**3. Collecting requires a type hint.**
`.collect()` can produce many different collection types, so the compiler needs
to know which one you want. Provide a type annotation or use turbofish:
```rust
let v = vec![1, 2, 3];
// let result = v.iter().collect();  // ERROR: cannot infer type
let result: Vec<&i32> = v.iter().collect();        // type annotation
let result2 = v.iter().collect::<Vec<&i32>>();      // turbofish syntax
let result3 = v.iter().copied().collect::<Vec<_>>(); // _ lets compiler infer element type
```

## Quick Reference

| Adapter (lazy) | Description | Example |
|---|---|---|
| `.map(f)` | Transform each element | `.map(\|x\| x * 2)` |
| `.filter(p)` | Keep elements where predicate is true | `.filter(\|x\| **x > 0)` |
| `.flat_map(f)` | Map then flatten | `.flat_map(\|s\| s.chars())` |
| `.enumerate()` | Attach index to each element | yields `(usize, T)` |
| `.zip(other)` | Pair elements from two iterators | `.zip(b.iter())` |
| `.chain(other)` | Concatenate two iterators | `.chain(b.iter())` |
| `.take(n)` | First `n` elements only | `.take(10)` |
| `.skip(n)` | Skip first `n` elements | `.skip(5)` |
| `.peekable()` | Allow lookahead with `.peek()` | useful in parsers |
| `.cloned()` / `.copied()` | Clone/copy referenced elements | `&T` -> `T` |

| Consumer (eager) | Description | Return type |
|---|---|---|
| `.collect()` | Gather into a collection | `Vec<T>`, `HashMap`, etc. |
| `.for_each(f)` | Apply side-effect to each element | `()` |
| `.sum()` | Sum all elements | `T` |
| `.count()` | Count elements | `usize` |
| `.min()` / `.max()` | Find extremes | `Option<T>` |
| `.any(p)` / `.all(p)` | Short-circuit boolean tests | `bool` |
| `.find(p)` | First element matching predicate | `Option<T>` |
| `.fold(init, f)` | Reduce to a single value | `T` |
| `.position(p)` | Index of first match | `Option<usize>` |
