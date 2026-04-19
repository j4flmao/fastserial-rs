//! # Schema Types
//!
//! This module defines types for schema representation and validation.

use alloc::boxed::Box;

/// Represents a field type in a schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Boolean type.
    Bool,
    /// Unsigned 8-bit integer.
    U8,
    /// Unsigned 16-bit integer.
    U16,
    /// Unsigned 32-bit integer.
    U32,
    /// Unsigned 64-bit integer.
    U64,
    /// Signed 8-bit integer.
    I8,
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// Signed 64-bit integer.
    I64,
    /// 32-bit floating point.
    F32,
    /// 64-bit floating point.
    F64,
    /// UTF-8 string.
    String,
    /// Byte sequence.
    Bytes,
    /// Optional type (may be null).
    Option(Box<FieldType>),
    /// Array of type.
    Vec(Box<FieldType>),
    /// Nested struct by name.
    Struct(&'static str),
}

impl FieldType {
    /// Returns true if this is a scalar type (no indirection).
    pub fn is_scalar(&self) -> bool {
        matches!(
            self,
            Self::Bool
                | Self::U8
                | Self::U16
                | Self::U32
                | Self::U64
                | Self::I8
                | Self::I16
                | Self::I32
                | Self::I64
                | Self::F32
                | Self::F64
        )
    }

    /// Returns the required alignment for this type.
    pub fn alignment(&self) -> usize {
        match self {
            Self::Bool | Self::U8 | Self::I8 => 1,
            Self::U16 | Self::I16 => 2,
            Self::U32 | Self::I32 | Self::F32 => 4,
            Self::U64 | Self::I64 | Self::F64 => 8,
            Self::String | Self::Bytes | Self::Option(_) | Self::Vec(_) | Self::Struct(_) => 8,
        }
    }

    /// Returns the fixed size if known, or None for dynamic types.
    pub fn size(&self) -> Option<usize> {
        match self {
            Self::Bool | Self::U8 | Self::I8 => Some(1),
            Self::U16 | Self::I16 => Some(2),
            Self::U32 | Self::I32 | Self::F32 => Some(4),
            Self::U64 | Self::I64 | Self::F64 => Some(8),
            Self::String | Self::Bytes | Self::Option(_) | Self::Vec(_) | Self::Struct(_) => None,
        }
    }
}
