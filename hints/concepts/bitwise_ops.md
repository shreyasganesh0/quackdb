# Bitwise Operations

> **Prerequisites:** None - this is a starting concept

## What This Is

Bitwise operations manipulate individual bits within integer values. They are fundamental to systems programming and appear throughout database engines: bitmasks track which slots in a page are occupied, bitpacking squeezes multiple small values into a single integer for compact storage, and bit flags encode sets of boolean options in a single field.

If you have used bitwise operations in C, C++, or Python, Rust's syntax will feel very familiar. The operators are identical: `&` (AND), `|` (OR), `^` (XOR), `!` (NOT), `<<` (left shift), `>>` (right shift). The key difference from C/C++ is that Rust does not allow implicit integer conversions, so you must be explicit about types. The difference from Python is that Rust integers have fixed widths (e.g., `u8`, `u32`, `u64`) rather than arbitrary precision, which means overflow is a concern you must handle.

Rust also provides useful methods on integer types for common bit operations: `count_ones()`, `leading_zeros()`, `trailing_zeros()`, `rotate_left()`, and `rotate_right()`. These often compile down to single CPU instructions and are safer than hand-rolling the equivalent logic.

## Syntax

```rust
fn main() {
    let a: u8 = 0b1100_1010;
    let b: u8 = 0b1010_0110;

    // Bitwise AND -- bits set in BOTH operands
    let and = a & b;       // 0b1000_0010

    // Bitwise OR -- bits set in EITHER operand
    let or = a | b;        // 0b1110_1110

    // Bitwise XOR -- bits set in ONE but not both
    let xor = a ^ b;       // 0b0110_1100

    // Bitwise NOT -- flip all bits
    let not_a = !a;        // 0b0011_0101

    // Left shift -- multiply by 2^n, fills with zeros on the right
    let shifted_left = a << 2;   // 0b0010_1000 (with wrapping on u8)

    // Right shift -- divide by 2^n for unsigned types
    let shifted_right = a >> 3;  // 0b0001_1001

    // Rust requires both operands to have the same type
    let x: u32 = 1;
    let y: u32 = x << 10;   // OK
    // let z: u32 = x << 10u8; // ERROR: shift amount must match type (on some ops)

    println!("and={and:#010b}, or={or:#010b}, xor={xor:#010b}");
}
```

## Common Patterns

### Bitmask for Slot Occupancy

A common database pattern is tracking which slots in a fixed-size page are free
or occupied using a bitmap. Each bit represents one slot.

```rust
struct SlotBitmap {
    bits: u64, // supports up to 64 slots
}

impl SlotBitmap {
    fn new() -> Self {
        SlotBitmap { bits: 0 }
    }

    /// Mark slot `i` as occupied.
    fn set(&mut self, i: usize) {
        assert!(i < 64);
        self.bits |= 1u64 << i;
    }

    /// Mark slot `i` as free.
    fn clear(&mut self, i: usize) {
        assert!(i < 64);
        self.bits &= !(1u64 << i);
    }

    /// Check if slot `i` is occupied.
    fn is_set(&self, i: usize) -> bool {
        assert!(i < 64);
        (self.bits >> i) & 1 == 1
    }

    /// Find the first free slot, or None if all are occupied.
    fn first_free(&self) -> Option<usize> {
        let inverted = !self.bits;
        if inverted == 0 {
            return None; // all 64 slots taken
        }
        Some(inverted.trailing_zeros() as usize)
    }

    /// Count how many slots are occupied.
    fn count_occupied(&self) -> u32 {
        self.bits.count_ones()
    }
}

fn main() {
    let mut bm = SlotBitmap::new();
    bm.set(0);
    bm.set(3);
    bm.set(7);
    assert!(bm.is_set(3));
    assert!(!bm.is_set(4));
    assert_eq!(bm.first_free(), Some(1));
    assert_eq!(bm.count_occupied(), 3);
}
```

### Bitpacking Multiple Values into One Integer

When storage is at a premium, you can pack several small fields into a single
integer. For example, encoding a record's type tag (4 bits) and length (12 bits)
into one `u16`.

