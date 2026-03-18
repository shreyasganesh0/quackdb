# Lesson 03: Columnar Vectors

## What You're Building
The fundamental data container for a columnar database: a Vector that stores values of
a single type in a contiguous byte buffer (`Vec<u8>`), plus a ValidityMask bitmask to
track NULLs. Unlike row-oriented storage, columnar vectors let the engine process one
column at a time, enabling SIMD-friendly tight loops and better cache utilization.

## Concept Recap
Building on Lesson 02: You defined `LogicalType` (for column schemas) and `ScalarValue` (for individual values). Now you will build the `Vector` that stores entire columns of data, using `LogicalType.byte_width()` to size the internal byte buffer and `ScalarValue` for get/set operations.

## Rust Concepts You'll Need
- [Bitwise Ops](../concepts/bitwise_ops.md) -- ValidityMask uses bit shifting and masking on u64 words
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- Vector stores data as Vec<u8> but exposes typed slices
- [Unsafe Rust](../concepts/unsafe_rust.md) -- get_data_slice reinterprets a byte buffer as a typed slice

## Key Patterns

### Bitmask Operations
A bitmask packs 64 booleans into a single `u64`. To find which word and which bit
an index maps to, divide and mod by 64. Think of a hotel room keycard board -- each
slot represents a room, and a card present means "occupied."

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
Use `std::slice::from_raw_parts` to reinterpret the byte buffer. This is like a
universal socket adapter -- the electricity is the same, but the shape of the plug
changes depending on the country (type).

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
Think of it like a mail sorter: each type of package goes to a different handling
station, but they all travel on the same conveyor belt (the byte buffer).

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

### Constant vs Flat Vectors
A constant vector stores a single value that logically repeats for all rows.
Flattening materializes that value into every slot. Think of a "fill color" in a
spreadsheet -- one setting applies to all cells until you expand them individually.

```rust
// Analogy: a default value that expands on demand
struct DefaultFill<T: Clone> {
    value: T,
    count: usize,
}

impl<T: Clone> DefaultFill<T> {
    fn expand(&self) -> Vec<T> {
        vec![self.value.clone(); self.count]
    }
}
```

## Step-by-Step Implementation Order
1. Start with `ValidityMask::new_all_valid()` -- compute `(count + 63) / 64` words, fill with `u64::MAX` (all bits set = all valid)
2. Implement `new_all_invalid()` -- same word count but fill with 0
3. Implement `is_valid()` and `set_valid()` -- use word_index = index/64, bit_index = index%64; set uses OR to set a bit, AND with NOT to clear
4. Implement `set_valid_range()` -- validate a range of indices efficiently
5. Implement `all_valid()` and `count_valid()` -- use `count_ones()` on each word, but be careful about extra bits in the last word
6. Implement `resize()` -- grow the words vector, preserving existing state, and fill new bits as valid
7. Implement `Vector::new()` -- allocate `count * byte_width` bytes for fixed-size types; set up empty offsets for Varchar
8. Implement `get_data_slice<T>` and `get_data_slice_mut<T>` -- unsafe from_raw_parts with ptr cast
9. Implement `set_value()`, `get_value()`, `set_null()` -- match on logical_type, use byte_width to index into data buffer; for Varchar, use the offsets; check validity in get_value

## Common Mistakes
- **Extra bits in the last u64 word**: If you have 100 entries, you need 2 words (128 bits), but only 100 are meaningful. When counting valid entries with `count_ones()`, mask or account for the unused 28 bits in the last word, or `all_valid()` may give wrong answers.
- **Forgetting to check the validity mask in `get_value`**: When a position is marked invalid (NULL), `get_value` should return a Null ScalarValue rather than reading garbage bytes from the buffer.
- **Not keeping Vector count in sync**: When using `set_value` or `set_null`, ensure the vector's `count` field reflects how many values are logically stored. Tests like `test_vector_flat_int32` explicitly call `set_count(10)` before writing values.

## Reading the Tests
- **`test_validity_mask_all_valid`** creates a 100-entry all-valid mask and checks `all_valid()`, `count_valid() == 100`, and that every individual index is valid. This confirms your word initialization with `u64::MAX` and your `is_valid` logic.
- **`test_validity_mask_all_invalid`** creates a 100-entry all-invalid mask and verifies `count_valid() == 0` and no index is valid. This tests the zero-filled path.
- **`test_validity_mask_set`** creates an all-valid mask of 64 entries, invalidates index 10, checks neighbors 9 and 11 are still valid, and verifies `count_valid()` is 63. It then re-validates index 10. This confirms your bit manipulation must toggle individual bits without affecting neighbors.
- **`test_validity_mask_range`** starts all-invalid, calls `set_valid_range(10, 20)` to validate 20 entries starting at index 10, and checks boundaries precisely. Indices 0-9 must be invalid, 10-29 valid, 30-99 invalid.
- **`test_validity_mask_resize`** creates a 50-entry mask, invalidates index 25, resizes to 100, and checks that index 25 is still invalid while new indices 50-99 are valid. Resize must preserve existing state.
- **`test_vector_flat_int32`** creates an Int32 vector, writes 10 values, reads them back, and checks `vector_type() == Flat` and `count() == 10`. This is the basic read/write path for fixed-size types.
- **`test_vector_nulls`** sets some values and marks indices 2 and 4 as null, then checks the validity mask and reads back non-null values. This confirms the interaction between `set_null` and the validity mask.
- **`test_vector_get_typed_slice`** creates an Int32 vector, gets `&mut [i32]` from it, writes values, then reads them back via `&[i32]`. This is the core unsafe pattern -- your byte buffer must be correctly sized and aligned for i32 access.
- **`test_vector_constant`** creates a constant vector of `Int32(42)` with count 100 and verifies that `get_value` returns 42 at indices 0, 50, and 99. The `vector_type()` must be `Constant`.
- **`test_vector_flatten`** creates a constant Int32(7) vector with 5 entries, calls `flatten()`, and checks that it becomes Flat with value 7 at every index. Flatten materializes the constant.
- **`test_selection_vector`** and **`test_selection_vector_incrementing`** verify that SelectionVector stores and retrieves index mappings correctly. The incrementing variant produces [0, 1, 2, ...].
- **`test_vector_copy_with_selection`** writes 10 values (0, 100, 200, ...), creates a selection [1, 3, 7], copies to a new vector, and checks the result is [100, 300, 700]. This is how filtered scans produce output.
- **`test_vector_string`** and **`test_vector_string_empty`** verify Varchar vectors can store and retrieve strings, including empty strings (which must not be confused with NULL).

## Rust Sidebar: Unsafe Type-Punning
If you hit `cannot cast *const u8 to *const i32` or `expected &[i32], found &[u8]`, here's what's happening: the `Vector` stores all data as `Vec<u8>`, but you need to read it as `&[i32]` or `&[f64]`. Rust's type system does not allow reinterpreting bytes as typed data without `unsafe`, because alignment and validity are not guaranteed at compile time.
The fix: `unsafe { std::slice::from_raw_parts(self.data.as_ptr() as *const T, count) }`. This works because your `Vector::new()` allocates `count * byte_width` bytes -- enough room and correct alignment for the target type. The `as *const T` cast reinterprets the byte pointer as a typed pointer.

## What Comes Next
With Vector and ValidityMask, you can store a single column. Lesson 04 bundles
multiple Vectors into a **DataChunk** -- the row-group batch that flows through
the query engine. Your `Vector` API (get_value, set_value, flatten) will be called
heavily by DataChunk methods.
