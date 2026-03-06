# Error Handling

> **Prerequisites:** [enums_and_matching](./enums_and_matching.md)

## What This Is

Rust does not have exceptions. There is no `try/catch`, no `throw`, no unwinding stack traces that silently propagate errors upward. Instead, Rust uses two enum types from the standard library to represent the possibility of failure or absence: `Result<T, E>` and `Option<T>`. If you come from Python, JavaScript, or Java, this is a fundamental shift -- errors are **values** that you must explicitly handle, not invisible control flow that jumps across stack frames.

`Result<T, E>` is an enum with two variants: `Ok(T)` for success and `Err(E)` for failure. `Option<T>` has `Some(T)` for a present value and `None` for absence (like `null` or `None` in other languages, but you cannot accidentally forget to check for it). The compiler forces you to handle both variants before you can access the inner value. This makes "forgot to check the error" bugs nearly impossible.

The `?` operator is Rust's ergonomic shortcut for error propagation. It replaces what would be pages of `match` statements with a single character. When you write `let value = some_function()?;`, the `?` operator checks if the result is `Ok` (extracts the value) or `Err` (immediately returns the error from the current function). This gives you the conciseness of exceptions with the explicitness of checked return values. C++ programmers can think of it as a disciplined, compiler-enforced version of error codes that cannot be ignored.

## Syntax

### Result and Option basics

```rust
// Result: operation that can fail
fn parse_port(s: &str) -> Result<u16, String> {
    s.parse::<u16>().map_err(|e| format!("Invalid port: {}", e))
}

// Option: value that might be absent
fn find_user(id: u64) -> Option<String> {
    if id == 1 { Some("Alice".to_string()) } else { None }
}
```

### The `?` operator

```rust
fn connect(address: &str) -> Result<(), String> {
    let port_str = address.split(':').nth(1).ok_or("Missing port")?;
    let port: u16 = port_str.parse().map_err(|e| format!("{}", e))?;
    println!("Connecting to port {}", port);
    Ok(())
}
```

### Unwrap and expect

```rust
// .unwrap() -- panics on Err/None. Only use when failure is truly impossible.
let num: i32 = "42".parse().unwrap();

// .expect("msg") -- same as unwrap but with a custom panic message
let num: i32 = "42".parse().expect("hardcoded value should always parse");
```

## Common Patterns

### Pattern 1: Chaining transformations with map and and_then

`Option` and `Result` both support functional-style method chains, letting you transform values without nested `match` blocks.

```rust
fn read_config_value(raw: &str) -> Option<u32> {
    raw.strip_prefix("size=")       // Option<&str>
        .map(|s| s.trim())          // Option<&str>
        .and_then(|s| s.parse().ok()) // Option<u32> -- parse can fail
}

fn main() {
    assert_eq!(read_config_value("size= 128"), Some(128));
    assert_eq!(read_config_value("name=foo"), None);
    assert_eq!(read_config_value("size=abc"), None);
}
```

### Pattern 2: Converting between Option and Result with ok_or

When a function returns `Result` but you have an `Option`, use `.ok_or()` or `.ok_or_else()` to supply the error value.

```rust
fn get_header(headers: &[(&str, &str)], key: &str) -> Result<String, String> {
    headers
        .iter()
        .find(|(k, _)| *k == key)       // Option<&(&str, &str)>
        .map(|(_, v)| v.to_string())     // Option<String>
        .ok_or(format!("Missing header: {}", key))  // Result<String, String>
}

fn process_request(headers: &[(&str, &str)]) -> Result<(), String> {
    let content_type = get_header(headers, "Content-Type")?;
    let length = get_header(headers, "Content-Length")?;
    println!("Type: {}, Length: {}", content_type, length);
    Ok(())
}
```

### Pattern 3: Custom error types

For libraries and larger projects, define your own error enum so callers get structured, matchable errors instead of opaque strings.

```rust
use std::fmt;

#[derive(Debug)]
enum StorageError {
    NotFound { key: String },
    CorruptedData(String),
    IoError(std::io::Error),
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::NotFound { key } => write!(f, "Key not found: {}", key),
            StorageError::CorruptedData(msg) => write!(f, "Corrupted data: {}", msg),
            StorageError::IoError(e) => write!(f, "I/O error: {}", e),
        }
    }
}

// Allow automatic conversion from std::io::Error using ?
impl From<std::io::Error> for StorageError {
    fn from(e: std::io::Error) -> Self {
        StorageError::IoError(e)
    }
}

fn read_record(path: &str) -> Result<Vec<u8>, StorageError> {
    let data = std::fs::read(path)?;  // io::Error auto-converts via From
    if data.is_empty() {
        return Err(StorageError::CorruptedData("Empty file".into()));
    }
    Ok(data)
}
```

## Gotchas

1. **Using `.unwrap()` in production code.** Calling `.unwrap()` is tempting during prototyping, but it **panics** (crashes the program) on `Err` or `None`. In a real application, use `?` to propagate errors, or handle them explicitly with `match`. Reserve `.unwrap()` for cases where failure is logically impossible and you can prove it with a comment, or for tests.

2. **The `?` operator only works in functions that return `Result` or `Option`.** If you try to use `?` inside `main()` or a function that returns `()`, you will get a compile error. Fix this by changing the function signature:

    ```rust
    // This won't compile:
    // fn main() { let x = "42".parse::<i32>()?; }

    // This works:
    fn main() -> Result<(), Box<dyn std::error::Error>> {
        let x = "42".parse::<i32>()?;
        println!("{}", x);
        Ok(())
    }
    ```

3. **Confusing `Option` methods with `Result` methods.** Both types share many method names (`.map()`, `.and_then()`, `.unwrap()`), but some are unique. For example, `Result` has `.map_err()` (transform the error), while `Option` has `.unwrap_or()` and `.unwrap_or_else()` (provide a default). Using the wrong one produces confusing type errors. Check which type you are working with.

## Quick Reference

| Method / Operator | Type | What it does |
|---|---|---|
| `?` | Result / Option | Return early on Err/None, extract Ok/Some |
| `.unwrap()` | Result / Option | Extract value or **panic** |
| `.expect("msg")` | Result / Option | Extract value or panic with message |
| `.map(f)` | Result / Option | Transform the success/some value |
| `.map_err(f)` | Result only | Transform the error value |
| `.and_then(f)` | Result / Option | Chain operations that themselves return Result/Option |
| `.ok_or(err)` | Option -> Result | Convert None to Err(err) |
| `.ok_or_else(f)` | Option -> Result | Convert None to Err(f()) -- lazy |
| `.unwrap_or(val)` | Result / Option | Extract value or use a default |
| `.unwrap_or_default()` | Result / Option | Extract value or use `Default::default()` |
| `.is_ok()` / `.is_some()` | Result / Option | Boolean check without consuming |
| `.ok()` | Result -> Option | Discard the error, convert to Option |

- **Rule of thumb:** Use `?` for propagation, `.map()`/`.and_then()` for transformation, `match` for branching.
- **In tests:** `.unwrap()` is fine -- a panic means a test failure, which is what you want.
- **For application entry points:** `fn main() -> Result<(), Box<dyn std::error::Error>>` lets you use `?` everywhere.
