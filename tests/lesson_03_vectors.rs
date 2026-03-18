//! # Lesson 03: Columnar Vectors — Test Suite
//!
//! Tests are ordered from simple to complex:
//! 1. Basic validity mask construction (`test_validity_mask_all_valid`, `test_validity_mask_all_invalid`)
//! 2. Validity mask manipulation (`test_validity_mask_set`, `test_validity_mask_range`, `test_validity_mask_resize`)
//! 3. Flat vector construction and access (`test_vector_flat_int32`, `test_vector_float64`)
//! 4. Null handling (`test_vector_nulls`, `test_vector_constant_null`)
//! 5. Constant and special vector types (`test_vector_constant`, `test_vector_flatten`)
//! 6. Selection vectors and copy (`test_selection_vector`, `test_vector_copy_with_selection`)
//! 7. String vectors and edge cases (`test_vector_string`, `test_vector_string_empty`)
//! 8. Low-level typed slice access (`test_vector_get_typed_slice`)

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::vector::{ValidityMask, Vector, VectorType, SelectionVector};

/// Helper: create a flat Int32 vector pre-filled with the given values.
/// Eliminates the repetitive new/set_count/set_value loop used in many tests.
fn make_int32_vector(values: &[i32]) -> Vector {
    let mut vec = Vector::new(LogicalType::Int32, values.len());
    vec.set_count(values.len());
    for (i, &v) in values.iter().enumerate() {
        vec.set_value(i, ScalarValue::Int32(v));
    }
    vec
}

/// Helper: assert that a vector contains exactly the expected Int32 values.
fn assert_int32_values(vec: &Vector, expected: &[i32]) {
    assert_eq!(vec.count(), expected.len(), "vector length mismatch");
    for (i, &v) in expected.iter().enumerate() {
        assert_eq!(vec.get_value(i), ScalarValue::Int32(v), "mismatch at index {}", i);
    }
}

// ── 1. Basic validity mask construction ─────────────────────────────

#[test]
fn test_validity_mask_all_valid() {
    let mask = ValidityMask::new_all_valid(100);
    assert!(mask.all_valid(), "new_all_valid must produce a mask where all_valid() returns true");
    assert_eq!(mask.count_valid(), 100);
    for i in 0..100 {
        assert!(mask.is_valid(i));
    }
}

#[test]
fn test_validity_mask_all_invalid() {
    let mask = ValidityMask::new_all_invalid(100);
    assert!(!mask.all_valid());
    assert_eq!(mask.count_valid(), 0, "all-invalid mask should have zero valid entries");
    for i in 0..100 {
        assert!(!mask.is_valid(i));
    }
}

// ── 2. Validity mask manipulation ───────────────────────────────────

#[test]
fn test_validity_mask_set() {
    let mut mask = ValidityMask::new_all_valid(64);
    mask.set_valid(10, false);
    assert!(!mask.is_valid(10));
    assert!(mask.is_valid(9), "invalidating index 10 must not affect neighboring index 9");
    assert!(mask.is_valid(11), "invalidating index 10 must not affect neighboring index 11");
    assert_eq!(mask.count_valid(), 63, "invalidating one entry should decrement the valid count by one");

    mask.set_valid(10, true);
    assert!(mask.is_valid(10), "re-validating a previously invalidated index must restore it");
    assert_eq!(mask.count_valid(), 64);
}

#[test]
fn test_validity_mask_range() {
    let mut mask = ValidityMask::new_all_invalid(100);
    mask.set_valid_range(10, 20);
    for i in 0..10 {
        assert!(!mask.is_valid(i));
    }
    for i in 10..30 {
        assert!(mask.is_valid(i));
    }
    for i in 30..100 {
        assert!(!mask.is_valid(i));
    }
}

#[test]
fn test_validity_mask_resize() {
    let mut mask = ValidityMask::new_all_valid(50);
    mask.set_valid(25, false);
    mask.resize(100);
    assert!(!mask.is_valid(25), "resize must preserve existing validity state");
    // New entries should be valid
    for i in 50..100 {
        assert!(mask.is_valid(i), "newly added entries after resize should default to valid");
    }
}

#[test]
fn test_validity_mask_single_element() {
    // Edge case: single-element validity mask
    let mask = ValidityMask::new_all_valid(1);
    assert!(mask.all_valid(), "single-element all-valid mask should report all_valid()");
    assert_eq!(mask.count_valid(), 1);
    assert!(mask.is_valid(0));
}

// ── 3. Flat vector construction and access ──────────────────────────

#[test]
fn test_vector_flat_int32() {
    let expected: Vec<i32> = (0..10).map(|i| i * 10).collect();
    let vec = make_int32_vector(&expected);
    assert_int32_values(&vec, &expected);
    assert_eq!(vec.vector_type(), VectorType::Flat, "default vector storage should be flat (uncompressed)");
}

#[test]
fn test_vector_float64() {
    let mut vec = Vector::new(LogicalType::Float64, 3);
    vec.set_count(3);
    vec.set_value(0, ScalarValue::Float64(1.5));
    vec.set_value(1, ScalarValue::Float64(2.5));
    vec.set_value(2, ScalarValue::Float64(3.5));

    assert_eq!(vec.get_value(0), ScalarValue::Float64(1.5));
    assert_eq!(vec.get_value(1), ScalarValue::Float64(2.5));
    assert_eq!(vec.get_value(2), ScalarValue::Float64(3.5));
}

// ── 4. Null handling ────────────────────────────────────────────────

