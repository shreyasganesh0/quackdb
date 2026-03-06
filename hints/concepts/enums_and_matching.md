# Enums and Matching

> **Prerequisites:** None - this is a starting concept

## What This Is

Rust enums are far more powerful than enums in most other languages. In C++ or Java, an enum is essentially a named integer constant. In Rust, each variant of an enum can **carry data** of different types and shapes. This makes Rust enums equivalent to **tagged unions** (C/C++), **discriminated unions** in TypeScript, or **algebraic data types** in Haskell. Python and JavaScript have no direct equivalent -- the closest would be a class hierarchy where each subclass holds different data, but Rust enums are a single type checked at compile time.

The companion feature is `match`, Rust's pattern matching expression. Unlike a `switch` statement in C++ or JavaScript, `match` is **exhaustive**: the compiler forces you to handle every possible variant. This eliminates an entire class of bugs where you forget to handle a case. If you add a new variant to an enum, every `match` on that enum will produce a compile error until you handle the new case.

Rust also provides lighter-weight matching tools: `if let` for when you only care about one variant, and the `matches!()` macro for boolean checks. Enums can also have methods via `impl` blocks, just like structs.

## Syntax

### Defining an enum with data variants

```rust
enum DataType {
    Integer,                        // no data
    Float(f64),                     // one value
    Text(String),                   // one value
    Timestamp { secs: i64, nanos: u32 },  // named fields (struct variant)
}
```

### Matching on an enum

```rust
fn type_size(dt: &DataType) -> usize {
    match dt {
        DataType::Integer => 8,
        DataType::Float(_) => 8,
        DataType::Text(s) => s.len(),
        DataType::Timestamp { secs: _, nanos: _ } => 12,
    }
}
```

### `if let` for single-variant checks

```rust
if let DataType::Text(s) = &column_type {
    println!("Text column with content: {}", s);
}
```

### `matches!()` macro

```rust
let is_numeric = matches!(column_type, DataType::Integer | DataType::Float(_));
```

## Common Patterns

### Pattern 1: Representing operation types

Enums naturally model a fixed set of operations, each carrying its own parameters.

```rust
enum Instruction {
    Load { register: u8, address: u64 },
    Store { register: u8, address: u64 },
    Add(u8, u8, u8),   // dest, src1, src2
    Halt,
}

fn execute(instr: &Instruction) {
    match instr {
        Instruction::Load { register, address } => {
            println!("Loading r{} from 0x{:x}", register, address);
        }
        Instruction::Store { register, address } => {
            println!("Storing r{} to 0x{:x}", register, address);
        }
        Instruction::Add(dest, src1, src2) => {
            println!("r{} = r{} + r{}", dest, src1, src2);
        }
        Instruction::Halt => {
            println!("Halting execution");
        }
    }
}
```

### Pattern 2: Enum methods and self-inspection

Enums can have methods, just like structs. This is useful for type-checking or conversion logic.

```rust
#[derive(Debug, Clone, PartialEq)]
enum Token {
    Number(f64),
    Identifier(String),
    Operator(char),
    EndOfInput,
}

impl Token {
    fn is_operator(&self) -> bool {
        matches!(self, Token::Operator(_))
    }

    fn as_number(&self) -> Option<f64> {
        match self {
            Token::Number(n) => Some(*n),
            _ => None,
        }
    }

    fn precedence(&self) -> u8 {
        match self {
            Token::Operator('+') | Token::Operator('-') => 1,
            Token::Operator('*') | Token::Operator('/') => 2,
            _ => 0,
        }
    }
}
```

### Pattern 3: Nested matching and guards

You can destructure deeply and add conditions with `if` guards.

```rust
enum Shape {
    Circle(f64),
    Rectangle(f64, f64),
}

enum DrawCommand {
    Fill(Shape, String),   // shape, color
    Stroke(Shape, f64),    // shape, line_width
}

fn describe(cmd: &DrawCommand) {
    match cmd {
        DrawCommand::Fill(Shape::Circle(r), color) if *r > 100.0 => {
            println!("Large filled circle (r={}) in {}", r, color);
        }
        DrawCommand::Fill(shape, color) => {
            println!("Fill {:?}-ish shape in {}", std::mem::discriminant(shape), color);
        }
        DrawCommand::Stroke(_, width) if *width < 1.0 => {
            println!("Hairline stroke");
        }
        DrawCommand::Stroke(_, width) => {
            println!("Stroke with width {}", width);
        }
    }
}
```

## Gotchas

1. **`match` must be exhaustive.** You must handle every variant, or use a wildcard `_` as the last arm. This is a feature, not a bug -- it catches forgotten cases at compile time. If you add a variant to an enum and forget to update a `match`, the compiler tells you immediately.

    ```rust
    match token {
        Token::Number(n) => { /* ... */ }
        Token::Operator(c) => { /* ... */ }
        _ => { /* catch-all for Identifier, EndOfInput, and any future variants */ }
    }
    ```

2. **Moving data out of an enum variant.** When you match on an owned enum (not a reference), the inner data is **moved** out. If you want to inspect without consuming, match on a reference (`&my_enum` or `ref` bindings).

    ```rust
    let tok = Token::Identifier("x".into());

    // This moves the String out of tok -- tok is no longer usable
    if let Token::Identifier(name) = tok {
        println!("{}", name);
    }

    // To avoid moving, match on a reference
    let tok2 = Token::Identifier("y".into());
    if let Token::Identifier(name) = &tok2 {
        println!("{}", name);   // name is &String here
        // tok2 is still usable
    }
    ```

3. **No implicit fallthrough.** Unlike C/C++ `switch` statements, Rust `match` arms do not fall through to the next arm. Each arm is independent. To match multiple patterns in one arm, use the `|` (or) operator: `Token::Number(_) | Token::Identifier(_) => { ... }`.

## Quick Reference

| Feature | Syntax | Use when... |
|---------|--------|-------------|
| Basic enum | `enum E { A, B, C }` | Fixed set of options, no data |
| Data variants | `enum E { A(i32), B(String) }` | Each variant carries different data |
| Struct variants | `enum E { A { x: i32, y: i32 } }` | Variant has named fields |
| `match` | `match val { E::A(x) => ..., E::B(s) => ... }` | Handle all cases |
| `if let` | `if let E::A(x) = val { ... }` | Only care about one variant |
| `matches!()` | `matches!(val, E::A(_) \| E::B(_))` | Boolean check against pattern(s) |
| Match guard | `E::A(x) if x > 10 => ...` | Extra condition on a match arm |
| Wildcard | `_ => ...` | Catch-all for remaining variants |
| Enum method | `impl E { fn foo(&self) { ... } }` | Add behavior to the enum |

- Enums are **stack-allocated** and their size equals the largest variant plus a discriminant tag.
- Use `#[derive(Debug)]` on enums just like on structs.
- The standard library's most important enums are `Option<T>` and `Result<T, E>` -- see [error_handling](./error_handling.md).
