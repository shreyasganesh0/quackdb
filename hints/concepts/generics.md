# Generics

> **Prerequisites:** [structs_and_impl](./structs_and_impl.md)

## What This Is

Generics allow you to write code that works with many different types without duplicating logic. If you come from Python, generics are similar to the type parameters you see in `list[int]` or `dict[str, Any]` -- but in Rust they are enforced at compile time and have zero runtime cost. If you come from C++, Rust generics are conceptually similar to templates but with an important difference: Rust generics are type-checked at the point of definition, not at each instantiation site. If you come from JavaScript or other dynamically typed languages, generics replace the pattern of writing a function that "just works" on any input -- but with full type safety.

Under the hood, the Rust compiler uses a process called monomorphization: when you write `fn add<T>(a: T, b: T)` and call it with `i32` and later with `f64`, the compiler generates two specialized versions of the function -- one for `i32` and one for `f64`. This means generic code runs exactly as fast as if you had written separate functions by hand. There is no boxing, no vtable lookup, no runtime type inspection. The tradeoff is that it can increase binary size if you use the same generic function with many different types.

To constrain what operations a generic type supports, Rust uses trait bounds. Writing `fn add<T: std::ops::Add<Output = T>>(a: T, b: T) -> T` says "T can be any type, as long as it implements the `Add` trait." This is stricter than C++ templates (which just try to compile and fail with cryptic errors if an operation is missing) and stricter than Python's duck typing (which fails at runtime). Rust tells you at the definition site if your generic code requires something it has not declared.

## Syntax

```rust
// A generic function: T is a type parameter
fn first<T>(items: &[T]) -> Option<&T> {
    items.first()
}

// A generic function with a trait bound: T must implement Display
fn print_labeled<T: std::fmt::Display>(label: &str, value: T) {
    println!("{}: {}", label, value);
}

// Equivalent using `where` clause (preferred when bounds get complex)
fn print_labeled_v2<T>(label: &str, value: T)
where
    T: std::fmt::Display,
{
    println!("{}: {}", label, value);
}

// A generic struct
struct Pair<A, B> {
    left: A,
    right: B,
}

// Implementing methods on a generic struct
impl<A, B> Pair<A, B> {
    fn new(left: A, right: B) -> Self {
        Self { left, right }
    }

    fn swap(self) -> Pair<B, A> {
        Pair {
            left: self.right,
            right: self.left,
        }
    }
}

// Implementing methods only for specific type combinations
impl Pair<String, i32> {
    fn format_record(&self) -> String {
        format!("{} = {}", self.left, self.right)
    }
}

fn main() {
    let numbers = vec![10, 20, 30];
    let words = vec!["hello", "world"];

    // The compiler infers T = i32 and T = &str
    println!("{:?}", first(&numbers));
    println!("{:?}", first(&words));

    // Turbofish syntax: explicitly specify the type parameter
    let parsed = "42".parse::<i32>().unwrap();
    let parsed_float = "3.14".parse::<f64>().unwrap();
    println!("{} {}", parsed, parsed_float);
}
```

### The turbofish `::<T>`

When the compiler cannot infer the type, you use turbofish to specify it explicitly:

```rust
let numbers: Vec<i32> = Vec::new();       // type annotation on the variable
let numbers = Vec::<i32>::new();           // turbofish on the function call
let numbers: Vec<_> = "1,2,3".split(',')  // _ lets the compiler infer part of the type
    .map(|s| s.parse::<i32>().unwrap())
    .collect();
```

## Common Patterns

### Pattern 1: A generic container with type-safe operations

```rust
/// A ring buffer that works with any element type.
struct RingBuffer<T> {
    data: Vec<T>,
    capacity: usize,
    write_pos: usize,
}

impl<T> RingBuffer<T> {
    fn new(capacity: usize) -> Self {
        Self {
            data: Vec::with_capacity(capacity),
            capacity,
            write_pos: 0,
        }
    }

    fn push(&mut self, item: T) {
        if self.data.len() < self.capacity {
            self.data.push(item);
        } else {
            self.data[self.write_pos] = item;
        }
        self.write_pos = (self.write_pos + 1) % self.capacity;
    }

    fn len(&self) -> usize {
        self.data.len()
    }
}

// Add Debug printing only when T itself is Debug
impl<T: std::fmt::Debug> RingBuffer<T> {
    fn dump(&self) {
        println!("RingBuffer({:?})", self.data);
    }
}

fn main() {
    let mut buf = RingBuffer::new(3);
    buf.push("alpha");
    buf.push("beta");
    buf.push("gamma");
    buf.push("delta");  // overwrites "alpha"
    buf.dump();          // RingBuffer(["delta", "beta", "gamma"])
}
```

