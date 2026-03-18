//! Lesson 02: Data Types & Type System Tests

use quackdb::types::{LogicalType, PhysicalType, ScalarValue};

#[test]
fn test_logical_to_physical_mapping() {
    assert_eq!(LogicalType::Boolean.physical_type(), PhysicalType::Bool);
    assert_eq!(LogicalType::Int8.physical_type(), PhysicalType::Int8);
    assert_eq!(LogicalType::Int16.physical_type(), PhysicalType::Int16);
    assert_eq!(LogicalType::Int32.physical_type(), PhysicalType::Int32);
    assert_eq!(LogicalType::Int64.physical_type(), PhysicalType::Int64);
    assert_eq!(LogicalType::Float32.physical_type(), PhysicalType::Float32);
    assert_eq!(LogicalType::Float64.physical_type(), PhysicalType::Float64);
    assert_eq!(LogicalType::Varchar.physical_type(), PhysicalType::Varchar);
}

#[test]
fn test_byte_widths() {
    assert_eq!(LogicalType::Boolean.byte_width(), Some(1));
    assert_eq!(LogicalType::Int8.byte_width(), Some(1));
    assert_eq!(LogicalType::Int16.byte_width(), Some(2));
    assert_eq!(LogicalType::Int32.byte_width(), Some(4));
    assert_eq!(LogicalType::Int64.byte_width(), Some(8));
    assert_eq!(LogicalType::UInt8.byte_width(), Some(1));
    assert_eq!(LogicalType::UInt16.byte_width(), Some(2));
    assert_eq!(LogicalType::UInt32.byte_width(), Some(4));
    assert_eq!(LogicalType::UInt64.byte_width(), Some(8));
    assert_eq!(LogicalType::Float32.byte_width(), Some(4));
    assert_eq!(LogicalType::Float64.byte_width(), Some(8));
    assert_eq!(LogicalType::Date.byte_width(), Some(4));
    assert_eq!(LogicalType::Timestamp.byte_width(), Some(8));
    assert_eq!(LogicalType::Varchar.byte_width(), None, "variable-length types must return None for byte_width");
    assert_eq!(LogicalType::Blob.byte_width(), None, "variable-length types must return None for byte_width");
}

#[test]
fn test_type_categories() {
    assert!(LogicalType::Int32.is_numeric());
    assert!(LogicalType::Float64.is_numeric());
    assert!(LogicalType::UInt16.is_numeric());
    assert!(!LogicalType::Varchar.is_numeric(), "Varchar is not a numeric type");
    assert!(!LogicalType::Boolean.is_numeric(), "Boolean is not numeric despite being stored as a single byte");

    assert!(LogicalType::Int32.is_integer());
    assert!(LogicalType::UInt64.is_integer());
    assert!(!LogicalType::Float32.is_integer(), "floats must not be classified as integers");

    assert!(LogicalType::Float32.is_float());
    assert!(LogicalType::Float64.is_float());
    assert!(!LogicalType::Int32.is_float());
}

#[test]
fn test_scalar_value_types() {
    assert_eq!(ScalarValue::Boolean(true).logical_type(), LogicalType::Boolean);
    assert_eq!(ScalarValue::Int32(42).logical_type(), LogicalType::Int32);
    assert_eq!(ScalarValue::Int64(100).logical_type(), LogicalType::Int64);
    assert_eq!(ScalarValue::Float64(3.14).logical_type(), LogicalType::Float64);
    assert_eq!(ScalarValue::Varchar("hello".into()).logical_type(), LogicalType::Varchar);
    assert_eq!(ScalarValue::Null(LogicalType::Int32).logical_type(), LogicalType::Int32, "NULL values must carry their logical type for schema consistency");
}

#[test]
fn test_scalar_value_is_null() {
    assert!(ScalarValue::Null(LogicalType::Int32).is_null());
    assert!(!ScalarValue::Int32(0).is_null(), "zero is a valid value, not NULL — NULL represents missing data");
    assert!(!ScalarValue::Boolean(false).is_null(), "false is a valid value, not NULL");
}

