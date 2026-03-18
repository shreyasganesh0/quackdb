# Type Conversions

> **Prerequisites:** [traits_and_derive](./traits_and_derive.md)

## Quick Reference
- `x as u32` -- primitive cast (fast but silently truncates -- dangerous for narrowing)
- `i64::from(42i32)` / `42i32.into()` -- infallible conversion via `From`/`Into` traits
- `u8::try_from(300u32)` -- fallible conversion, returns `Result` (safe for narrowing)
- `.checked_add()`, `.saturating_add()`, `.wrapping_add()` -- explicit overflow strategies
- Always implement `From`, not `Into` -- you get `Into` for free

## Common Compiler Errors

**`error[E0308]: mismatched types -- expected 'u32', found 'usize'`**
You passed one integer type where another was expected.
Fix: use `as` for primitive casts (`x as u32`) or `.try_into()` for checked conversion.

**`error[E0277]: the trait bound 'u8: From<u32>' is not satisfied`**
`u8::from(x)` only works for widening conversions. Narrowing from `u32` to `u8` can lose data.
Fix: use `u8::try_from(x)` which returns `Result`, or `x as u8` if you accept truncation.

**`error[E0604]: only primitives can be cast with 'as'`**
You used `as` on a non-primitive type (struct, enum, etc.).
Fix: implement `From`/`Into` for your custom types instead.

## When You'll Use This
- **Lesson 26 (Cost Optimizer):** converting between row counts (`u64`), selectivity (`f64`), and cost components
- **Lesson 2 (Types):** `cast_to` converting between ScalarValue types
- **Lesson 9 (Pages):** converting `PageType` enum to/from `u8` discriminant

Type conversions appear frequently when working with mixed numeric types in computations.

## What This Is

Rust is a strongly typed language with no implicit type coercion. Unlike JavaScript where `"5" + 3` silently produces `"53"`, or C where an `int` quietly widens to a `long`, Rust requires every type conversion to be explicit. This eliminates an entire category of subtle bugs -- but it means you need to understand the different conversion mechanisms and when to use each one.

There are three main approaches to type conversion in Rust. The simplest is the `as` keyword, which performs primitive casts similar to C-style casts -- it is fast but can silently truncate or wrap values. For richer, type-safe conversions between your own types, Rust provides the `From`/`Into` traits, which represent infallible conversions that always succeed. For conversions that might fail (like parsing a string to a number, or narrowing a `u64` to a `u16`), Rust provides `TryFrom`/`TryInto`, which return a `Result` so you must handle the error case.

If you come from C++, think of `as` as `static_cast`, `From`/`Into` as implicit conversion constructors (but explicit in Rust), and `TryFrom` as a checked narrowing cast. Python programmers can think of `From` as being like `int(x)` or `float(x)` -- a function that converts between types -- while `TryFrom` is like those same functions but wrapped in a try/except. The key insight is that Rust pushes you toward conversions that cannot fail, and makes you handle failures explicitly when they can occur.

## Syntax

```rust
fn main() {
    // --- `as` casts (primitive types only) ---

    // Widening: always safe, no data loss
    let small: u8 = 200;
    let big: u32 = small as u32;        // 200

    // Narrowing: silently truncates!
    let large: u32 = 300;
    let truncated: u8 = large as u8;    // 44 (300 % 256) -- data loss!

    // Float to int: truncates toward zero
    let pi: f64 = 3.99;
    let rounded: i32 = pi as i32;       // 3 (not 4!)

    // Signed to unsigned: reinterprets bits
    let negative: i8 = -1;
    let as_unsigned: u8 = negative as u8;  // 255

    // --- From / Into (infallible conversions) ---

    // From: explicit constructor-style
    let n: i64 = i64::from(42i32);      // i32 -> i64 always succeeds

    // Into: caller-side conversion (same trait, other direction)
    let m: i64 = 42i32.into();          // equivalent to above

    // String conversions
    let s: String = String::from("hello");
    let s2: String = "world".into();    // &str -> String

    // --- TryFrom / TryInto (fallible conversions) ---
    use std::convert::TryFrom;

    let big_number: i64 = 1_000_000;
    let attempt: Result<i16, _> = i16::try_from(big_number);
    match attempt {
        Ok(val) => println!("fits: {val}"),
        Err(e) => println!("overflow: {e}"),  // this branch runs
    }
}
```

