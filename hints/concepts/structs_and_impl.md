# Structs and Impl Blocks

> **Prerequisites:** None - this is a starting concept

## What This Is

Structs are Rust's primary way of creating custom data types that group related values together. If you come from Python, think of them as a typed version of a dataclass. If you come from JavaScript, think of them as objects with a fixed shape enforced at compile time. If you come from C++, they are essentially the same as C++ structs (and classes), but without inheritance.

Unlike classes in Python, JavaScript, or C++, Rust structs do not bundle data and behavior in the same declaration. Instead, you define the data (fields) in a `struct` block, and then attach methods and associated functions in a separate `impl` block. This separation is deliberate: it means you can add behavior to a type across multiple `impl` blocks, even in different files, and it keeps the data layout visually distinct from the logic.

Rust has no constructors, no `__init__`, no `constructor()`. By convention, you write an associated function called `new()` that returns an instance of the struct. This is just a convention, not a language feature -- the compiler does not treat `new` specially.

## Syntax

```rust
// Define a struct with named fields
struct Sensor {
    id: u32,
    name: String,
    reading: f64,
    is_active: bool,
}

// Attach methods and associated functions via an impl block
impl Sensor {
    // Associated function (no `self` parameter) -- like a static method / classmethod.
    // Called as Sensor::new(...), not sensor.new(...)
    fn new(id: u32, name: String) -> Self {
        // `Self` is an alias for the type being impl'd (here, `Sensor`)
        Self {
            id,
            name,           // field init shorthand: `name: name` can be written as just `name`
            reading: 0.0,
            is_active: true,
        }
    }

    // Immutable method -- borrows self, cannot modify fields
    fn label(&self) -> String {
        format!("Sensor[{}]: {}", self.id, self.name)
    }

    // Mutable method -- borrows self mutably, can modify fields
    fn record(&mut self, value: f64) {
        self.reading = value;
    }

    // Method that takes ownership (consumes the struct)
    fn decommission(self) -> String {
        // After this call, the original variable can no longer be used
        format!("{} has been decommissioned", self.name)
    }
}
```

### The three flavors of `self`

| Signature       | Python equivalent     | What it means                          |
|-----------------|-----------------------|----------------------------------------|
| `&self`         | `def method(self)`    | Borrow the struct immutably (read only)|
| `&mut self`     | `def method(self)`    | Borrow the struct mutably (read/write) |
| `self`          | (no direct parallel)  | Take ownership; struct is consumed     |

## Common Patterns

### Pattern 1: Builder-style construction

When a struct has many optional fields, use a builder pattern with method chaining.

```rust
struct QueryPlan {
    table: String,
    filter: Option<String>,
    limit: Option<usize>,
    parallel: bool,
}

impl QueryPlan {
    fn new(table: &str) -> Self {
        Self {
            table: table.to_string(),
            filter: None,
            limit: None,
            parallel: false,
        }
    }

    fn with_filter(mut self, expr: &str) -> Self {
        self.filter = Some(expr.to_string());
        self
    }

    fn with_limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }

    fn enable_parallel(mut self) -> Self {
        self.parallel = true;
        self
    }
}

// Usage:
// let plan = QueryPlan::new("orders")
//     .with_filter("amount > 100")
//     .with_limit(1000)
//     .enable_parallel();
```

### Pattern 2: Structs that hold references to other data

```rust
struct CacheEntry<'a> {
    key: &'a str,       // borrows a string owned elsewhere
    hits: u64,
}

impl<'a> CacheEntry<'a> {
    fn new(key: &'a str) -> Self {
        Self { key, hits: 0 }
    }

    fn touch(&mut self) {
        self.hits += 1;
    }
}
```

### Pattern 3: Tuple structs and unit structs

```rust
// Tuple struct -- fields accessed by index, useful for newtypes
struct Meters(f64);
struct Seconds(f64);

impl Meters {
    fn as_kilometers(&self) -> f64 {
        self.0 / 1000.0
    }
}

// Unit struct -- no fields, used as a marker or token
struct ReadOnly;
```

## Gotchas

1. **No default values in struct definitions.** Unlike Python dataclasses or C++ default member initializers, you cannot write `reading: f64 = 0.0` in a struct definition. You must supply every field when constructing, or derive/implement the `Default` trait and use `..Default::default()`.

2. **Forgetting `mut` on the variable, not just the method.** Even if a method takes `&mut self`, the variable holding the struct must be declared `let mut sensor = Sensor::new(...)`. If you write `let sensor = ...`, calling `sensor.record(42.0)` will fail to compile.

3. **Moving out of a struct field.** If you try to take a `String` field out of a struct (e.g., `let name = sensor.name;`), the struct becomes partially moved and can no longer be used as a whole. Use `.clone()` if you need a copy, or take a reference `&sensor.name`.

## Quick Reference

| Concept              | Syntax                              | Notes                                    |
|----------------------|--------------------------------------|------------------------------------------|
| Define a struct      | `struct Foo { x: i32, y: String }`  | Named fields                             |
| Tuple struct         | `struct Foo(i32, String);`          | Fields accessed by `.0`, `.1`            |
| Unit struct          | `struct Foo;`                       | No data, marker type                     |
| Impl block           | `impl Foo { ... }`                  | Can have multiple impl blocks            |
| Associated function  | `fn new() -> Self`                  | No self param; called as `Foo::new()`    |
| Immutable method     | `fn bar(&self) -> T`               | Borrows; struct still usable after call  |
| Mutable method       | `fn bar(&mut self)`                 | Requires `let mut` on the variable       |
| Consuming method     | `fn bar(self) -> T`                 | Struct is moved; cannot be used after    |
| Field init shorthand | `Self { x, y }`                    | When variable names match field names    |
| Struct update syntax | `Foo { x: 10, ..other }`           | Copy remaining fields from another value |
