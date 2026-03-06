# Lesson 03: Columnar Vectors

## What You're Building
The fundamental data container for a columnar database: a Vector that stores values of
a single type in a contiguous byte buffer (`Vec<u8>`), plus a ValidityMask bitmask to
track NULLs. Unlike row-oriented storage, columnar vectors let the engine process one
column at a time, enabling SIMD-friendly tight loops and better cache utilization.

## Rust Concepts You'll Need
- [Bitwise Ops](../concepts/bitwise_ops.md) -- ValidityMask uses bit shifting and masking on u64 words
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- Vector stores data as Vec<u8> but exposes typed slices
- [Unsafe Rust](../concepts/unsafe_rust.md) -- get_data_slice reinterprets a byte buffer as a typed slice

## Key Patterns

### Bitmask Operations
A bitmask packs 64 booleans into a single `u64`. To find which word and which bit
an index maps to, divide and mod by 64.

```rust
// Analogy: attendance tracker for a classroom (NOT the QuackDB solution)
struct Attendance {
    words: Vec<u64>,
}

impl Attendance {
    fn mark_present(&mut self, student_id: usize) {
        let word_idx = student_id / 64;
        let bit_idx = student_id % 64;
        self.words[word_idx] |= 1u64 << bit_idx;   // set bit
    }

    fn is_present(&self, student_id: usize) -> bool {
        let word_idx = student_id / 64;
        let bit_idx = student_id % 64;
        (self.words[word_idx] >> bit_idx) & 1 == 1  // test bit
    }

    fn mark_absent(&mut self, student_id: usize) {
        let word_idx = student_id / 64;
        let bit_idx = student_id % 64;
        self.words[word_idx] &= !(1u64 << bit_idx); // clear bit
    }
}
```

### Unsafe Type-Punning for Typed Slices
The Vector stores all data as `Vec<u8>`, but consumers need `&[i32]` or `&[f64]`.
Use `std::slice::from_raw_parts` to reinterpret the byte buffer.

```rust
// Analogy: reading sensor readings from a binary buffer
fn read_temperatures(buf: &[u8]) -> &[f32] {
    let count = buf.len() / std::mem::size_of::<f32>();
    unsafe {
        std::slice::from_raw_parts(buf.as_ptr() as *const f32, count)
    }
}
```

### Enum Dispatch for Physical Types
`get_value` and `set_value` must handle each physical type differently. Match on
the logical type to determine how many bytes to read and how to interpret them.

```rust
// Analogy: a sensor log that stores different sensor types
enum SensorReading { TempC(f32), Humidity(u16), Switch(bool) }

fn read_sensor(sensor_type: &str, buf: &[u8], offset: usize) -> SensorReading {
    match sensor_type {
        "temp" => {
            let bytes: [u8; 4] = buf[offset..offset+4].try_into().unwrap();
            SensorReading::TempC(f32::from_le_bytes(bytes))
        }
        "humidity" => {
            let bytes: [u8; 2] = buf[offset..offset+2].try_into().unwrap();
            SensorReading::Humidity(u16::from_le_bytes(bytes))
        }
        _ => SensorReading::Switch(buf[offset] != 0),
    }
}
```

## Step-by-Step Implementation Order
1. Start with `ValidityMask::new_all_valid()` -- compute `(count + 63) / 64` words, fill with `u64::MAX` (all bits set = all valid)
2. Implement `is_valid()` and `set_valid()` -- use word_index = index/64, bit_index = index%64
3. Implement `all_valid()` and `count_valid()` -- use `count_ones()` on each word, but be careful about extra bits in the last word
4. Implement `Vector::new()` -- allocate `count * byte_width` bytes for fixed-size types; set up empty offsets vec for Varchar
5. Implement `get_data_slice<T>` and `get_data_slice_mut<T>` -- unsafe from_raw_parts with ptr cast
6. Implement `set_value()` and `get_value()` -- match on logical_type, use byte_width to index into data buffer; for Varchar, use the offsets array
7. Implement `new_constant()` -- store a single value, mark VectorType::Constant
8. Implement `flatten()` -- replicate the single constant value across count entries, change to VectorType::Flat
9. Watch out for: Varchar needs separate handling with offsets; the validity mask must be checked in get_value to decide whether to return a Null

## Reading the Tests
- **`test_validity_mask_set`** creates an all-valid mask of 64 entries, invalidates index 10, checks neighbors are still valid, and verifies `count_valid()` is 63. This confirms your bit manipulation must toggle individual bits without affecting neighbors.
- **`test_vector_get_typed_slice`** creates an Int32 vector, gets `&mut [i32]` from it, writes values, then reads them back via `&[i32]`. This is the core unsafe pattern -- your byte buffer must be correctly sized and aligned for i32 access.
