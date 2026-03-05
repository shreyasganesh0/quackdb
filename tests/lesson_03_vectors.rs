//! Lesson 03: Columnar Vectors Tests

use quackdb::types::{LogicalType, ScalarValue};
use quackdb::vector::{ValidityMask, Vector, VectorType, SelectionVector};

#[test]
fn test_validity_mask_all_valid() {
    let mask = ValidityMask::new_all_valid(100);
    assert!(mask.all_valid());
    assert_eq!(mask.count_valid(), 100);
    for i in 0..100 {
        assert!(mask.is_valid(i));
    }
}

#[test]
fn test_validity_mask_all_invalid() {
    let mask = ValidityMask::new_all_invalid(100);
    assert!(!mask.all_valid());
    assert_eq!(mask.count_valid(), 0);
    for i in 0..100 {
        assert!(!mask.is_valid(i));
    }
}

#[test]
fn test_validity_mask_set() {
    let mut mask = ValidityMask::new_all_valid(64);
    mask.set_valid(10, false);
    assert!(!mask.is_valid(10));
    assert!(mask.is_valid(9));
    assert!(mask.is_valid(11));
    assert_eq!(mask.count_valid(), 63);

    mask.set_valid(10, true);
    assert!(mask.is_valid(10));
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
    assert!(!mask.is_valid(25));
    // New entries should be valid
    for i in 50..100 {
        assert!(mask.is_valid(i));
    }
}

#[test]
fn test_vector_flat_int32() {
    let mut vec = Vector::new(LogicalType::Int32, 10);
    vec.set_count(10);
    for i in 0..10 {
        vec.set_value(i, ScalarValue::Int32(i as i32 * 10));
    }
    for i in 0..10 {
        let val = vec.get_value(i);
        assert_eq!(val, ScalarValue::Int32(i as i32 * 10));
    }
    assert_eq!(vec.vector_type(), VectorType::Flat);
    assert_eq!(vec.count(), 10);
}

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
    assert!(!vec.validity().is_valid(2));
    assert!(vec.validity().is_valid(3));
    assert!(!vec.validity().is_valid(4));

    assert_eq!(vec.get_value(0), ScalarValue::Int64(100));
    assert_eq!(vec.get_value(1), ScalarValue::Int64(200));
    assert_eq!(vec.get_value(3), ScalarValue::Int64(400));
}

#[test]
fn test_vector_constant() {
    let vec = Vector::new_constant(ScalarValue::Int32(42), 100);
    assert_eq!(vec.vector_type(), VectorType::Constant);
    assert_eq!(vec.count(), 100);
    assert_eq!(vec.get_value(0), ScalarValue::Int32(42));
    assert_eq!(vec.get_value(50), ScalarValue::Int32(42));
    assert_eq!(vec.get_value(99), ScalarValue::Int32(42));
}

#[test]
fn test_vector_constant_null() {
    let vec = Vector::new_constant(ScalarValue::Null(LogicalType::Int32), 50);
    assert_eq!(vec.vector_type(), VectorType::Constant);
    for i in 0..50 {
        assert!(!vec.validity().is_valid(0));
    }
}

#[test]
fn test_vector_flatten() {
    let mut vec = Vector::new_constant(ScalarValue::Int32(7), 5);
    assert_eq!(vec.vector_type(), VectorType::Constant);
    vec.flatten();
    assert_eq!(vec.vector_type(), VectorType::Flat);
    for i in 0..5 {
        assert_eq!(vec.get_value(i), ScalarValue::Int32(7));
    }
}

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
fn test_vector_copy_with_selection() {
    let mut src = Vector::new(LogicalType::Int32, 10);
    src.set_count(10);
    for i in 0..10 {
        src.set_value(i, ScalarValue::Int32(i as i32 * 100));
    }

    let sel = SelectionVector::new(vec![1, 3, 7]);
    let mut dst = Vector::new(LogicalType::Int32, 3);
    src.copy_with_selection(&sel, &mut dst);

    assert_eq!(dst.count(), 3);
    assert_eq!(dst.get_value(0), ScalarValue::Int32(100));
    assert_eq!(dst.get_value(1), ScalarValue::Int32(300));
    assert_eq!(dst.get_value(2), ScalarValue::Int32(700));
}

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

    assert_eq!(vec.get_string(0), Some(""));
    assert_eq!(vec.get_string(1), Some("notempty"));
}

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
