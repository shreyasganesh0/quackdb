# Lesson 07: Bitpacking & Delta Encoding

## What You're Building
Two complementary compression techniques that shine on integer columns. Bitpacking
stores each value using only the minimum bits needed (e.g., values 0-15 need only 4
bits instead of 32). Delta encoding stores differences between consecutive values,
which are often tiny for sorted or sequential data like timestamps. Combined, they can
compress a column of sequential timestamps from 8 bytes/value down to fractions of a
byte.

**Core concept count: 2** — bitpacking and delta encoding. Everything else (frame-of-reference, bits_required, compression_ratio) is scaffolding that supports these two.

> **Unified Concept:** While this lesson spans two files, both serve ONE concept: integer compression. Bitpacking reduces bit-width, delta reduces value magnitude. They combine in `delta_bitpack`. Think of them as two stages of a single compression pipeline, split into separate files only for code organization.

## Concept Recap
Building on Lesson 06: You used dictionary encoding to replace repeated values with integer codes. Now you will compress those integers themselves -- bitpacking shrinks the codes (or any integer column) by using only the bits needed, and delta encoding transforms sequential data into tiny differences before packing.

## Rust Concepts You'll Need
- [Bitwise Ops](../concepts/bitwise_ops.md) -- shifting, masking, and packing bits into byte buffers
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- working with Vec<u8> as a bitstream

## Key Patterns

### Determining Bits Required
The minimum bits to represent a value is `floor(log2(value)) + 1`, but you can
compute it without floating point using bit operations. Think of it like finding how
many digits a number needs: 99 needs 2 decimal digits, 100 needs 3.

```rust
// Analogy: how many digits to represent a number in a given base
fn decimal_digits(n: u64) -> u32 {
    if n == 0 { return 1; }
    (n as f64).log10().floor() as u32 + 1
}

// For binary, use the bit length:
fn bit_length(n: u64) -> u8 {
    if n == 0 { return 0; }
    64 - n.leading_zeros() as u8
}
```

### Bitpacking: Shift and Mask
To pack values at arbitrary bit widths, maintain a bit-level cursor into a byte
buffer. For each value, write `bit_width` bits starting at the current bit offset.
Think of filling a jar with marbles of a fixed size -- each marble takes exactly
the same amount of space, and you pack them in sequence.

```rust
// Analogy: packing 3-bit RGB channel indices into a byte stream
fn pack_3bit_values(values: &[u8]) -> Vec<u8> {
    let total_bits = values.len() * 3;
    let mut output = vec![0u8; (total_bits + 7) / 8];
    let mut bit_pos = 0usize;
    for &val in values {
        let byte_idx = bit_pos / 8;
        let bit_offset = bit_pos % 8;
        // Write lower bits into current byte
        output[byte_idx] |= (val & 0x07) << bit_offset;
        // If it spills into the next byte, write the overflow
        if bit_offset + 3 > 8 {
            output[byte_idx + 1] |= (val & 0x07) >> (8 - bit_offset);
        }
        bit_pos += 3;
    }
    output
}
```

### Delta Encoding as Differences
Store the first value as a base, then record `data[i] - data[i-1]` for each subsequent
value. Decoding is a prefix sum starting from the base. Think of recording your daily
spending versus your total balance -- the deltas (daily amounts) are usually much
smaller than the totals.

```rust
// Analogy: recording elevation changes on a hike
fn elevation_deltas(readings: &[i64]) -> (i64, Vec<i64>) {
    if readings.is_empty() { return (0, vec![]); }
    let base = readings[0];
    let deltas: Vec<i64> = readings.windows(2)
        .map(|w| w[1] - w[0])
        .collect();
    (base, deltas)
}

fn reconstruct_elevations(base: i64, deltas: &[i64]) -> Vec<i64> {
    let mut result = vec![base];
    for &d in deltas {
        result.push(*result.last().unwrap() + d);
    }
    result
}
```

### Frame-of-Reference as Offsets from Minimum
Subtract the minimum value from every element, producing non-negative offsets that
can be bitpacked efficiently. Store the minimum separately. Think of normalizing
test scores: instead of storing scores 85-100, store offsets 0-15 from the minimum.

```rust
// Analogy: normalizing temperature readings relative to the coldest
fn normalize_temps(temps: &[i64]) -> (i64, Vec<u64>) {
    let min = *temps.iter().min().unwrap();
    let offsets: Vec<u64> = temps.iter().map(|&t| (t - min) as u64).collect();
    (min, offsets)
}
```

## Where to Start
Start with `bitpack.rs` — it is self-contained and shorter. Once pack/unpack work, move to `delta.rs` where encode/decode are simple consecutive differences. The combined `delta_bitpack` functions tie them together at the end.

