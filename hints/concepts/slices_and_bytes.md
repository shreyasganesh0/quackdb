# Slices and Bytes

> **Prerequisites:** [ownership_and_borrowing](./ownership_and_borrowing.md)

## Quick Reference
- `&[T]` = shared (immutable) slice, `&mut [T]` = mutable slice
- `&arr[1..4]` borrows elements at indices 1, 2, 3 (exclusive end)
- `b"hello"` = byte string literal of type `&[u8; 5]` (coerces to `&[u8]`)
- `.get(i)` returns `Option<&T>` (safe), `[i]` panics on out-of-bounds
- `.chunks(n)` and `.chunks_exact(n)` split into fixed-size sub-slices

## Common Compiler Errors

**`error[E0277]: the size for values of type '[T]' cannot be known at compilation time`**
You used `[T]` (unsized) instead of `&[T]` (a reference to a slice).
Fix: slices are always behind a reference: use `&[T]` or `&mut [T]`.

**`error[E0502]: cannot borrow 'v' as mutable because it is also borrowed as immutable`**
You held a slice reference to a Vec and then tried to mutate the Vec.
Fix: ensure the slice reference is no longer in use before mutating, or use `.split_at_mut()`.

**`error[E0507]: cannot move out of index of 'Vec<String>'`**
You tried `let s = vec[0];` on a non-Copy type.
Fix: use `&vec[0]` to borrow, `.clone()` to copy, or `.remove(0)` to take it out.

## When You'll Use This
- **Lesson 3 (Vectors):** `Vector` stores data as `Vec<u8>` but exposes typed slices
- **Lesson 7 (Bitpack/Delta):** working with `Vec<u8>` as a bitstream
- **Lesson 9 (Pages):** working with `&[u8]` slices and `Vec<u8>` buffers
- **Lesson 35 (SIMD):** all functions operate on `&[T]` and `&mut [T]` slices

## What This Is

A **slice** in Rust is a dynamically-sized view into a contiguous sequence of elements. Written as `&[T]` (shared slice) or `&mut [T]` (mutable slice), a slice is essentially a fat pointer: it stores both a pointer to the data and a length. Slices do not own the data they reference; they borrow it from some owning collection like a `Vec<T>` or an array `[T; N]`.

If you come from Python, think of slices as similar to `memoryview` objects or NumPy array views -- they let you look at a portion of a buffer without copying it. In JavaScript, the closest analog is a `TypedArray` backed by an `ArrayBuffer`. In C++, `std::span<T>` (C++20) is almost exactly the same concept: a non-owning view of contiguous elements with a known length.

Byte slices (`&[u8]`) are particularly important in systems programming. They represent raw memory as a sequence of bytes, and they appear everywhere: file I/O, network buffers, serialization, and page-level storage. Rust provides safe abstractions for working with bytes while still giving you the low-level control you need for database internals, binary protocols, and memory-mapped files.

## Syntax

```rust
fn main() {
    // Creating slices from arrays and vectors
    let arr: [i32; 5] = [10, 20, 30, 40, 50];
    let full_slice: &[i32] = &arr;          // slice of the whole array
    let partial: &[i32] = &arr[1..4];       // elements at index 1, 2, 3 => [20, 30, 40]

    let mut vec = vec![1, 2, 3, 4];
    let slice: &[i32] = &vec;               // immutable slice of the vector
    let mut_slice: &mut [i32] = &mut vec;   // mutable slice -- can modify elements in place
    mut_slice[0] = 99;

    // Byte slices
    let bytes: &[u8] = b"hello";            // byte string literal => &[u8; 5], coerces to &[u8]
    let raw: &[u8] = &[0xFF, 0x00, 0xAB];  // explicit byte array

    // Slice length and emptiness
    assert_eq!(partial.len(), 3);
    assert!(!partial.is_empty());

    // Subslicing with ranges
    let first_two: &[i32] = &arr[..2];      // [10, 20]
    let last_two: &[i32] = &arr[3..];       // [40, 50]
}
```

## Common Patterns

### Reading Fixed-Size Fields from a Byte Buffer

When parsing binary formats (headers, pages, records), you often need to read
integers from a byte buffer at known offsets.

```rust
fn read_u32_le(buf: &[u8], offset: usize) -> u32 {
    // Grab a 4-byte sub-slice, convert to fixed-size array, interpret as little-endian
    let bytes: [u8; 4] = buf[offset..offset + 4]
        .try_into()
        .expect("slice must be at least 4 bytes from offset");
    u32::from_le_bytes(bytes)
}

fn main() {
    let page: Vec<u8> = vec![0; 4096];
    let page_id = read_u32_le(&page, 0);  // pass Vec as &[u8] automatically
    println!("page_id = {page_id}");
}
```