## Common Patterns

### Safe Numeric Narrowing for Storage

When writing values to a binary format, you often need to narrow from a
computation type (like `usize` or `u64`) to a storage type (like `u16` or
`u32`). Using `TryFrom` makes overflow bugs impossible to miss.

```rust
use std::convert::TryFrom;

/// Write a length-prefixed record. The length must fit in u16.
fn encode_record(data: &[u8]) -> Result<Vec<u8>, String> {
    let len = u16::try_from(data.len())
        .map_err(|_| format!("record too large: {} bytes (max 65535)", data.len()))?;

    let mut buf = Vec::with_capacity(2 + data.len());
    buf.extend_from_slice(&len.to_le_bytes());
    buf.extend_from_slice(data);
    Ok(buf)
}

fn main() {
    let small_record = vec![0u8; 100];
    let encoded = encode_record(&small_record).unwrap();
    println!("encoded {} bytes", encoded.len()); // 102

    let huge_record = vec![0u8; 70_000];
    let result = encode_record(&huge_record);
    println!("{}", result.unwrap_err()); // "record too large: 70000 bytes (max 65535)"
}
```

### Implementing `From` for Domain Types

Implementing `From` for your own types gives you idiomatic, composable
conversions. You automatically get `Into` for free.

```rust
/// A column value that can hold different types.
enum ColumnValue {
    Integer(i64),
    Float(f64),
    Text(String),
    Boolean(bool),
}

// Implement From for each source type
impl From<i64> for ColumnValue {
    fn from(v: i64) -> Self {
        ColumnValue::Integer(v)
    }
}

impl From<f64> for ColumnValue {
    fn from(v: f64) -> Self {
        ColumnValue::Float(v)
    }
}

impl From<String> for ColumnValue {
    fn from(v: String) -> Self {
        ColumnValue::Text(v)
    }
}

impl From<bool> for ColumnValue {
    fn from(v: bool) -> Self {
        ColumnValue::Boolean(v)
    }
}

fn insert_value(val: impl Into<ColumnValue>) {
    let column_val = val.into();
    match column_val {
        ColumnValue::Integer(n) => println!("storing integer: {n}"),
        ColumnValue::Float(f) => println!("storing float: {f}"),
        ColumnValue::Text(s) => println!("storing text: {s}"),
        ColumnValue::Boolean(b) => println!("storing bool: {b}"),
    }
}

fn main() {
    insert_value(42i64);                    // From<i64>
    insert_value(3.14f64);                  // From<f64>
    insert_value("hello".to_string());      // From<String>
    insert_value(true);                     // From<bool>
}
```

### Handling Numeric Overflow Explicitly

Rust provides several strategies for dealing with integer overflow beyond
just `as` truncation. Choosing the right one depends on your domain.

```rust
fn main() {
    let a: u8 = 200;
    let b: u8 = 100;

    // Checked: returns None on overflow
    let checked = a.checked_add(b);
    assert_eq!(checked, None);  // 200 + 100 overflows u8

    // Saturating: clamps to max/min value
    let saturated = a.saturating_add(b);
    assert_eq!(saturated, 255);  // clamped to u8::MAX

    // Wrapping: two's-complement wrap-around (defined behavior)
    let wrapped = a.wrapping_add(b);
    assert_eq!(wrapped, 44);   // (200 + 100) % 256

    // Overflowing: returns (result, did_overflow)
    let (result, overflowed) = a.overflowing_add(b);
    assert_eq!(result, 44);
    assert!(overflowed);

    // In debug builds, standard + panics on overflow.
    // In release builds, standard + wraps silently.
    // Use the explicit methods above when overflow behavior matters.

    // Converting between float and int safely
    let f: f64 = 1e18;
    if f >= i64::MIN as f64 && f <= i64::MAX as f64 {
        let n = f as i64;
        println!("safe conversion: {n}");
    } else {
        println!("value out of i64 range");
    }
}
```

