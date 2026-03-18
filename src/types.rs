//! Lesson 02: Data Types & Type System
//!
//! Core type system for QuackDB: logical types (user-facing SQL types),
//! physical types (in-memory representation), and scalar values (individual
//! constants/literals).
//!
//! Key Rust concepts: enums with data, `match` exhaustiveness, `Display` trait
//! implementation, and `From`/`Into` style conversions.

use std::fmt;

/// Logical data types supported by QuackDB.
///
/// These correspond to SQL-level types. Each logical type maps to a
/// `PhysicalType` that determines how values are stored in memory.
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
    /// Fixed-point decimal with configurable precision and scale.
    Decimal { precision: u8, scale: u8 },
    Blob,
}

/// Physical representation info -- how types are stored in memory.
///
/// Fixed-width types store values inline; variable-width types (like `Varchar`)
/// use an offset/length indirection into a separate data buffer.
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
    /// Fixed-size byte array of given width (e.g. for Decimal).
    FixedSize(usize),
}

impl LogicalType {
    /// Get the physical type corresponding to this logical type.
    ///
    // Hint: a straightforward `match` on `self`. Decimal maps to
    // `FixedSize(16)` (i128); Blob and Varchar map to `PhysicalType::Varchar`.
    pub fn physical_type(&self) -> PhysicalType {
        match self {
            LogicalType::Boolean => PhysicalType::Bool,
            LogicalType::Int8 => PhysicalType::Int8,
            LogicalType::Int16 => PhysicalType::Int16,
            LogicalType::Int32 => PhysicalType::Int32,
            LogicalType::Int64 => PhysicalType::Int64,
            LogicalType::UInt8 => PhysicalType::Int8,
            LogicalType::UInt16 => PhysicalType::Int16,
            LogicalType::UInt32 => PhysicalType::Int32,
            LogicalType::UInt64 => PhysicalType::Int64,
            LogicalType::Float32 => PhysicalType::Float32,
            LogicalType::Float64 => PhysicalType::Float64,
            LogicalType::Varchar => PhysicalType::Varchar,
            LogicalType::Date => PhysicalType::Int32,
            LogicalType::Timestamp => PhysicalType::Int64,
            LogicalType::Decimal { .. } => PhysicalType::FixedSize(16),
            LogicalType::Blob => PhysicalType::Varchar,
        }
    }

    /// Get the byte width of this type's physical representation.
    ///
    /// Returns `None` for variable-length types (Varchar, Blob).
    // Hint: match on physical_type(); Varchar => None, FixedSize(n) => Some(n), etc.
    pub fn byte_width(&self) -> Option<usize> {
        match self.physical_type() {
            PhysicalType::Bool => Some(1),
            PhysicalType::Int8 => Some(1),
            PhysicalType::Int16 => Some(2),
            PhysicalType::Int32 => Some(4),
            PhysicalType::Int64 => Some(8),
            PhysicalType::Float32 => Some(4),
            PhysicalType::Float64 => Some(8),
            PhysicalType::Varchar => None,
            PhysicalType::FixedSize(n) => Some(n),
        }
    }

    /// Check if this type is a numeric type (integers, floats, or decimal).
    pub fn is_numeric(&self) -> bool {
        self.is_integer() || self.is_float() || matches!(self, LogicalType::Decimal { .. })
    }

    /// Check if this type is an integer type (signed or unsigned).
    pub fn is_integer(&self) -> bool {
        matches!(
            self,
            LogicalType::Int8
                | LogicalType::Int16
                | LogicalType::Int32
                | LogicalType::Int64
                | LogicalType::UInt8
                | LogicalType::UInt16
                | LogicalType::UInt32
                | LogicalType::UInt64
        )
    }

    /// Check if this type is a floating-point type.
    pub fn is_float(&self) -> bool {
        matches!(self, LogicalType::Float32 | LogicalType::Float64)
    }

    /// Determine the result type of a coercion between two types.
    ///
    /// Returns `None` if the types cannot be coerced. When both types are
    /// numeric, the result is the wider/more-precise type.
    // Hint: if left == right, return that type. Otherwise, define a
    // numeric type-promotion hierarchy (Int8 < Int16 < ... < Float64).
    pub fn coerce(left: &LogicalType, right: &LogicalType) -> Option<LogicalType> {
        todo!()
    }

    /// Check if a value of type `from` can be cast to type `to`.
    // Hint: numeric-to-numeric is always allowed; string-to-numeric
    // and numeric-to-string are allowed; some combinations are not.
    pub fn can_cast(from: &LogicalType, to: &LogicalType) -> bool {
        todo!()
    }
}

