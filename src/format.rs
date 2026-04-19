//! # Format Trait
//!
//! This module defines the [`Format`] trait for implementing custom serialization formats.

use crate::Error;
use crate::io::{ReadBuffer, WriteBuffer};

/// Trait for implementing custom serialization formats.
///
/// This trait provides a unified interface for encoding and decoding values
/// in different formats (JSON, Binary, MsgPack, etc.).
///
/// # Implementing a Format
///
/// To implement a custom format, you need to provide methods for:
/// - Writing primitives (null, bool, numbers, strings)
/// - Beginning/ending objects and arrays
/// - Reading primitives
/// - Beginning/ending objects and arrays during decode
pub trait Format: Send + Sync {
    /// Returns the format name.
    fn name(&self) -> &'static str;

    /// Encodes a struct with format-specific header.
    fn encode_struct<T: crate::Encode + ?Sized, W: WriteBuffer>(
        val: &T,
        w: &mut W,
    ) -> Result<(), Error>;

    /// Writes null value.
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes boolean value.
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes unsigned 64-bit integer.
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes signed 64-bit integer.
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes 64-bit floating point number.
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes string value.
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes byte slice value.
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Begins encoding an object.
    fn begin_object(n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes object field key.
    fn write_field_key(key: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes separator between object fields.
    fn field_separator(w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Ends encoding an object.
    fn end_object(w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Begins encoding an array.
    fn begin_array(len: usize, w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Writes separator between array elements.
    fn array_separator(w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Ends encoding an array.
    fn end_array(w: &mut impl WriteBuffer) -> Result<(), Error>;

    /// Reads boolean value.
    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error>;

    /// Reads unsigned 64-bit integer.
    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error>;

    /// Reads signed 64-bit integer.
    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error>;

    /// Reads 64-bit floating point number.
    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error>;

    /// Reads string value.
    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error>;

    /// Reads byte slice value.
    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error>;

    /// Reads null value.
    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error>;

    /// Begins decoding an object.
    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error>;

    /// Reads object field key.
    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error>;

    /// Ends decoding an object.
    fn end_object_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error>;

    /// Begins decoding an array.
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error>;

    /// Ends decoding an array.
    fn end_array_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error>;

    /// Skips a value during decoding.
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error>;
}

/// JSON format implementation.
///
/// This format writes values as JSON text (RFC 8259).
#[allow(dead_code)]
pub struct JsonFormat;

impl Format for JsonFormat {
    #[inline(always)]
    fn name(&self) -> &'static str {
        "json"
    }

    #[inline(always)]
    fn encode_struct<T: crate::Encode + ?Sized, W: WriteBuffer>(
        val: &T,
        w: &mut W,
    ) -> Result<(), Error> {
        val.encode(w)
    }

    #[inline(always)]
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(b"null")
    }

    #[inline(always)]
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(if v { b"true" } else { b"false" })
    }

    #[inline(always)]
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        crate::codec::write_u64(v, w)
    }

    #[inline(always)]
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        crate::codec::write_i64(v, w)
    }

    #[inline(always)]
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        crate::codec::write_f64(v, w)
    }

    #[inline(always)]
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        crate::codec::write_str(v, w)
    }

    #[inline(always)]
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        crate::codec::write_bytes(v, w)
    }

    #[inline(always)]
    fn begin_object(_n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'{')
    }

    #[inline(always)]
    fn write_field_key(key: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'"')?;
        w.write_bytes(key)?;
        w.write_bytes(b"\":")
    }

    #[inline(always)]
    fn field_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b',')
    }

    #[inline(always)]
    fn end_object(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'}')
    }

    #[inline(always)]
    fn begin_array(_len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'[')
    }

    #[inline(always)]
    fn array_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b',')
    }

    #[inline(always)]
    fn end_array(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b']')
    }

    #[inline(always)]
    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        crate::codec::read_bool(r)
    }

    #[inline(always)]
    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        crate::codec::read_u64(r)
    }

    #[inline(always)]
    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        crate::codec::read_i64(r)
    }

    #[inline(always)]
    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        crate::codec::read_f64(r)
    }

    #[inline(always)]
    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        crate::codec::read_string(r)
    }

    #[inline(always)]
    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        crate::codec::read_bytes_impl(r)
    }

    #[inline(always)]
    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        crate::codec::read_null(r)
    }

    #[inline(always)]
    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        crate::codec::json::skip_whitespace(r);
        r.expect_byte(b'{')?;
        Ok(0)
    }

    #[inline(always)]
    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        crate::codec::read_string(r)
    }

    #[inline(always)]
    fn end_object_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        r.expect_byte(b'}')
    }

    #[inline(always)]
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        crate::codec::json::skip_whitespace(r);
        r.expect_byte(b'[')?;
        Ok(0)
    }

    #[inline(always)]
    fn end_array_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        r.expect_byte(b']')
    }

    #[inline(always)]
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        crate::codec::skip_value(r)
    }
}
