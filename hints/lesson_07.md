# Lesson 07: Bitpacking & Delta Encoding

## What You're Building
Two complementary compression techniques that shine on integer columns. Bitpacking
stores each value using only the minimum bits needed (e.g., values 0-15 need only 4
bits instead of 32). Delta encoding stores differences between consecutive values,
which are often tiny for sorted or sequential data like timestamps. Combined, they can
compress a column of sequential timestamps from 8 bytes/value down to fractions of a
byte.

## Rust Concepts You'll Need
- [Bitwise Ops](../concepts/bitwise_ops.md) -- shifting, masking, and packing bits into byte buffers
- [Slices and Bytes](../concepts/slices_and_bytes.md) -- working with Vec<u8> as a bitstream

## Key Patterns

### Determining Bits Required
The minimum bits to represent a value is `floor(log2(value)) + 1`, but you can
compute it without floating point using bit operations.

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
value. Decoding is a prefix sum starting from the base.

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
can be bitpacked efficiently. Store the minimum separately.

```rust
// Analogy: normalizing temperature readings relative to the coldest
fn normalize_temps(temps: &[i64]) -> (i64, Vec<u64>) {
    let min = *temps.iter().min().unwrap();
    let offsets: Vec<u64> = temps.iter().map(|&t| (t - min) as u64).collect();
    (min, offsets)
}
```

## Step-by-Step Implementation Order
1. Start with `bits_required()` -- handle 0 as a special case (return 0), otherwise use `64 - value.leading_zeros()` cast to u8
2. Implement `pack()` for u32 -- iterate values, for each one write `bit_width` bits at the current bit offset into the output Vec<u8>; handle spanning across byte boundaries
3. Implement `unpack()` -- reverse the process: read `bit_width` bits at each position, apply a mask of `(1 << bit_width) - 1`
4. Implement `pack_u64()` and `unpack_u64()` -- same logic but values are u64; bit_width can be up to 64
5. Implement `compression_ratio()` -- simply `original_bits as f64 / packed_bits as f64`
6. Implement `delta::encode()` -- store base = data[0], compute deltas via consecutive differences
7. Implement `delta::decode()` -- prefix sum from base
8. Implement `frame_of_reference_encode/decode` -- find min, subtract, cast to u64; decode adds min back
9. Implement `delta_bitpack_encode()` -- first delta-encode, then frame-of-reference the deltas to get u64 offsets, then bitpack; store metadata (base, min_delta, bit_width, count) in a header
10. Implement `delta_bitpack_decode()` -- read the header, unpack, undo frame-of-reference, undo delta
11. Watch out for: bit_width=0 (all values are 0), empty input, and the mask for 32-bit or 64-bit width which would overflow `1u32 << 32`

## Reading the Tests
- **`test_bits_required`** checks that 0 needs 0 bits, 1 needs 1 bit, 3 needs 2 bits, 255 needs 8, 256 needs 9. This confirms the formula must handle 0 specially and return the exact ceiling.
- **`test_bitpack_roundtrip_various_widths`** loops bit_width from 1 to 32, packs 64 values, and unpacks. This means your pack/unpack must work for every possible width without off-by-one errors.
- **`test_delta_bitpack_combined`** encodes 1000 sequential timestamps and expects > 8x compression. Since deltas are all 1, they need only 1 bit each after frame-of-reference, yielding about 125 bytes for 8000 bytes of input.
- **`test_frame_of_reference`** checks explicit expected offsets `[0, 1, 5, 3, 2]` for input `[1000, 1001, 1005, 1003, 1002]`. The minimum is 1000.
