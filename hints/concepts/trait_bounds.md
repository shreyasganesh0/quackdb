# Trait Bounds

> **Prerequisites:** [generics](./generics.md), [traits_and_derive](./traits_and_derive.md)

## What This Is

Trait bounds are how Rust constrains generic type parameters. When you write a generic function like `fn process<T>(item: T)`, the function knows nothing about `T` -- it cannot print it, clone it, compare it, or call any methods on it. Trait bounds tell the compiler "this type parameter `T` must implement these specific traits," which unlocks the corresponding methods inside the function body. If you come from TypeScript, think of `T extends Interface`. In Java, this is `<T extends Comparable<T>>`. In C++, this is loosely similar to concepts (C++20) or the informal constraints that templates relied on before concepts existed.

Without trait bounds, Rust generics would be nearly useless -- you could accept any type but do nothing with it. With trait bounds, you get the best of both worlds: the function works for any type that satisfies the constraints, and the compiler verifies at each call site that the concrete type actually implements the required traits. This is checked at compile time, not runtime, so there is zero overhead.

Rust provides two syntactic forms for expressing bounds: the inline form `<T: Clone + Debug>` and the `where` clause. They are equivalent in meaning, but the `where` clause is preferred when bounds get complex or involve multiple parameters. You will see both forms throughout Rust codebases, and understanding both is essential for reading library documentation.

## Syntax

### Inline trait bounds

```rust
use std::fmt::Debug;

fn print_pair<T: Debug>(a: &T, b: &T) {
    println!("({:?}, {:?})", a, b);
}
```

### Multiple bounds with `+`

```rust
fn deduplicate<T: Clone + PartialEq>(items: &[T]) -> Vec<T> {
    let mut result: Vec<T> = Vec::new();
    for item in items {
        if !result.contains(item) {   // requires PartialEq
            result.push(item.clone()); // requires Clone
        }
    }
    result
}
```

### `where` clause (equivalent, but cleaner for complex bounds)

```rust
fn merge_sorted<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Ord + Clone,
{
    let mut result = Vec::with_capacity(left.len() + right.len());
    let (mut i, mut j) = (0, 0);
    while i < left.len() && j < right.len() {
        if left[i] <= right[j] {
            result.push(left[i].clone());
            i += 1;
        } else {
            result.push(right[j].clone());
            j += 1;
        }
    }
    result.extend_from_slice(&left[i..]);
    result.extend_from_slice(&right[j..]);
    result
}
```

### Bounds on struct implementations

```rust
struct Cache<K, V> {
    entries: Vec<(K, V)>,
}

impl<K, V> Cache<K, V>
where
    K: PartialEq + Clone,
    V: Clone,
{
    fn get(&self, key: &K) -> Option<&V> {
        self.entries.iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v)
    }

    fn insert(&mut self, key: K, value: V) {
        self.entries.push((key, value));
    }
}
```

## Common Patterns

### Pattern 1: Display + Debug bounds for logging and user output

When building systems that need to report values to users and to logs, require both formatting traits.

```rust
use std::fmt::{Debug, Display};

struct Metric<T: Display + Debug> {
    name: String,
    value: T,
}

impl<T: Display + Debug> Metric<T> {
    fn new(name: &str, value: T) -> Self {
        Metric { name: name.to_string(), value }
    }

    fn report(&self) {
        println!("[METRIC] {} = {}", self.name, self.value);   // Display
    }

    fn debug_dump(&self) {
        println!("[DEBUG] {} = {:?}", self.name, self.value);  // Debug
    }
}

fn main() {
    let latency = Metric::new("query_latency_ms", 42.7_f64);
    latency.report();      // [METRIC] query_latency_ms = 42.7
    latency.debug_dump();  // [DEBUG] query_latency_ms = 42.7
}
```

### Pattern 2: Default bound for initialization

The `Default` trait bound lets generic code create "empty" or "zero" values without knowing the concrete type.

```rust
fn create_grid<T: Default + Clone>(rows: usize, cols: usize) -> Vec<Vec<T>> {
    vec![vec![T::default(); cols]; rows]
}

fn main() {
    let int_grid: Vec<Vec<i32>> = create_grid(3, 4);
    // [[0, 0, 0, 0], [0, 0, 0, 0], [0, 0, 0, 0]]

    let str_grid: Vec<Vec<String>> = create_grid(2, 2);
    // [["", ""], ["", ""]]

    let bool_grid: Vec<Vec<bool>> = create_grid(2, 3);
    // [[false, false, false], [false, false, false]]
}
```

