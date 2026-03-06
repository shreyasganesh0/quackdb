# Lifetimes

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md)

## What This Is

Lifetimes are Rust's way of tracking how long references remain valid. Every reference in Rust has a lifetime -- the region of code during which the reference is guaranteed to point to valid data. Most of the time, the compiler figures out lifetimes automatically (this is called "lifetime elision"). But in certain situations -- especially when a function returns a reference, or when a struct holds a reference -- you need to annotate lifetimes explicitly so the compiler can verify your code is safe.

If you come from C or C++, you have dealt with dangling pointers: a pointer that outlives the data it points to. Rust lifetimes exist to prevent exactly this problem, at compile time rather than at runtime. In garbage-collected languages like Python, JavaScript, or Go, this problem does not exist because the runtime keeps objects alive as long as any reference exists. Rust achieves the same safety without a garbage collector by using lifetime annotations as compile-time proof that references will not dangle.

A lifetime annotation like `'a` does not change how long data lives. It is a constraint that tells the compiler "these references must all be valid for the same region of code." Think of lifetimes as labels that let the compiler connect the dots between where data is created, where it is borrowed, and where the borrow is used. When the dots do not connect, the compiler gives you an error instead of letting you create a dangling reference.

## Syntax

```rust
// Explicit lifetime annotation on a function:
// "The returned reference lives at least as long as both input references"
fn longer<'a>(s1: &'a str, s2: &'a str) -> &'a str {
    if s1.len() >= s2.len() { s1 } else { s2 }
}

fn main() {
    let result;
    let string1 = String::from("long string");
    {
        let string2 = String::from("hi");
        result = longer(&string1, &string2);
        println!("Longer: {}", result);  // OK: both strings are still alive
    }
    // println!("{}", result);  // COMPILE ERROR: `string2` does not live long enough
    // The lifetime 'a must cover the scope where `result` is used,
    // but `string2` is dropped at the closing brace above.
}
```

### Reading lifetime annotations

```
fn foo<'a>(x: &'a str) -> &'a str
       ^^      ^^           ^^
       |       |            |
       |       |            Return value lives at least as long as 'a
       |       Parameter x has lifetime 'a
       Declare a lifetime parameter called 'a
```

## Common Patterns

### Pattern 1: Struct that borrows data (zero-copy parsing)

When you want a struct to hold a reference to data owned elsewhere -- for example, to avoid copying a large buffer -- the struct needs a lifetime parameter.

```rust
/// A parsed log entry that borrows from the original log line.
/// No data is copied -- this is a zero-copy "view" into the source string.
struct LogEntry<'src> {
    timestamp: &'src str,
    level: &'src str,
    message: &'src str,
}

impl<'src> LogEntry<'src> {
    fn parse(line: &'src str) -> Option<Self> {
        // Expected format: "2025-01-15T10:30:00 INFO User logged in"
        let mut parts = line.splitn(3, ' ');
        Some(LogEntry {
            timestamp: parts.next()?,
            level: parts.next()?,
            message: parts.next()?,
        })
    }

    fn is_error(&self) -> bool {
        self.level == "ERROR"
    }
}

fn main() {
    let raw_line = String::from("2025-01-15T10:30:00 ERROR Disk full");
    if let Some(entry) = LogEntry::parse(&raw_line) {
        println!("[{}] {}", entry.level, entry.message);
    }
    // `entry` borrows from `raw_line`, so `raw_line` must outlive `entry`
}
```

### Pattern 2: Multiple lifetime parameters

Sometimes inputs have different lifetimes, and the return value only depends on one of them.

```rust
/// Returns a reference into `data`, using `separator` only for the search.
/// The return value's lifetime is tied to `data`, not to `separator`.
fn first_segment<'data, 'sep>(data: &'data str, separator: &'sep str) -> &'data str {
    match data.find(separator) {
        Some(idx) => &data[..idx],
        None => data,
    }
}

fn main() {
    let csv_line = String::from("Alice,30,Engineer");
    let name;
    {
        let sep = String::from(",");
        name = first_segment(&csv_line, &sep);
        // `sep` is about to be dropped, but that's fine:
        // `name` borrows from `csv_line`, not from `sep`
    }
    println!("Name: {}", name);  // OK!
}
```

### Pattern 3: The `'static` lifetime

`'static` means the reference is valid for the entire duration of the program. String literals have the type `&'static str` because they are embedded in the binary.

```rust
fn app_name() -> &'static str {
    "DataProcessor v2.1"   // string literal -- lives forever
}

// A common use: trait objects that own their data must be 'static
fn spawn_worker(task: Box<dyn Fn() + Send + 'static>) {
    std::thread::spawn(move || task());
}

fn main() {
    println!("{}", app_name());

    let name = String::from("worker-1");
    spawn_worker(Box::new(move || {
        // `name` is moved into the closure, so the closure is 'static
        println!("Running {}", name);
    }));
}
```

## Gotchas

1. **Lifetime annotations do not control how long values live.** A common misconception is that writing `'a` somehow extends the life of a value. It does not. Lifetimes are descriptive, not prescriptive. They tell the compiler about relationships between references so it can check correctness. If the relationships are impossible, you get a compile error -- you do not get magic memory extension.

2. **Returning a reference to a local variable is always wrong.** No amount of lifetime annotation can fix this:
   ```rust
   fn broken() -> &str {       // What lifetime would this even be?
       let s = String::from("local");
       &s                       // `s` is dropped here -- would dangle
   }
   // Fix: return an owned String instead
   fn fixed() -> String {
       String::from("local")
   }
   ```

3. **Elision rules can mask what is happening.** The compiler automatically inserts lifetimes in common cases, which means beginners often do not realize lifetimes are involved. The three elision rules are:
   - Each input reference gets its own lifetime parameter.
   - If there is exactly one input lifetime, it is assigned to all output references.
   - If one of the inputs is `&self` or `&mut self`, its lifetime is assigned to all output references.

   When these rules are insufficient, the compiler asks you to add explicit annotations. Understanding elision helps you see why.

## Quick Reference

| Concept                    | Syntax                                       | Meaning                                               |
|----------------------------|----------------------------------------------|-------------------------------------------------------|
| Lifetime parameter         | `<'a>`                                       | Declares a lifetime variable                          |
| Reference with lifetime    | `&'a T`                                      | Reference valid for at least `'a`                     |
| Mutable ref with lifetime  | `&'a mut T`                                  | Mutable reference valid for at least `'a`             |
| Struct with lifetime       | `struct Foo<'a> { field: &'a str }`          | Struct borrows data; cannot outlive the source         |
| Multiple lifetimes         | `fn f<'a, 'b>(x: &'a str, y: &'b str)`      | Inputs may have independent lifetimes                 |
| `'static`                  | `&'static str`                               | Reference valid for entire program                    |
| Lifetime bound             | `T: 'a`                                      | Type T must be valid for at least lifetime `'a`       |

**Elision rules (when you can omit annotations):**

1. Each input reference gets a distinct lifetime: `fn f(x: &str, y: &str)` becomes `fn f<'a, 'b>(x: &'a str, y: &'b str)`
2. Single input lifetime applies to all outputs: `fn f(x: &str) -> &str` becomes `fn f<'a>(x: &'a str) -> &'a str`
3. Method with `&self`: self's lifetime applies to all outputs: `fn f(&self, x: &str) -> &str` uses self's lifetime for the return