// Implement Display so LogicalType can be printed in error messages and EXPLAIN output.
impl fmt::Display for LogicalType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LogicalType::Boolean => write!(f, "BOOLEAN"),
            LogicalType::Int8 => write!(f, "TINYINT"),
            LogicalType::Int16 => write!(f, "SMALLINT"),
            LogicalType::Int32 => write!(f, "INTEGER"),
            LogicalType::Int64 => write!(f, "BIGINT"),
            LogicalType::UInt8 => write!(f, "UTINYINT"),
            LogicalType::UInt16 => write!(f, "USMALLINT"),
            LogicalType::UInt32 => write!(f, "UINTEGER"),
            LogicalType::UInt64 => write!(f, "UBIGINT"),
            LogicalType::Float32 => write!(f, "FLOAT"),
            LogicalType::Float64 => write!(f, "DOUBLE"),
            LogicalType::Varchar => write!(f, "VARCHAR"),
            LogicalType::Date => write!(f, "DATE"),
            LogicalType::Timestamp => write!(f, "TIMESTAMP"),
            LogicalType::Decimal { precision, scale } => {
                write!(f, "DECIMAL({},{})", precision, scale)
            }
            LogicalType::Blob => write!(f, "BLOB"),
        }
    }
}

/// A single scalar value, used for constants, literals, and aggregation results.
///
/// Each variant carries both the value and (implicitly) its type.
/// `Null` carries a `LogicalType` so the system knows what type a NULL is.
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
    // Hint: match each variant to its corresponding LogicalType.
    pub fn logical_type(&self) -> LogicalType {
        match self {
            ScalarValue::Null(t) => t.clone(),
            ScalarValue::Boolean(_) => LogicalType::Boolean,
            ScalarValue::Int8(_) => LogicalType::Int8,
            ScalarValue::Int16(_) => LogicalType::Int16,
            ScalarValue::Int32(_) => LogicalType::Int32,
            ScalarValue::Int64(_) => LogicalType::Int64,
            ScalarValue::UInt8(_) => LogicalType::UInt8,
            ScalarValue::UInt16(_) => LogicalType::UInt16,
            ScalarValue::UInt32(_) => LogicalType::UInt32,
            ScalarValue::UInt64(_) => LogicalType::UInt64,
            ScalarValue::Float32(_) => LogicalType::Float32,
            ScalarValue::Float64(_) => LogicalType::Float64,
            ScalarValue::Varchar(_) => LogicalType::Varchar,
            ScalarValue::Date(_) => LogicalType::Date,
            ScalarValue::Timestamp(_) => LogicalType::Timestamp,
            ScalarValue::Decimal { precision, scale, .. } => LogicalType::Decimal {
                precision: *precision,
                scale: *scale,
            },
            ScalarValue::Blob(_) => LogicalType::Blob,
        }
    }

    /// Try to cast this scalar value to a different type.
    ///
    /// Returns `Err` with a description if the cast is not supported.
    // Hint: first check `LogicalType::can_cast`, then perform the
    // numeric conversion (e.g. `as` casts or `TryFrom`).
    pub fn cast_to(&self, target: &LogicalType) -> Result<ScalarValue, String> {
        todo!()
    }

    /// Check if this value is null.
    pub fn is_null(&self) -> bool {
        matches!(self, ScalarValue::Null(_))
    }

    /// Encode this scalar value to bytes (little-endian for numerics).
    // Hint: use `to_le_bytes()` for integer/float types.
    pub fn to_bytes(&self) -> Vec<u8> {
        todo!()
    }

    /// Decode a scalar value from bytes given a type.
    // Hint: use `from_le_bytes` and match on the logical_type to pick the variant.
    pub fn from_bytes(bytes: &[u8], logical_type: &LogicalType) -> Result<ScalarValue, String> {
        todo!()
    }
}

impl fmt::Display for ScalarValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScalarValue::Null(_) => write!(f, "NULL"),
            ScalarValue::Boolean(v) => write!(f, "{}", v),
            ScalarValue::Int8(v) => write!(f, "{}", v),
            ScalarValue::Int16(v) => write!(f, "{}", v),
            ScalarValue::Int32(v) => write!(f, "{}", v),
            ScalarValue::Int64(v) => write!(f, "{}", v),
            ScalarValue::UInt8(v) => write!(f, "{}", v),
            ScalarValue::UInt16(v) => write!(f, "{}", v),
            ScalarValue::UInt32(v) => write!(f, "{}", v),
            ScalarValue::UInt64(v) => write!(f, "{}", v),
            ScalarValue::Float32(v) => write!(f, "{}", v),
            ScalarValue::Float64(v) => write!(f, "{}", v),
            ScalarValue::Varchar(v) => write!(f, "{}", v),
            ScalarValue::Date(v) => write!(f, "DATE({})", v),
            ScalarValue::Timestamp(v) => write!(f, "TIMESTAMP({})", v),
            ScalarValue::Decimal { value, precision, scale } => {
                write!(f, "DECIMAL({},{},{})", value, precision, scale)
            }
            ScalarValue::Blob(v) => write!(f, "BLOB[{} bytes]", v.len()),
        }
    }
}
