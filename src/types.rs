//! Lesson 02: Data Types & Type System
//!
//! Core type system for the database: logical types, physical types, and scalar values.

use std::fmt;

/// Logical data types supported by QuackDB.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogicalType {
    Boolean,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Float32,
    Float64,
    Varchar,
    Date,
    Timestamp,
    Decimal { precision: u8, scale: u8 },
    Blob,
}

/// Physical representation info — how types are stored in memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalType {
    Bool,
    Int8,
    Int16,
    Int32,
    Int64,
    Float32,
    Float64,
    Varchar,
    /// Fixed-size byte array of given width
    FixedSize(usize),
}

impl LogicalType {
    /// Get the physical type corresponding to this logical type.
    pub fn physical_type(&self) -> PhysicalType {
        todo!()
    }

    /// Get the byte width of this type's physical representation.
    /// Returns None for variable-length types (Varchar, Blob).
    pub fn byte_width(&self) -> Option<usize> {
        todo!()
    }

    /// Check if this type is a numeric type.
    pub fn is_numeric(&self) -> bool {
        todo!()
    }

    /// Check if this type is an integer type (signed or unsigned).
    pub fn is_integer(&self) -> bool {
        todo!()
    }

    /// Check if this type is a floating-point type.
    pub fn is_float(&self) -> bool {
        todo!()
    }

    /// Determine the result type of a coercion between two types.
    /// Returns None if the types cannot be coerced.
    pub fn coerce(left: &LogicalType, right: &LogicalType) -> Option<LogicalType> {
        todo!()
    }

    /// Check if a value of type `from` can be cast to type `to`.
    pub fn can_cast(from: &LogicalType, to: &LogicalType) -> bool {
        todo!()
    }
}

impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// A single scalar value, used for constants and literals.
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    Null(LogicalType),
    Boolean(bool),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    UInt8(u8),
    UInt16(u16),
    UInt32(u32),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Varchar(String),
    Date(i32),
    Timestamp(i64),
    Decimal { value: i128, precision: u8, scale: u8 },
    Blob(Vec<u8>),
}

impl ScalarValue {
    /// Get the logical type of this scalar value.
    pub fn logical_type(&self) -> LogicalType {
        todo!()
    }

    /// Try to cast this scalar value to a different type.
    pub fn cast_to(&self, target: &LogicalType) -> Result<ScalarValue, String> {
        todo!()
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, ScalarValue::Null(_))
    }

    /// Encode this scalar value to bytes.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Decode a scalar value from bytes given a type.
    pub fn from_bytes(bytes: &[u8], logical_type: &LogicalType) -> Result<ScalarValue, String> {
        todo!()
    }
}

impl fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