### Pattern 3: Combining bounds for a generic registry

Real-world data structures often need several bounds together. The `where` clause keeps this readable.

```rust
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;

struct Registry<K, V> {
    items: HashMap<K, V>,
}

impl<K, V> Registry<K, V>
where
    K: Eq + Hash + Display + Clone,
    V: Display,
{
    fn new() -> Self {
        Registry { items: HashMap::new() }
    }

    fn register(&mut self, key: K, value: V) {
        println!("Registering: {} => {}", &key, &value);
        self.items.insert(key, value);
    }

    fn lookup(&self, key: &K) -> Option<&V> {
        self.items.get(key)
    }

    fn list_all(&self) {
        for (k, v) in &self.items {
            println!("  {} => {}", k, v);
        }
    }
}

fn main() {
    let mut reg = Registry::new();
    reg.register("temperature".to_string(), 98.6_f64);
    reg.register("humidity".to_string(), 45.0);
    reg.list_all();
}
```

## Gotchas

1. **Forgetting a bound and getting a confusing error.** If you try to use `==` on a generic `T` without `T: PartialEq`, the error message says something like "binary operation `==` cannot be applied to type `T`." The fix is always to add the required trait bound. Read the error carefully -- Rust usually suggests which trait is missing.

    ```rust
    // Error: "no implementation for `T == T`"
    // fn find<T>(items: &[T], target: &T) -> bool { ... }

    // Fix: add PartialEq
    fn find<T: PartialEq>(items: &[T], target: &T) -> bool {
        items.iter().any(|item| item == target)
    }
    ```

2. **Bounds on the struct definition vs. on the impl block.** You can put bounds in either place, but the convention is to put them on the `impl` block, not on the struct definition. Putting bounds on the struct itself forces every use of the struct to satisfy those bounds, even in contexts where they are not needed (like storing items without comparing them).

    ```rust
    // Prefer this: bounds on impl, not on struct
    struct Bag<T> {
        items: Vec<T>,
    }

    impl<T: PartialEq> Bag<T> {
        fn contains(&self, item: &T) -> bool {
            self.items.contains(item)
        }
    }

    // Avoid this: bounds on the struct
    // struct Bag<T: PartialEq> { items: Vec<T> }
    // Now Bag<SomeNonEqType> won't compile even if you never call contains()
    ```

3. **`Clone` vs `Copy` as bounds.** If you use `T: Copy`, you restrict your function to small, stack-only types (integers, booleans, references). If you use `T: Clone`, you support heap-owning types like `String` and `Vec<T>` too. Prefer `Clone` unless you specifically need the implicit-copy semantics of `Copy`.

## Quick Reference

| Syntax | Meaning |
|---|---|
| `fn f<T: Clone>(x: T)` | `T` must implement `Clone` |
| `fn f<T: Clone + Debug>(x: T)` | `T` must implement both `Clone` and `Debug` |
| `fn f<T>(x: T) where T: Clone + Debug` | Same as above, using `where` clause |
| `impl<T: PartialEq> MyStruct<T>` | Methods available only when `T: PartialEq` |
| `fn f<T: Default>() -> T` | Can call `T::default()` inside the body |
| `fn f<T: Into<String>>(s: T)` | Accepts anything convertible to String |

**Common standard library trait bounds and what they unlock:**

| Trait | Unlocks | Example types |
|---|---|---|
| `Debug` | `{:?}` formatting | Almost everything with `#[derive(Debug)]` |
| `Display` | `{}` formatting | Primitives, custom `impl Display` |
| `Clone` | `.clone()` explicit copy | String, Vec, most types |
| `Copy` | Implicit copy on assignment | i32, f64, bool, &T |
| `PartialEq` | `==` and `!=` | All primitives, derive-able |
| `Eq` | Strict equality (no NaN) | Integers, strings (not f64) |
| `Ord` | `<`, `>`, sorting | Integers, strings (not f64) |
| `Hash` | Use as HashMap key | Integers, strings (not f64) |
| `Default` | `T::default()` zero value | 0 for numbers, "" for String, false for bool |
| `Send` | Safe to transfer between threads | Most types (not Rc) |
| `Sync` | Safe to share references between threads | Most types (not RefCell) |

- **Rule of thumb:** start with no bounds, add them as the compiler tells you what is needed.
- **Use `where` clauses** when you have more than two bounds or more than one type parameter.
- **Bounds propagate:** if your struct holds a `T` and you need to clone the struct, then `T` must be `Clone` too.