#[test]
fn test_vector_nulls() {
    let mut vec = Vector::new(LogicalType::Int64, 5);
    vec.set_count(5);
    vec.set_value(0, ScalarValue::Int64(100));
    vec.set_value(1, ScalarValue::Int64(200));
    vec.set_null(2);
    vec.set_value(3, ScalarValue::Int64(400));
    vec.set_null(4);

    assert!(vec.validity().is_valid(0));
    assert!(vec.validity().is_valid(1));
    assert!(!vec.validity().is_valid(2), "set_null must mark the entry as invalid in the validity mask");
    assert!(vec.validity().is_valid(3));
    assert!(!vec.validity().is_valid(4));

    assert_eq!(vec.get_value(0), ScalarValue::Int64(100));
    assert_eq!(vec.get_value(1), ScalarValue::Int64(200));
    assert_eq!(vec.get_value(3), ScalarValue::Int64(400));
}

#[test]
fn test_vector_all_nulls() {
    // Edge case: a vector where every element is null
    let mut vec = Vector::new(LogicalType::Int32, 3);
    vec.set_count(3);
    vec.set_null(0);
    vec.set_null(1);
    vec.set_null(2);

    assert!(!vec.validity().is_valid(0), "all elements should be null");
    assert!(!vec.validity().is_valid(1));
    assert!(!vec.validity().is_valid(2));
}

#[test]
fn test_vector_constant_null() {
    let vec = Vector::new_constant(ScalarValue::Null(LogicalType::Int32), 50);
    assert_eq!(vec.vector_type(), VectorType::Constant);
    for _i in 0..50 {
        assert!(!vec.validity().is_valid(0));
    }
}

// ── 5. Constant and special vector types ────────────────────────────

#[test]
fn test_vector_constant() {
    let vec = Vector::new_constant(ScalarValue::Int32(42), 100);
    assert_eq!(vec.vector_type(), VectorType::Constant, "constant vectors store one value for all rows");
    assert_eq!(vec.count(), 100);
    assert_eq!(vec.get_value(0), ScalarValue::Int32(42));
    assert_eq!(vec.get_value(50), ScalarValue::Int32(42), "constant vector must return the same value at any index");
    assert_eq!(vec.get_value(99), ScalarValue::Int32(42));
}

#[test]
fn test_vector_flatten() {
    let mut vec = Vector::new_constant(ScalarValue::Int32(7), 5);
    assert_eq!(vec.vector_type(), VectorType::Constant);
    vec.flatten();
    assert_eq!(vec.vector_type(), VectorType::Flat, "flatten must convert constant to flat representation");
    for i in 0..5 {
        assert_eq!(vec.get_value(i), ScalarValue::Int32(7), "flatten must materialize the constant value into every slot");
    }
}

// ── 6. Selection vectors and copy ───────────────────────────────────

#[test]
fn test_selection_vector() {
    let sel = SelectionVector::new(vec![2, 5, 8]);
    assert_eq!(sel.len(), 3);
    assert_eq!(sel.get(0), 2);
    assert_eq!(sel.get(1), 5);
    assert_eq!(sel.get(2), 8);
}

#[test]
fn test_selection_vector_incrementing() {
    let sel = SelectionVector::incrementing(5);
    assert_eq!(sel.len(), 5);
    for i in 0..5 {
        assert_eq!(sel.get(i), i as u32);
    }
}

#[test]
fn test_selection_vector_empty() {
    // Edge case: empty selection vector
    let sel = SelectionVector::new(vec![]);
    assert_eq!(sel.len(), 0, "empty selection vector must have length 0");
}

#[test]
fn test_vector_copy_with_selection() {
    let values: Vec<i32> = (0..10).map(|i| i * 100).collect();
    let src = make_int32_vector(&values);

    let sel = SelectionVector::new(vec![1, 3, 7]);
    let mut dst = Vector::new(LogicalType::Int32, 3);
    src.copy_with_selection(&sel, &mut dst);

    assert_int32_values(&dst, &[100, 300, 700]);
}

// ── 7. String vectors and edge cases ────────────────────────────────

#[test]
fn test_vector_string() {
    let mut vec = Vector::new(LogicalType::Varchar, 3);
    vec.append_string("hello");
    vec.append_string("world");
    vec.append_string("quack");

    assert_eq!(vec.get_string(0), Some("hello"));
    assert_eq!(vec.get_string(1), Some("world"));
    assert_eq!(vec.get_string(2), Some("quack"));
}

#[test]
fn test_vector_string_empty() {
    let mut vec = Vector::new(LogicalType::Varchar, 2);
    vec.append_string("");
    vec.append_string("notempty");

    assert_eq!(vec.get_string(0), Some(""), "empty strings must be stored and retrieved, not treated as NULL");
    assert_eq!(vec.get_string(1), Some("notempty"));
}

// ── 8. Low-level typed slice access ─────────────────────────────────

#[test]
fn test_vector_get_typed_slice() {
    let mut vec = Vector::new(LogicalType::Int32, 4);
    vec.set_count(4);
    {
        let slice: &mut [i32] = vec.get_data_slice_mut();
        slice[0] = 10;
        slice[1] = 20;
        slice[2] = 30;
        slice[3] = 40;
    }
    let slice: &[i32] = vec.get_data_slice();
    assert_eq!(slice, &[10, 20, 30, 40]);
}

#[test]
fn test_vector_empty() {
    // Edge case: zero-length vector
    let vec = Vector::new(LogicalType::Int32, 0);
    assert_eq!(vec.count(), 0, "zero-capacity vector must report count 0");
}