```rust
/// Pack a 4-bit tag and a 12-bit length into a single u16.
fn pack(tag: u8, length: u16) -> u16 {
    assert!(tag < 16, "tag must fit in 4 bits");
    assert!(length < 4096, "length must fit in 12 bits");
    ((tag as u16) << 12) | length
}

/// Unpack the tag and length from a packed u16.
fn unpack(packed: u16) -> (u8, u16) {
    let tag = (packed >> 12) as u8;        // top 4 bits
    let length = packed & 0x0FFF;          // bottom 12 bits (mask with 0b0000_1111_1111_1111)
    (tag, length)
}

fn main() {
    let packed = pack(5, 1023);
    let (tag, len) = unpack(packed);
    assert_eq!(tag, 5);
    assert_eq!(len, 1023);
    println!("packed = {packed:#018b}");    // 0b0101_0011_1111_1111
}
```

### Bit Flags with Constants

Using named constants for bit flags makes code readable while still being
efficient. This pattern is used for permission systems, feature flags, and
status indicators.

```rust
// Column flags for a table schema
const FLAG_NULLABLE: u8    = 1 << 0;  // 0b0000_0001
const FLAG_PRIMARY_KEY: u8 = 1 << 1;  // 0b0000_0010
const FLAG_INDEXED: u8     = 1 << 2;  // 0b0000_0100
const FLAG_UNIQUE: u8      = 1 << 3;  // 0b0000_1000

fn describe_flags(flags: u8) {
    if flags & FLAG_NULLABLE != 0 { println!("  nullable"); }
    if flags & FLAG_PRIMARY_KEY != 0 { println!("  primary key"); }
    if flags & FLAG_INDEXED != 0 { println!("  indexed"); }
    if flags & FLAG_UNIQUE != 0 { println!("  unique"); }
}

fn main() {
    // Combine flags with OR
    let col_flags = FLAG_PRIMARY_KEY | FLAG_INDEXED | FLAG_UNIQUE;
    describe_flags(col_flags);

    // Remove a flag with AND NOT
    let updated = col_flags & !FLAG_INDEXED;
    assert_eq!(updated & FLAG_INDEXED, 0); // indexed bit is cleared

    // Toggle a flag with XOR
    let toggled = col_flags ^ FLAG_NULLABLE;
    assert_ne!(toggled & FLAG_NULLABLE, 0); // nullable is now set
}
```

## Gotchas

**1. Shift overflow panics in debug mode.**
In debug builds, shifting by an amount greater than or equal to the bit width
of the type causes a panic. In release builds, the behavior wraps. Use
`wrapping_shl` / `wrapping_shr` if you need defined behavior regardless of
build mode:
```rust
let x: u8 = 1;
// x << 8;               // PANIC in debug! (u8 is only 8 bits wide)
let safe = x.wrapping_shl(8); // always returns 0
```

**2. `!` is bitwise NOT, not logical NOT.**
In C/C++, `~` is bitwise NOT and `!` is logical NOT. In Rust, `!` does both
jobs: it is bitwise NOT on integers and logical NOT on `bool`. This can confuse
C++ programmers:
```rust
let x: u8 = 0b0000_0001;
let flipped = !x;          // 0b1111_1110 -- bitwise NOT, not logical
let flag = true;
let negated = !flag;       // false -- logical NOT on bool
```

**3. No implicit widening between integer types.**
Unlike C/C++ where `int` operands are implicitly promoted, Rust requires
explicit casts. This prevents subtle bugs but requires more typing:
```rust
let small: u8 = 0xFF;
let big: u32 = 0x100;
// let result = small & big;          // ERROR: mismatched types
let result = (small as u32) & big;    // OK
```

## Quick Reference

| Operator | Syntax | Description |
|---|---|---|
| AND | `a & b` | Bits set in both |
| OR | `a \| b` | Bits set in either |
| XOR | `a ^ b` | Bits set in exactly one |
| NOT | `!a` | Flip all bits |
| Left shift | `a << n` | Shift bits left by `n` |
| Right shift | `a >> n` | Shift bits right by `n` (zero-fill for unsigned) |

| Method | Description |
|---|---|
| `val.count_ones()` | Number of `1` bits (popcount) |
| `val.count_zeros()` | Number of `0` bits |
| `val.leading_zeros()` | Leading zero bits |
| `val.trailing_zeros()` | Trailing zero bits |
| `val.rotate_left(n)` | Rotate bits left |
| `val.rotate_right(n)` | Rotate bits right |
| `val.wrapping_shl(n)` | Shift left, wrapping on overflow |
| `val.checked_shl(n)` | Shift left, returns `None` on overflow |
| `val.reverse_bits()` | Reverse the bit order |
