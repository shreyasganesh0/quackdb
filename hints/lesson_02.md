# Lesson 02: Data Types

## What You're Building
The core type system that underpins every other component of QuackDB. You will define
how SQL-level logical types (INT32, VARCHAR, DECIMAL) map to physical storage types,
implement type coercion rules for mixed-type expressions, and build a ScalarValue enum
that can serialize/deserialize individual values to bytes.

## Concept Recap
Building on Lesson 01: You used raw byte buffers in the arena allocator. Now you will define the *meaning* of those bytes -- the type system that tells the database whether a 4-byte chunk is an INT32, a FLOAT32, or part of a VARCHAR.

## Rust Concepts You'll Need
- [Enums and Matching](../concepts/enums_and_matching.md) -- LogicalType, PhysicalType, and ScalarValue are all enums you must match exhaustively
- [Traits and Derive](../concepts/traits_and_derive.md) -- implementing Display, PartialEq; deriving Debug, Clone, Hash
- [Error Handling](../concepts/error_handling.md) -- cast_to and from_bytes return Result<_, String>

## Key Patterns

### Exhaustive Match on Enum Variants
When an enum has many variants, Rust forces you to handle every one. Group related
arms with `|` to keep things manageable. Think of a customs officer checking passports --
every country must have a known entry procedure, even if some share the same rules.

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
variant maps to a user-facing string. Think of it like a name badge -- each type
gets a human-readable label.

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
a hierarchy (e.g., Int8 < Int16 < Int32 < Int64 < Float64). It works like
unit conversion -- you always convert to the unit that can hold both values
without losing precision.

```rust
// Analogy: unit coercion for lengths (NOT the QuackDB solution)
enum LengthUnit { Mm, Cm, M, Km }

fn coerce_unit(a: &LengthUnit, b: &LengthUnit) -> Option<LengthUnit> {
    use LengthUnit::*;
    let rank = |u: &LengthUnit| match u { Mm => 0, Cm => 1, M => 2, Km => 3 };
    if rank(a) >= rank(b) { Some(a.clone()) } else { Some(b.clone()) }
}
```

### Little-Endian Byte Serialization
Numeric values serialize to bytes using `.to_le_bytes()` and deserialize with
`from_le_bytes()`. Little-endian is like writing a number with the least significant
digit first -- consistently ordered so both sides agree.

```rust
// Analogy: encoding a temperature reading to bytes
fn encode_temp(temp: f32) -> [u8; 4] {
    temp.to_le_bytes()
}
fn decode_temp(bytes: &[u8; 4]) -> f32 {
    f32::from_le_bytes(*bytes)
}
```

## Step-by-Step Implementation Order
1. Start with `physical_type()` -- a straightforward match mapping each LogicalType variant to a PhysicalType
2. Implement `byte_width()` -- return `Some(n)` for fixed-size types, `None` for Varchar and Blob
3. Implement `is_numeric()`, `is_integer()`, `is_float()` -- use match with `|` to group variants; remember that Decimal is numeric but Boolean is not
4. Implement `Display for LogicalType` -- return uppercase names like "INT32", "VARCHAR"; handle Decimal with its precision and scale
5. Implement `ScalarValue::logical_type()` -- match each variant and return the corresponding LogicalType; Null carries its type
6. Implement `ScalarValue::is_null()` and `Display for ScalarValue` -- Null matches the Null variant; display prints the value or "NULL"
7. Implement `to_bytes()` and `from_bytes()` -- use `.to_le_bytes()` / `from_le_bytes()` for numeric types; for Varchar, write the raw UTF-8 bytes
8. Implement `cast_to()` -- handle numeric widening (i32 as i64), int-to-float, and numeric-to-string conversions; return Err for unsupported casts
9. Implement `coerce()` and `can_cast()` -- build a ranking of numeric types; return the wider type; return None for incompatible pairs like Boolean+Varchar

## Common Mistakes
- **Forgetting that Null carries a type**: `ScalarValue::Null(LogicalType::Int32)` must report its `logical_type()` as `Int32`, not some generic null type. The type system depends on nulls knowing their column type.
- **Treating Boolean as numeric**: Several tests explicitly check that `Boolean.is_numeric()` returns `false`. Boolean is stored as a single byte, but it is not a number.
- **Using big-endian bytes**: The tests use `to_le_bytes()` (little-endian). If you use big-endian serialization, the roundtrip tests will fail because the byte patterns will not match.

## Reading the Tests
- **`test_logical_to_physical_mapping`** checks every LogicalType variant maps to its expected PhysicalType (Boolean to Bool, Int8 to Int8, Varchar to Varchar, etc.). This is a completeness check -- make sure you handle every variant.
- **`test_byte_widths`** asserts specific byte widths for every fixed-size type (Boolean=1, Int16=2, Int32=4, Int64=8, Date=4, Timestamp=8) and `None` for Varchar and Blob. This reveals the exact sizes your implementation must return.
- **`test_type_categories`** checks `is_numeric()`, `is_integer()`, and `is_float()` for various types. Notably, it asserts Boolean is NOT numeric and floats are NOT integers. Use this to get your category groupings right.
- **`test_scalar_value_types`** verifies that `ScalarValue::Boolean(true).logical_type()` returns `LogicalType::Boolean`, and similarly for Int32, Int64, Float64, Varchar, and `Null(Int32)`. The Null case is important: it must return the wrapped type.
- **`test_scalar_value_is_null`** checks that `Null(Int32)` is null, but `Int32(0)` and `Boolean(false)` are NOT null. Zero and false are valid values, not missing data.
- **`test_scalar_roundtrip`** encodes several ScalarValue variants to bytes, then decodes them and asserts equality. The test passes the value's own `logical_type()` to `from_bytes`, so your serialization format must be self-consistent per type. Pay attention to Float32 -- it uses `to_le_bytes()` which produces 4 bytes, not 8.
- **`test_scalar_varchar_roundtrip`** serializes "hello world" as a Varchar and decodes it back. This confirms that your Varchar serialization uses raw UTF-8 bytes.
- **`test_type_coercion`** checks that Int32+Int64 yields Int64, Int32+Float64 yields Float64, Float32+Float64 yields Float64, same types return themselves, and Boolean+Varchar yields None. This reveals the expected coercion hierarchy and that incompatible types must return None.
- **`test_can_cast`** verifies that Int32 can cast to Int64, Int32 to Float64, and any numeric type to Varchar. This tells you casting to string must be supported for all numerics.
- **`test_scalar_cast`** casts Int32(42) to Int64 and Float64, checking that the numeric value is preserved. Widening casts must not lose data.
- **`test_decimal_type`** creates `Decimal { precision: 10, scale: 2 }` and checks it is numeric with a fixed byte width. Decimal is a special case with metadata fields.
- **`test_logical_type_display`** checks that Int32 displays as "INT32", Varchar as "VARCHAR", Boolean as "BOOLEAN", and Decimal includes relevant info. Match these exact strings.
- **`test_scalar_display`** checks that `Int32(42)` displays as "42", `Boolean(true)` as "true", `Varchar("hello")` as "hello", and `Null(Int32)` contains "NULL" or "null".

## What Comes Next
With the type system in place, Lesson 03 builds **columnar vectors** -- the `Vector`
type that stores a column of values in a contiguous byte buffer. Your `LogicalType`
determines the byte width for fixed-size columns, and `ScalarValue` is the unit used
by `get_value()` and `set_value()` on vectors.
