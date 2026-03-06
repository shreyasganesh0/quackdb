# String Types

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md)

## What This Is

Rust has two main string types, and this is one of the first things that confuses newcomers from Python, JavaScript, or even C++. In those languages, there is essentially one string type (`str` in Python, `string` in JS, `std::string` in C++). Rust has `String` (an owned, heap-allocated, growable buffer) and `&str` (a borrowed reference to a slice of UTF-8 bytes). Understanding the difference is essential because the compiler will refuse to compile code that mixes them incorrectly.

The relationship between `String` and `&str` mirrors the relationship between `Vec<T>` and `&[T]`. `String` **owns** its data -- it is responsible for allocating and freeing the memory. `&str` is a **view** into string data owned by someone else (a `String`, a string literal compiled into the binary, or a subsection of another string). Think of `String` as Python's `str` and `&str` as a read-only window into that string. In practice, function parameters should usually accept `&str` (the broadest type), while struct fields that need to own their data should use `String`.

All Rust strings are guaranteed to be valid **UTF-8**. This means you cannot index a string by byte position and get a character -- a single character might span 1 to 4 bytes. This is different from Python 3 (which abstracts over encoding) and C++ (which gives you raw bytes). Rust forces you to be explicit about whether you want bytes, characters, or grapheme clusters, which prevents subtle Unicode bugs.

## Syntax

### Creating strings

```rust
// String literal is &str (baked into the binary)
let greeting: &str = "hello";

// Owned String from a literal
let owned: String = String::from("hello");
let also_owned: String = "hello".to_string();

// From format macro
let name = "world";
let msg: String = format!("hello, {}!", name);
```

### Converting between String and &str

```rust
let owned = String::from("database");

// String -> &str (cheap: just a reference)
let borrowed: &str = &owned;
let also_borrowed: &str = owned.as_str();

// &str -> String (allocates new heap memory)
let new_owned: String = borrowed.to_string();
let also_new: String = String::from(borrowed);
```

### Iterating

```rust
let text = "cafe\u{0301}";  // "cafe" + combining accent = "cafe\u{0301}"

// Iterate over Unicode scalar values (chars)
for ch in text.chars() {
    print!("{} ", ch);  // c a f e  ́
}

// Iterate over raw bytes
for byte in text.as_bytes() {
    print!("{:02x} ", byte);  // 63 61 66 65 cc 81
}
```

## Common Patterns

### Pattern 1: Functions that accept &str for maximum flexibility

By accepting `&str`, a function works with both `String` and string literals without copying.

```rust
fn is_valid_column_name(name: &str) -> bool {
    !name.is_empty()
        && name.chars().next().map_or(false, |c| c.is_alphabetic())
        && name.chars().all(|c| c.is_alphanumeric() || c == '_')
}

fn main() {
    // Works with &str literals
    assert!(is_valid_column_name("user_id"));

    // Works with String (auto-derefs to &str)
    let col = String::from("total_amount");
    assert!(is_valid_column_name(&col));
}
```

### Pattern 2: Building strings with format! and push_str

When constructing strings dynamically, `format!` is the most readable approach. For loops, use `String::new()` with `.push_str()`.

```rust
fn build_select(table: &str, columns: &[&str]) -> String {
    let cols = columns.join(", ");
    format!("SELECT {} FROM {}", cols, table)
}

fn build_csv_row(values: &[&str]) -> String {
    let mut row = String::new();
    for (i, val) in values.iter().enumerate() {
        if i > 0 {
            row.push(',');       // push a single char
        }
        row.push_str(val);      // push a &str
    }
    row.push('\n');
    row
}

fn main() {
    println!("{}", build_select("users", &["id", "name", "email"]));
    // SELECT id, name, email FROM users

    println!("{}", build_csv_row(&["Alice", "30", "NYC"]));
    // Alice,30,NYC
}
```

### Pattern 3: Splitting, trimming, and searching

Rust's `&str` has a rich set of methods for text processing, all returning iterators or slices.

