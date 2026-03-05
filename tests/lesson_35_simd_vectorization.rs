//! Lesson 35: SIMD-Style Vectorization Tests

use quackdb::simd::*;

#[test]
fn test_vectorized_add_i32() {
    let a = vec![1, 2, 3, 4, 5];
    let b = vec![10, 20, 30, 40, 50];
    let mut out = vec![0i32; 5];
    vectorized_add_i32(&a, &b, &mut out);
    assert_eq!(out, vec![11, 22, 33, 44, 55]);
}

#[test]
fn test_vectorized_add_f64() {
    let a = vec![1.0, 2.0, 3.0];
    let b = vec![0.5, 0.5, 0.5];
    let mut out = vec![0.0f64; 3];
    vectorized_add_f64(&a, &b, &mut out);
    assert_eq!(out, vec![1.5, 2.5, 3.5]);
}

#[test]
fn test_vectorized_filter_gt_i32() {
    let values = vec![1, 5, 3, 8, 2, 9, 4];
    let mut indices = vec![0u32; values.len()];
    let count = vectorized_filter_gt_i32(&values, 4, &mut indices);
    assert_eq!(count, 3); // 5, 8, 9
    let selected: Vec<i32> = indices[..count].iter().map(|&i| values[i as usize]).collect();
    assert!(selected.contains(&5));
    assert!(selected.contains(&8));
    assert!(selected.contains(&9));
}

#[test]
fn test_vectorized_filter_gt_f64() {
    let values = vec![1.0, 5.0, 3.0, 8.0];
    let mut indices = vec![0u32; values.len()];
    let count = vectorized_filter_gt_f64(&values, 4.0, &mut indices);
    assert_eq!(count, 2); // 5.0, 8.0
}

#[test]
fn test_vectorized_hash_i64() {
    let values = vec![1i64, 2, 3, 4, 5];
    let mut out = vec![0u64; 5];
    vectorized_hash_i64(&values, &mut out);

    assert_ne!(out[0], out[1]);
    assert_ne!(out[1], out[2]);

    let mut out2 = vec![0u64; 5];
    vectorized_hash_i64(&values, &mut out2);
    assert_eq!(out, out2);
}

#[test]
fn test_branchless_min_i32() {
    let a = vec![3, 1, 4, 1, 5];
    let b = vec![2, 7, 1, 8, 2];
    let mut out = vec![0i32; 5];
    branchless_min_i32(&a, &b, &mut out);
    assert_eq!(out, vec![2, 1, 1, 1, 2]);
}

#[test]
fn test_branchless_max_i32() {
    let a = vec![3, 1, 4, 1, 5];
    let b = vec![2, 7, 1, 8, 2];
    let mut out = vec![0i32; 5];
    branchless_max_i32(&a, &b, &mut out);
    assert_eq!(out, vec![3, 7, 4, 8, 5]);
}

#[test]
fn test_vectorized_add_nullable() {
    let a = vec![10, 20, 30, 40];
    let b = vec![1, 2, 3, 4];
    let validity_a = vec![0b1011u64]; // index 2 is null
    let validity_b = vec![0b1110u64]; // index 0 is null
    let mut out = vec![0i32; 4];
    let mut validity_out = vec![0u64; 1];

    vectorized_add_nullable_i32(&a, &b, &validity_a, &validity_b, &mut out, &mut validity_out);

    assert_eq!(out[1], 22);
    assert_eq!(out[3], 44);
}

#[test]
fn test_vectorized_sum_i32() {
    let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    let sum = vectorized_sum_i32(&values);
    assert_eq!(sum, 55);
}

#[test]
fn test_vectorized_sum_f64() {
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let sum = vectorized_sum_f64(&values);
    assert!((sum - 15.0).abs() < 1e-10);
}

#[test]
fn test_aligned_alloc() {
    let buf = aligned_alloc(256, 64);
    assert!(buf.len() >= 256);
    let ptr = buf.as_ptr() as usize;
    assert_eq!(ptr % 64, 0, "Expected 64-byte alignment, got ptr={:#x}", ptr);
}

#[test]
fn test_vectorized_large_batch() {
    let n = 10000;
    let a: Vec<i32> = (0..n).collect();
    let b: Vec<i32> = (0..n).map(|x| x * 2).collect();
    let mut out = vec![0i32; n as usize];

    vectorized_add_i32(&a, &b, &mut out);

    for i in 0..n as usize {
        assert_eq!(out[i], a[i] + b[i]);
    }
}

#[test]
fn test_vectorized_empty() {
    let a: Vec<i32> = vec![];
    let b: Vec<i32> = vec![];
    let mut out: Vec<i32> = vec![];
    vectorized_add_i32(&a, &b, &mut out);
    assert!(out.is_empty());
}

#[test]
fn test_vectorized_sum_empty() {
    let sum = vectorized_sum_i32(&[]);
    assert_eq!(sum, 0);
}