## Step-by-Step Implementation Order
1. Start with `bits_required()` -- handle 0 as a special case (return 0), otherwise use `64 - value.leading_zeros()` cast to u8
2. Implement `pack()` for u32 -- iterate values, for each one write `bit_width` bits at the current bit offset into the output Vec<u8>; handle spanning across byte boundaries
3. Implement `unpack()` -- reverse the process: read `bit_width` bits at each position, apply a mask of `(1 << bit_width) - 1`; watch out for the 32-bit overflow when bit_width is 32
4. Implement `pack_u64()` and `unpack_u64()` -- same logic but values are u64; bit_width can be up to 64
5. Implement `compression_ratio()` -- `original_bits as f64 / packed_bits as f64`
6. Implement `delta::encode()` -- store base = data[0], compute deltas via consecutive differences
7. Implement `delta::decode()` -- prefix sum from base
8. Implement `frame_of_reference_encode/decode` -- find min, subtract, cast to u64; decode adds min back
9. Implement `delta_bitpack_encode()` -- first delta-encode, then frame-of-reference the deltas to get u64 offsets, then bitpack; store metadata (base, min_delta, bit_width, count) in a header

## Common Mistakes
- **Mask overflow at full width**: When `bit_width` is 32, the expression `(1u32 << 32) - 1` overflows. Use `u32::MAX` (or `u64::MAX` for 64-bit) as a special case, or compute the mask as `(1u64 << bit_width) - 1` using a wider type.
- **Forgetting to handle byte-boundary spanning in pack/unpack**: A value's bits often split across two (or even three for u64) bytes. After writing the lower portion into the current byte, you must write the upper portion into the next byte.
- **Incorrect delta decode (prefix sum)**: Each decoded value is `base + sum of all deltas up to that point`. If you use only the previous delta instead of the cumulative sum, you will get wrong results for non-constant differences.

## Reading the Tests
- **`test_bits_required`** checks that 0 needs 0 bits, 1 needs 1 bit, 3 needs 2 bits, 255 needs 8, 256 needs 9, and u32::MAX needs 32. This confirms the formula must handle 0 specially and return the exact ceiling.
- **`test_bitpack_roundtrip_1bit`** packs and unpacks 8 values of 0s and 1s at 1-bit width. This is the simplest packing case and confirms your basic bit manipulation works.
- **`test_bitpack_roundtrip_4bits`** packs 100 values (0-15 repeating) at 4-bit width. At 4 bits, two values fit per byte. This tests the common case where values pack evenly.
- **`test_bitpack_roundtrip_various_widths`** loops bit_width from 1 to 32, packs 64 values, and unpacks. This means your pack/unpack must work for every possible width without off-by-one errors. This is the most comprehensive correctness test.
- **`test_bitpack_compression`** packs 1000 zeros at 1-bit width and checks the output is much smaller than 4000 bytes. This confirms your output buffer sizing: 1000 bits = 125 bytes.
- **`test_bitpack_u64`** packs and unpacks u64 values, using `bits_required` on the max value to determine bit width. This tests the 64-bit variant.
- **`test_bitpack_compression_ratio`** checks that `compression_ratio(32, 4)` returns approximately 8.0 (32/4). This reveals the formula is original_bits / packed_bits.
- **`test_delta_encode_sequential`** encodes 100-109 and checks base is 100 and all deltas are 1. Sequential integers produce constant deltas.
- **`test_delta_roundtrip`** encodes non-uniform data `[100, 105, 103, 110, 108]` and verifies lossless decode. Deltas can be negative.
- **`test_delta_negative`** encodes a decreasing sequence `[100, 90, 80, 70, 60]` where all deltas are -10. This confirms your implementation handles negative deltas.
- **`test_delta_single`** encodes a single element, expecting base=42 and empty deltas. Edge case.
- **`test_frame_of_reference`** checks explicit expected offsets `[0, 1, 5, 3, 2]` for input `[1000, 1001, 1005, 1003, 1002]`. The minimum is 1000.
- **`test_delta_bitpack_combined`** encodes 1000 sequential timestamps and expects > 8x compression. Since deltas are all 1, they need only 1 bit each after frame-of-reference, yielding about 125 bytes for 8000 bytes of input.
- **`test_delta_empty`** encodes an empty vec and checks the roundtrip produces empty output.

## What Comes Next
You now have four compression algorithms: RLE, dictionary, bitpacking, and delta.
Lesson 08 builds the **compression framework** that wraps compressed data in
self-describing frames and automatically selects the best algorithm based on data
characteristics. Your encoders and decoders from Lessons 05-07 will be called by
the framework's `auto_compress` and `decompress` functions.