#[test]
fn test_scalar_roundtrip() {
    let values = vec![
        ScalarValue::Boolean(true),
        ScalarValue::Int8(42),
        ScalarValue::Int16(-100),
        ScalarValue::Int32(123456),
        ScalarValue::Int64(i64::MAX),
        ScalarValue::UInt8(255),
        ScalarValue::UInt32(1000),
        ScalarValue::Float32(3.14),
        ScalarValue::Float64(2.71828),
    ];

    for val in &values {
        let bytes = val.to_bytes();
        let decoded = ScalarValue::from_bytes(&bytes, &val.logical_type()).unwrap();
        assert_eq!(&decoded, val, "Roundtrip failed for {:?}", val);
    }
}

#[test]
fn test_scalar_varchar_roundtrip() {
    let val = ScalarValue::Varchar("hello world".into());
    let bytes = val.to_bytes();
    let decoded = ScalarValue::from_bytes(&bytes, &LogicalType::Varchar).unwrap();
    assert_eq!(decoded, val, "variable-length types must survive serialization roundtrip");
}

#[test]
fn test_type_coercion() {
    // Int32 + Int64 -> Int64
    let result = LogicalType::coerce(&LogicalType::Int32, &LogicalType::Int64);
    assert_eq!(result, Some(LogicalType::Int64), "coercion must widen to the larger integer type");

    // Int32 + Float64 -> Float64
    let result = LogicalType::coerce(&LogicalType::Int32, &LogicalType::Float64);
    assert_eq!(result, Some(LogicalType::Float64), "integer-float coercion must promote to float");

    // Float32 + Float64 -> Float64
    let result = LogicalType::coerce(&LogicalType::Float32, &LogicalType::Float64);
    assert_eq!(result, Some(LogicalType::Float64));

    // Same types
    let result = LogicalType::coerce(&LogicalType::Int32, &LogicalType::Int32);
    assert_eq!(result, Some(LogicalType::Int32));

    // Incompatible types
    let result = LogicalType::coerce(&LogicalType::Boolean, &LogicalType::Varchar);
    assert_eq!(result, None, "incompatible types must return None rather than silently coerce");
}

#[test]
fn test_can_cast() {
    assert!(LogicalType::can_cast(&LogicalType::Int32, &LogicalType::Int64));
    assert!(LogicalType::can_cast(&LogicalType::Int32, &LogicalType::Float64));
    assert!(LogicalType::can_cast(&LogicalType::Int64, &LogicalType::Varchar), "any numeric type should be castable to string representation");
    assert!(LogicalType::can_cast(&LogicalType::Float64, &LogicalType::Varchar), "any numeric type should be castable to string representation");
}

#[test]
fn test_scalar_cast() {
    let val = ScalarValue::Int32(42);
    let casted = val.cast_to(&LogicalType::Int64).unwrap();
    assert_eq!(casted, ScalarValue::Int64(42), "widening cast must preserve the numeric value");

    let val = ScalarValue::Int32(42);
    let casted = val.cast_to(&LogicalType::Float64).unwrap();
    assert_eq!(casted, ScalarValue::Float64(42.0), "int-to-float cast must preserve the numeric value");
}

#[test]
fn test_decimal_type() {
    let decimal = LogicalType::Decimal { precision: 10, scale: 2 };
    assert!(decimal.is_numeric(), "Decimal is a numeric type despite having precision/scale metadata");
    assert!(decimal.byte_width().is_some(), "Decimal has a fixed in-memory width for columnar storage");
}

#[test]
fn test_logical_type_display() {
    assert_eq!(format!("{}", LogicalType::Int32), "INT32");
    assert_eq!(format!("{}", LogicalType::Varchar), "VARCHAR");
    assert_eq!(format!("{}", LogicalType::Boolean), "BOOLEAN");
    let d = LogicalType::Decimal { precision: 10, scale: 2 };
    let display = format!("{}", d);
    assert!(display.contains("DECIMAL") || display.contains("10") || display.contains("2"));
}

#[test]
fn test_scalar_display() {
    assert_eq!(format!("{}", ScalarValue::Int32(42)), "42");
    assert_eq!(format!("{}", ScalarValue::Boolean(true)), "true");
    assert_eq!(format!("{}", ScalarValue::Varchar("hello".into())), "hello");
    let null_display = format!("{}", ScalarValue::Null(LogicalType::Int32));
    assert!(null_display.contains("NULL") || null_display.contains("null"));
}