### Pattern 2: Generic functions with multiple trait bounds

```rust
use std::fmt::Display;
use std::cmp::PartialOrd;

/// Finds the maximum value and prints it. Requires that T can be compared and displayed.
fn print_max<T>(items: &[T])
where
    T: PartialOrd + Display,
{
    if items.is_empty() {
        println!("(empty)");
        return;
    }
    let mut max = &items[0];
    for item in &items[1..] {
        if item > max {
            max = item;
        }
    }
    println!("Max: {}", max);
}

fn main() {
    print_max(&[3, 1, 4, 1, 5, 9]);       // Max: 9
    print_max(&[2.7, 1.4, 3.1]);           // Max: 3.1
    print_max(&["banana", "apple", "cherry"]); // Max: cherry
}
```

### Pattern 3: Implementing a trait generically

```rust
use std::fmt;

struct Matrix<T> {
    rows: usize,
    cols: usize,
    data: Vec<T>,
}

impl<T> Matrix<T> {
    fn new(rows: usize, cols: usize, fill: T) -> Self
    where
        T: Clone,
    {
        Self {
            rows,
            cols,
            data: vec![fill; rows * cols],
        }
    }

    fn get(&self, row: usize, col: usize) -> &T {
        &self.data[row * self.cols + col]
    }
}

// Implement Display for any Matrix<T> where T is displayable
impl<T: fmt::Display> fmt::Display for Matrix<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in 0..self.rows {
            for col in 0..self.cols {
                if col > 0 { write!(f, " ")?; }
                write!(f, "{}", self.data[row * self.cols + col])?;
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

fn main() {
    let grid = Matrix::new(2, 3, 0.0_f64);
    println!("{}", grid);
}
```

## Gotchas

1. **You cannot use operations on a generic type unless you add a trait bound.** Unlike C++ templates where you just use `a + b` and hope for the best, Rust requires you to declare `T: Add<Output = T>` if you want to use `+`. The compiler error message will tell you exactly which trait you need to add.

2. **Turbofish is needed more often than you expect.** Calling `.collect()` on an iterator almost always needs a type annotation, because the compiler does not know which collection type you want. Common fix: `let v: Vec<_> = iter.collect();` or `iter.collect::<Vec<_>>()`.

3. **Monomorphization means each concrete type generates its own machine code.** If you use `HashMap<String, Vec<i32>>` and `HashMap<String, Vec<f64>>`, the compiler generates two complete copies of `HashMap`'s code. This is rarely a problem, but in very generic codebases with many type combinations, it can noticeably increase compile times and binary size. If that becomes an issue, consider using trait objects (`dyn Trait`) for dynamic dispatch instead.

## Quick Reference

| Concept                  | Syntax                                    | Notes                                         |
|--------------------------|-------------------------------------------|-----------------------------------------------|
| Generic function         | `fn foo<T>(x: T)`                        | T is inferred at call site                    |
| Trait bound              | `fn foo<T: Clone>(x: T)`                 | T must implement Clone                        |
| Multiple bounds          | `T: Clone + Debug`                       | T must implement both traits                  |
| Where clause             | `fn foo<T>(x: T) where T: Clone`         | Equivalent, preferred for complex bounds      |
| Generic struct           | `struct Foo<T> { field: T }`             | T is determined when Foo is constructed       |
| Impl on generic struct   | `impl<T> Foo<T> { ... }`                | Methods available for all T                   |
| Impl on specific type    | `impl Foo<String> { ... }`              | Methods only for `Foo<String>`                |
| Turbofish                | `"42".parse::<i32>()`                    | Explicit type specification                   |
| Inferred type            | `let v: Vec<_> = ...`                   | Compiler fills in the `_`                     |
| Default type parameter   | `struct Foo<T = i32> { ... }`            | Uses i32 if T is not specified                |
| Const generic            | `struct Buf<const N: usize>`             | Parameterize by a compile-time constant       |