### Splitting a Slice into Chunks

Processing data in fixed-size blocks is common for page-oriented storage or
SIMD-friendly layouts.

```rust
fn sum_columns(data: &[f64], num_columns: usize) -> Vec<f64> {
    let mut totals = vec![0.0; num_columns];
    // chunks_exact splits the slice into non-overlapping chunks
    for row in data.chunks_exact(num_columns) {
        for (i, val) in row.iter().enumerate() {
            totals[i] += val;
        }
    }
    totals
}

fn main() {
    // 3 rows x 2 columns stored in row-major order
    let flat_data = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
    let sums = sum_columns(&flat_data, 2);
    assert_eq!(sums, vec![9.0, 12.0]); // col0: 1+3+5, col1: 2+4+6
}
```

### Using `std::mem::size_of` for Type-Punning Calculations

When you need to know how many bytes a type occupies, `size_of` is your tool.
This is essential for computing buffer sizes and pointer arithmetic.

```rust
use std::mem;

#[repr(C)] // guarantees C-compatible memory layout
struct RecordHeader {
    record_len: u32,
    flags: u16,
    _padding: u16,
}

fn main() {
    // size_of returns the size in bytes
    println!("i32 = {} bytes", mem::size_of::<i32>());           // 4
    println!("f64 = {} bytes", mem::size_of::<f64>());           // 8
    println!("RecordHeader = {} bytes", mem::size_of::<RecordHeader>()); // 8

    // Calculate how many records fit in a 4 KiB page
    let page_size = 4096usize;
    let header_size = mem::size_of::<RecordHeader>();
    let max_records = page_size / header_size;
    println!("max records per page: {max_records}");
}
```

## Gotchas

**1. Out-of-bounds indexing panics at runtime.**
Unlike C/C++ where out-of-bounds access is undefined behavior, Rust panics
(crashes cleanly). However, panicking in production is still bad. Use `.get()`
to return an `Option` instead:
```rust
let data = [10, 20, 30];
// data[5];          // PANIC at runtime!
let safe = data.get(5); // Returns None
```

**2. You cannot return a slice that borrows from a local variable.**
Slices are borrows, so the data they reference must outlive the slice. This
is a common stumbling block for newcomers:
```rust
// DOES NOT COMPILE:
// fn make_slice() -> &[i32] {
//     let v = vec![1, 2, 3];
//     &v  // v is dropped at end of function, slice would dangle
// }

// Instead, return the owned Vec:
fn make_vec() -> Vec<i32> {
    vec![1, 2, 3]
}
```

**3. Mutable slice access follows the "one mutable XOR many shared" rule.**
You can have either one `&mut [T]` or many `&[T]` to the same data, never
both simultaneously. If you need to mutate two non-overlapping parts of a
slice, use `split_at_mut`:
```rust
let mut data = [1, 2, 3, 4, 5];
let (left, right) = data.split_at_mut(3);
left[0] = 100;   // OK -- left and right don't overlap
right[1] = 200;  // OK
```

## Related Concepts

- [Ownership and Borrowing](./ownership_and_borrowing.md) -- slices are borrowed views; the owner controls the data's lifetime
- [String Types](./string_types.md) -- `&str` is a specialized UTF-8 slice; `&[u8]` is a raw byte slice
- [Unsafe Rust](./unsafe_rust.md) -- reinterpreting byte buffers as typed slices often requires unsafe
- [Collections](./collections.md) -- `Vec<T>` owns the data that `&[T]` borrows from

## Quick Reference

| Operation | Syntax | Notes |
|---|---|---|
| Full slice from array/vec | `&arr`, `&vec` | Coerces automatically |
| Range slice | `&arr[1..4]` | Indices 1, 2, 3 (exclusive end) |
| Mutable slice | `&mut vec[..]` | Requires `mut` binding |
| Byte string literal | `b"hello"` | Type is `&[u8; 5]`, coerces to `&[u8]` |
| Length | `slice.len()` | Returns `usize` |
| Safe indexing | `slice.get(i)` | Returns `Option<&T>` |
| Split into chunks | `slice.chunks(n)` | Iterator of sub-slices |
| Split into exact chunks | `slice.chunks_exact(n)` | Panics if remainder exists |
| Split mutably | `slice.split_at_mut(mid)` | Two non-overlapping `&mut` slices |
| Copy between slices | `dst.copy_from_slice(src)` | Lengths must match |
| Size of a type | `std::mem::size_of::<T>()` | Compile-time constant |
| Convert slice to array | `slice.try_into()` | Returns `Result<[T; N], _>` |
| Sort a mutable slice | `slice.sort()` | In-place, stable sort |
