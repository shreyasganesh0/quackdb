//! Lesson 07: Bitpacking & Delta Encoding Tests

use quackdb::compression::bitpack;
use quackdb::compression::delta;

/// Helper: bitpack values at the given bit width and verify the roundtrip is lossless.
fn assert_bitpack_roundtrip(values: &[u32], bit_width: u8) {
    let packed = bitpack::pack(values, bit_width);
    let unpacked = bitpack::unpack(&packed, bit_width, values.len());
    assert_eq!(unpacked, values, "bitpack roundtrip failed for bit_width={}", bit_width);
}

/// Helper: delta-encode then decode data and verify the roundtrip is lossless.
fn assert_delta_roundtrip(data: &[i64]) {
    let encoded = delta::encode(data);
    let decoded = delta::decode(&encoded);
    assert_eq!(decoded, data, "delta encode/decode roundtrip must be lossless");
}

#[test]
fn test_bits_required() {
    assert_eq!(bitpack::bits_required(0), 0, "zero needs no bits to represent");
    assert_eq!(bitpack::bits_required(1), 1);
    assert_eq!(bitpack::bits_required(2), 2, "value 2 requires 2 bits (binary 10)");
    assert_eq!(bitpack::bits_required(3), 2);
    assert_eq!(bitpack::bits_required(255), 8, "max single-byte value needs exactly 8 bits");
    assert_eq!(bitpack::bits_required(256), 9, "256 overflows 8 bits, needs 9");
    assert_eq!(bitpack::bits_required(u32::MAX as u64), 32);
}

#[test]
fn test_bitpack_roundtrip_1bit() {
    let values: Vec<u32> = vec![0, 1, 0, 1, 1, 0, 0, 1];
    assert_bitpack_roundtrip(&values, 1);
}

#[test]
fn test_bitpack_roundtrip_4bits() {
    let values: Vec<u32> = (0..100).map(|i| i % 16).collect();
    assert_bitpack_roundtrip(&values, 4);
}

#[test]
fn test_bitpack_roundtrip_various_widths() {
    for bit_width in 1..=32 {
        let max_val = if bit_width == 32 { u32::MAX } else { (1u32 << bit_width) - 1 };
        let values: Vec<u32> = (0..64).map(|i| (i as u32) % (max_val + 1).max(1)).collect();
        let packed = bitpack::pack(&values, bit_width);
        let unpacked = bitpack::unpack(&packed, bit_width, values.len());
        assert_eq!(unpacked, values, "Failed for bit_width={}", bit_width);
    }
}

#[test]
fn test_bitpack_compression() {
    let values: Vec<u32> = vec![0; 1000];
    let bit_width = 1;
    let packed = bitpack::pack(&values, bit_width);
    // 1000 values * 1 bit = 125 bytes vs 4000 bytes original
    assert!(packed.len() < 4000, "1-bit packing of 1000 values should use ~125 bytes, not 4000");
}

#[test]
fn test_bitpack_u64() {
    let values: Vec<u64> = vec![100, 200, 300, 150, 250];
    let bit_width = bitpack::bits_required(*values.iter().max().unwrap());
    let packed = bitpack::pack_u64(&values, bit_width);
    let unpacked = bitpack::unpack_u64(&packed, bit_width, values.len());
    assert_eq!(unpacked, values);
}

#[test]
fn test_bitpack_compression_ratio() {
    let ratio = bitpack::compression_ratio(32, 4);
    assert!((ratio - 8.0).abs() < 0.01, "32/4 = 8x compression");
}

#[test]
fn test_delta_encode_sequential() {
    let data: Vec<i64> = (100..110).collect();
    let encoded = delta::encode(&data);
    assert_eq!(encoded.base, 100, "base stores the first value of the sequence");
    // All deltas should be 1
    for d in &encoded.deltas {
        assert_eq!(*d, 1, "sequential integers have a constant delta of 1");
    }
}

#[test]
fn test_delta_roundtrip() {
    let data: Vec<i64> = vec![100, 105, 103, 110, 108];
    assert_delta_roundtrip(&data);
}

#[test]
fn test_delta_negative() {
    let data: Vec<i64> = vec![100, 90, 80, 70, 60];
    assert_delta_roundtrip(&data);
}

#[test]
fn test_delta_single() {
    let data = vec![42i64];
    let encoded = delta::encode(&data);
    assert_eq!(encoded.base, 42, "single element is stored only as the base value");
    assert!(encoded.deltas.is_empty(), "no deltas needed for a single-element sequence");
    let decoded = delta::decode(&encoded);
    assert_eq!(decoded, data);
}

#[test]
fn test_frame_of_reference() {
    let data: Vec<i64> = vec![1000, 1001, 1005, 1003, 1002];
    let (min_val, offsets) = delta::frame_of_reference_encode(&data);
    assert_eq!(min_val, 1000, "frame-of-reference stores the minimum as the reference point");
    assert_eq!(offsets, vec![0, 1, 5, 3, 2], "offsets are the difference from the minimum value");

    let decoded = delta::frame_of_reference_decode(min_val, &offsets);
    assert_eq!(decoded, data);
}

#[test]
fn test_delta_bitpack_combined() {
    // Timestamps with constant 1-second intervals
    let data: Vec<i64> = (0..1000).map(|i| 1_700_000_000 + i).collect();
    let encoded = delta::delta_bitpack_encode(&data);
    let decoded = delta::delta_bitpack_decode(&encoded, data.len());
    assert_eq!(decoded, data);

    // Check compression ratio — timestamps should compress very well
    let original_size = data.len() * 8; // 8 bytes per i64
    let compressed_size = encoded.len();
    let ratio = original_size as f64 / compressed_size as f64;
    assert!(ratio > 8.0, "Expected >8x compression for sequential timestamps, got {}x", ratio);
}

#[test]
fn test_delta_empty() {
    let data: Vec<i64> = vec![];
    let encoded = delta::encode(&data);
    let decoded = delta::decode(&encoded);
    assert!(decoded.is_empty(), "delta encoding of empty input must round-trip to empty");
}