## Gotchas

**1. `as` casts silently truncate and can hide bugs.**
This is the most dangerous conversion mechanism in Rust. It will never produce
a compile error for numeric types, even when data loss is guaranteed:
```rust
let big: u64 = u64::MAX;  // 18446744073709551615
let small: u8 = big as u8; // 255 -- silently truncated!

// Prefer TryFrom for narrowing conversions:
let safe = u8::try_from(big); // Err(TryFromIntError)
```

**2. Float-to-integer casts have surprising edge cases.**
`as` truncates toward zero, and for values outside the integer's range, the
result is **saturating** as of Rust 1.45+ (it used to be undefined!):
```rust
let huge: f64 = 1e20;
let n: i32 = huge as i32;      // i32::MAX (2147483647) -- saturated
let nan: f64 = f64::NAN;
let m: i32 = nan as i32;       // 0 -- NaN becomes zero
let neg: f64 = -1e20;
let k: i32 = neg as i32;       // i32::MIN
```

**3. Implementing `From` gives you `Into` for free, but not vice versa.**
Always implement `From` rather than `Into`. The standard library provides a
blanket implementation: if `T: From<U>`, then `U: Into<T>` automatically.
Implementing `Into` directly does not give you `From`:
```rust
// GOOD: implement From
// impl From<Celsius> for Fahrenheit { ... }
// Now both work: Fahrenheit::from(c) and c.into()

// BAD: don't implement Into directly
// impl Into<Fahrenheit> for Celsius { ... }  // doesn't give you From
```

## Related Concepts

- [Error Handling](./error_handling.md) -- `TryFrom`/`TryInto` return `Result`, integrating with `?` for error propagation
- [Traits and Derive](./traits_and_derive.md) -- `From`, `Into`, `TryFrom` are all traits you implement
- [Bitwise Ops](./bitwise_ops.md) -- `as` casts are needed when mixing integer types in bit operations
- [Generics](./generics.md) -- `fn f<T: Into<String>>(s: T)` accepts anything convertible

## Quick Reference

| Mechanism | When to Use | Can Fail? | Example |
|---|---|---|---|
| `as` | Primitive numeric casts | No (truncates silently) | `x as u32` |
| `From::from` | Infallible conversions | No | `i64::from(42i32)` |
| `.into()` | Same as `From`, caller side | No | `let x: i64 = 42i32.into()` |
| `TryFrom::try_from` | Fallible conversions | Yes (`Result`) | `u8::try_from(300u32)` |
| `.try_into()` | Same as `TryFrom`, caller side | Yes (`Result`) | `300u32.try_into()` |

| Overflow Strategy | Method | Behavior |
|---|---|---|
| Checked | `.checked_add(n)` | Returns `Option<T>` |
| Saturating | `.saturating_add(n)` | Clamps to `MIN`/`MAX` |
| Wrapping | `.wrapping_add(n)` | Wraps around (modular) |
| Overflowing | `.overflowing_add(n)` | Returns `(T, bool)` |

| Common `From` Impls in std | |
|---|---|
| `i64::from(i32)` | Widening integer |
| `f64::from(f32)` | Widening float |
| `String::from(&str)` | String from str slice |
| `Vec<u8>::from(&[u8])` | Vec from byte slice |
| `PathBuf::from(&str)` | Path from string |
| `IpAddr::from([u8; 4])` | IP from byte array |
