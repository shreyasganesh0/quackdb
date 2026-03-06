# Traits and Derive

> **Prerequisites:** [structs_and_impl](./structs_and_impl.md)

## What This Is

Traits in Rust are the primary mechanism for defining shared behavior across types. If you come from Java or TypeScript, think of traits as **interfaces**: they declare a set of methods that a type must implement. If you come from C++, they are similar to abstract base classes with pure virtual functions, but without inheritance hierarchies. Python programmers can think of them as **protocols** (from `typing.Protocol`), but enforced at compile time rather than duck-typed at runtime.

Unlike interfaces in many languages, Rust traits can provide **default implementations** for methods, so types that implement the trait can choose to override or accept the default. Traits are also the foundation of Rust's generics system -- you use them to express constraints on type parameters (covered in [trait_bounds](./trait_bounds.md)).

A huge productivity feature in Rust is the `derive` macro. Many common traits like `Debug`, `Clone`, `PartialEq`, and `Default` have mechanical implementations that the compiler can generate automatically. Instead of writing boilerplate, you annotate your struct or enum with `#[derive(Debug, Clone, PartialEq)]` and the compiler writes the implementation for you. This is roughly analogous to Python's `@dataclass` decorator or Lombok's `@Data` annotation in Java.

## Syntax

### Defining a trait

```rust
trait Summary {
    // Required method -- implementors must provide this
    fn summarize(&self) -> String;

    // Default method -- implementors can override or use as-is
    fn preview(&self) -> String {
        format!("{}...", &self.summarize()[..20])
    }
}
```

### Implementing a trait for a type

```rust
struct Article {
    title: String,
    author: String,
    content: String,
}

impl Summary for Article {
    fn summarize(&self) -> String {
        format!("{} by {}", self.title, self.author)
    }
    // preview() uses the default implementation
}
```

### Using derive for standard traits

```rust
#[derive(Debug, Clone, PartialEq, Default)]
struct Coordinate {
    x: f64,
    y: f64,
}
```

### Implementing Display manually

```rust
use std::fmt;

struct Temperature {
    celsius: f64,
}

impl fmt::Display for Temperature {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}
```

## Common Patterns

### Pattern 1: Deriving Debug and Display for logging

In any database-like system, you want types that are easy to print for debugging and readable for users.

```rust
use std::fmt;

#[derive(Debug, Clone)]
struct Column {
    name: String,
    data_type: String,
}

impl fmt::Display for Column {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.data_type)
    }
}

fn main() {
    let col = Column { name: "age".into(), data_type: "INT".into() };
    println!("Debug:   {:?}", col);   // Column { name: "age", data_type: "INT" }
    println!("Display: {}", col);      // age (INT)
}
```

### Pattern 2: Trait as a shared contract across types

When multiple types need to support the same operation, define a trait and implement it for each.

```rust
trait Encode {
    fn encode(&self) -> Vec<u8>;
}

struct IntValue(i64);
struct TextValue(String);

impl Encode for IntValue {
    fn encode(&self) -> Vec<u8> {
        self.0.to_le_bytes().to_vec()
    }
}

impl Encode for TextValue {
    fn encode(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

fn write_all(values: &[&dyn Encode]) {
    for v in values {
        let bytes = v.encode();
        println!("Wrote {} bytes", bytes.len());
    }
}
```

### Pattern 3: Default for builder-style initialization

`Default` gives you a zero-value constructor, useful when a struct has many optional fields.

```rust
#[derive(Debug, Default)]
struct ScanOptions {
    batch_size: usize,
    parallel: bool,
    filter: Option<String>,
}

fn main() {
    let opts = ScanOptions {
        batch_size: 1024,
        ..ScanOptions::default()  // fill remaining fields with defaults
    };
    println!("{:?}", opts);
}
```

## Gotchas

1. **`Display` cannot be derived.** Unlike `Debug`, there is no `#[derive(Display)]` in the standard library. You must always implement `fmt::Display` by hand. This trips up beginners who try `#[derive(Debug, Display)]` and get a compiler error.

2. **Orphan rule: you can only implement a trait if you own the trait or the type.** You cannot implement someone else's trait for someone else's type. For example, you cannot `impl Display for Vec<i32>` because you own neither `Display` nor `Vec`. The workaround is the newtype pattern: wrap the foreign type in your own struct.

3. **`Copy` requires `Clone`, and not all types can be `Copy`.** `Copy` means the type is duplicated by simple bitwise copy (like integers and booleans). Types that own heap memory (`String`, `Vec<T>`) cannot implement `Copy`. If you try `#[derive(Copy, Clone)]` on a struct containing a `String`, the compiler will refuse.

## Quick Reference

| Trait        | What it does                          | Derivable? | Notes                                  |
|-------------|---------------------------------------|-----------|----------------------------------------|
| `Debug`     | Format with `{:?}`                    | Yes       | Almost always derive this              |
| `Display`   | Format with `{}`                      | No        | Must implement manually                |
| `Clone`     | Explicit deep copy via `.clone()`     | Yes       | All fields must also be `Clone`        |
| `Copy`      | Implicit bitwise copy on assignment   | Yes       | Requires `Clone`; no heap-owning types |
| `Default`   | Zero/empty value via `Type::default()`| Yes       | All fields must also be `Default`      |
| `PartialEq` | Equality comparison with `==`         | Yes       | All fields must also be `PartialEq`    |

- **Define a trait:** `trait Name { fn method(&self) -> ReturnType; }`
- **Implement a trait:** `impl Name for MyType { fn method(&self) -> ReturnType { ... } }`
- **Derive common traits:** `#[derive(Debug, Clone, PartialEq, Default)]`
- **Implement Display:** `impl fmt::Display for MyType { fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, ...) } }`
