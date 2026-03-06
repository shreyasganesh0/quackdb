# Lesson 02: Data Types

## What You're Building
The core type system that underpins every other component of QuackDB. You will define
how SQL-level logical types (INT32, VARCHAR, DECIMAL) map to physical storage types,
implement type coercion rules for mixed-type expressions, and build a ScalarValue enum
that can serialize/deserialize individual values to bytes.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- LogicalType, PhysicalType, and ScalarValue are all enums you must match exhaustively
- [Traits and Derive](../concepts/traits_and_derive.md) -- implementing Display, PartialEq; deriving Debug, Clone, Hash
- [Error Handling](../concepts/error_handling.md) -- cast_to and from_bytes return Result<_, String>

## Key Patterns

### Exhaustive Match on Enum Variants
When an enum has many variants, Rust forces you to handle every one. Group related
arms with `|` to keep things manageable.

```rust
// Analogy: a color mixer (NOT the QuackDB solution)
enum Color { Red, Green, Blue, Yellow, Cyan, Magenta }

impl Color {
    fn is_primary(&self) -> bool {
        match self {
            Color::Red | Color::Green | Color::Blue => true,
            _ => false,
        }
    }
}
```

### Implementing Display
The `fmt::Display` trait lets you control how a type prints with `{}`. Each enum
variant maps to a user-facing string.

```rust
use std::fmt;

enum Shape { Circle, Square, Triangle }

impl fmt::Display for Shape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Shape::Circle => write!(f, "CIRCLE"),
            Shape::Square => write!(f, "SQUARE"),
            Shape::Triangle => write!(f, "TRIANGLE"),
        }
    }
}
```

### Type Coercion Hierarchies
When two different types meet in an expression, a coercion function picks the
"wider" type. The pattern is: if both are the same, return it; otherwise climb
a hierarchy (e.g., Int8 < Int16 < Int32 < Int64 < Float64).

```rust
// Analogy: unit coercion for lengths (NOT the QuackDB solution)
enum LengthUnit { Mm, Cm, M, Km }

fn coerce_unit(a: &LengthUnit, b: &LengthUnit) -> Option<LengthUnit> {
    use LengthUnit::*;
    let rank = |u: &LengthUnit| match u { Mm => 0, Cm => 1, M => 2, Km => 3 };
    if rank(a) >= rank(b) { Some(a.clone()) } else { Some(b.clone()) }
}
```

## Step-by-Step Implementation Order
1. Start with `physical_type()` -- a straightforward match mapping each LogicalType variant to a PhysicalType
2. Implement `byte_width()` -- return `Some(n)` for fixed-size types, `None` for Varchar and Blob
3. Implement `is_numeric()`, `is_integer()`, `is_float()` -- use match with `|` to group variants
4. Implement `Display for LogicalType` -- return uppercase names like "INT32", "VARCHAR"
5. Implement `ScalarValue::logical_type()` -- match each variant and return the corresponding LogicalType
6. Implement `to_bytes()` and `from_bytes()` -- use `.to_le_bytes()` / `from_le_bytes()` for numeric types; for Varchar, write the raw UTF-8 bytes
7. Implement `cast_to()` -- handle numeric widening (i32 as i64), int-to-float, and numeric-to-string conversions; return Err for unsupported casts
8. Implement `coerce()` and `can_cast()` -- build a ranking of numeric types; return the wider type

## Reading the Tests
- **`test_type_coercion`** checks that Int32+Int64 yields Int64, Int32+Float64 yields Float64, and Boolean+Varchar yields None. This reveals the expected coercion hierarchy and that incompatible types must return None.
- **`test_scalar_roundtrip`** encodes several ScalarValue variants to bytes, then decodes them and asserts equality. The test passes the value's own `logical_type()` to `from_bytes`, so your serialization format must be self-consistent per type. Pay attention to Float32 -- it uses `to_le_bytes()` which produces 4 bytes, not 8.