```rust
fn parse_key_value(line: &str) -> Option<(&str, &str)> {
    let trimmed = line.trim();
    if trimmed.starts_with('#') || trimmed.is_empty() {
        return None; // skip comments and blank lines
    }
    let mut parts = trimmed.splitn(2, '=');
    let key = parts.next()?.trim();
    let value = parts.next()?.trim();
    Some((key, value))
}

fn main() {
    let config = "host = localhost\nport = 5432\n# comment\ndb = analytics";
    for line in config.lines() {
        if let Some((k, v)) = parse_key_value(line) {
            println!("{:>10} => {}", k, v);
        }
    }
    //       host => localhost
    //       port => 5432
    //         db => analytics
}
```

## Gotchas

1. **You cannot index strings with `s[0]`.** Rust strings are UTF-8, and a single character may be multiple bytes. `s[0]` would be ambiguous: do you want byte 0 or character 0? Use `.chars().nth(0)` for the first character, or `.as_bytes()[0]` for the first byte. You can slice with byte ranges (`&s[0..4]`), but this will **panic at runtime** if the range does not fall on a character boundary.

    ```rust
    let emoji = "hello!";
    // let ch = emoji[0];           // COMPILE ERROR
    let ch = emoji.chars().nth(0);  // Some('h')
    let slice = &emoji[0..5];       // "hello" -- OK, all ASCII
    ```

2. **String concatenation with `+` consumes the left-hand side.** The `+` operator takes ownership of the left `String`. This surprises beginners who expect it to work like Python or JavaScript.

    ```rust
    let first = String::from("hello");
    let second = String::from(" world");
    let combined = first + &second;  // first is MOVED, second is borrowed
    // println!("{}", first);        // COMPILE ERROR: first was moved
    println!("{}", combined);        // "hello world"
    ```

    Prefer `format!("{}{}", a, b)` which borrows both sides and is easier to read.

3. **`.len()` returns byte count, not character count.** For ASCII this is the same, but for non-ASCII text it differs. Use `.chars().count()` for the number of Unicode scalar values.

    ```rust
    let text = "hello";
    assert_eq!(text.len(), 5);           // 5 bytes = 5 chars (ASCII)
    assert_eq!(text.chars().count(), 5);

    let kanji = "日本語";
    assert_eq!(kanji.len(), 9);           // 9 bytes (3 chars * 3 bytes each)
    assert_eq!(kanji.chars().count(), 3); // 3 characters
    ```

## Quick Reference

| Operation | Code | Returns |
|---|---|---|
| Create owned string | `String::from("hi")` or `"hi".to_string()` | `String` |
| Borrow a String | `&my_string` or `my_string.as_str()` | `&str` |
| Format | `format!("x={}, y={}", x, y)` | `String` |
| Concatenate | `format!("{}{}", a, b)` or `a.push_str(b)` | `String` / `()` |
| Length in bytes | `s.len()` | `usize` |
| Length in chars | `s.chars().count()` | `usize` |
| Split | `s.split(',')` / `s.splitn(2, '=')` | Iterator |
| Trim whitespace | `s.trim()` / `s.trim_start()` / `s.trim_end()` | `&str` |
| Starts/ends with | `s.starts_with("x")` / `s.ends_with("y")` | `bool` |
| Find substring | `s.contains("foo")` / `s.find("foo")` | `bool` / `Option<usize>` |
| Replace | `s.replace("old", "new")` | `String` |
| To uppercase/lower | `s.to_uppercase()` / `s.to_lowercase()` | `String` |
| Iterate chars | `s.chars()` | `Iterator<Item=char>` |
| Iterate bytes | `s.as_bytes()` or `s.bytes()` | `&[u8]` / `Iterator<Item=u8>` |

- **Function parameters:** prefer `&str` (accepts both `&String` and `"literals"`)
- **Struct fields:** use `String` (owns the data, lives as long as the struct)
- **Return values:** usually `String` (caller gets ownership)
