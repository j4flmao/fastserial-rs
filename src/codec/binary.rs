//! # Binary Format
//!
//! This module provides a compact, schema-validated binary format for fastserial.
//!
//! ## Format Specification
//!
//! The binary format uses a 16-byte header followed by encoded data:
//!
//! | Bytes | Name | Description |
//! |-------|------|-------------|
//! | 0-3 | Magic | Fixed string `FBIN` (0x4642494E) |
//! | 4-5 | Version | Format version (0x0001) |
//! | 6-13 | Schema Hash | 64-bit FNV-1a hash of the type's schema |
//! | 14-15 | Reserved | Reserved for future use (must be 0) |
//!
//! ## Value Encoding
//!
//! - `null`: 0x00
//! - `bool`: 0x01 (true) or 0x02 (false)
//! - integers: little-endian fixed-size encoding
//! - strings: 4-byte length prefix + UTF-8 data
//! - arrays: 4-byte length prefix + elements
//! - objects: no length prefix (fields are implicit)

use crate::io::{ReadBuffer, WriteBuffer};
use crate::{Encode, Error, Format};

/// Binary format implementation.
///
/// This format provides efficient binary serialization with schema hash validation.
/// Use [`Format`] trait to serialize/deserialize with this format.
pub struct BinaryFormat;

const MAGIC: [u8; 4] = *b"FBIN";
const VERSION: u16 = 0x0001;

impl Format for BinaryFormat {
    /// Returns "binary".
    fn name(&self) -> &'static str {
        "binary"
    }

    /// Encodes a value with the 16-byte header.
    fn encode_struct<T: Encode + ?Sized, W: WriteBuffer>(val: &T, w: &mut W) -> Result<(), Error> {
        w.write_bytes(&MAGIC)?;
        w.write_bytes(&VERSION.to_le_bytes())?;
        w.write_bytes(&T::SCHEMA_HASH.to_le_bytes())?;
        w.write_bytes(&[0, 0])?;
        val.encode(w)
    }

    /// Writes null as a single 0x00 byte.
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0)
    }

    /// Writes bool as 0x01 (true) or 0x02 (false).
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(if v { 1 } else { 0 })
    }

    /// Writes u64 as 8 bytes little-endian.
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(&v.to_le_bytes())
    }

    /// Writes i64 as 8 bytes little-endian.
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(&v.to_le_bytes())
    }

    /// Writes f64 as 8 bytes little-endian IEEE 754.
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(&v.to_le_bytes())
    }

    /// Writes string with 4-byte length prefix + UTF-8 data.
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = v.len() as u32;
        w.write_bytes(&len.to_le_bytes())?;
        w.write_bytes(v.as_bytes())
    }

    /// Writes bytes with 4-byte length prefix.
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = v.len() as u32;
        w.write_bytes(&len.to_le_bytes())?;
        w.write_bytes(v)
    }

    /// Objects have no delimiter in binary format.
    fn begin_object(_n_fields: usize, _w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// Field keys are not written in binary format.
    fn write_field_key(_key: &[u8], _w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// No separator needed in binary format.
    fn field_separator(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// Objects have no delimiter in binary format.
    fn end_object(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// Writes array length as 4-byte little-endian.
    fn begin_array(len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = len as u32;
        w.write_bytes(&len.to_le_bytes())
    }

    /// No separator needed in binary format.
    fn array_separator(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// No delimiter needed in binary format.
    fn end_array(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    /// Reads bool from non-zero byte.
    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        let b = r.next_byte()?;
        Ok(b != 0)
    }

    /// Reads u64 from 8 bytes little-endian.
    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        let bytes = r.peek_slice(8);
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        let v = u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        r.advance(8);
        Ok(v)
    }

    /// Reads i64 from 8 bytes little-endian.
    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        let bytes = r.peek_slice(8);
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        let v = i64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        r.advance(8);
        Ok(v)
    }

    /// Reads f64 from 8 bytes little-endian IEEE 754.
    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        let bytes = r.peek_slice(8);
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        let v = f64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]);
        r.advance(8);
        Ok(v)
    }

    /// Reads string from 4-byte length + UTF-8 data.
    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        let len_bytes = r.peek_slice(4);
        if len_bytes.len() < 4 {
            return Err(Error::UnexpectedEof);
        }
        let len =
            u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
        r.advance(4);

        let data = r.peek_slice(len);
        if data.len() < len {
            return Err(Error::UnexpectedEof);
        }
        let s =
            core::str::from_utf8(data).map_err(|_| Error::InvalidUtf8 { byte_offset: r.pos })?;
        r.advance(len);
        Ok(s)
    }

    /// Reads bytes from 4-byte length + data.
    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        let len_bytes = r.peek_slice(4);
        if len_bytes.len() < 4 {
            return Err(Error::UnexpectedEof);
        }
        let len =
            u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
        r.advance(4);

        let data = r.peek_slice(len);
        if data.len() < len {
            return Err(Error::UnexpectedEof);
        }
        r.advance(len);
        Ok(data)
    }

    /// Reads null (expects 0x00 byte).
    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        let b = r.next_byte()?;
        if b == 0 {
            Ok(())
        } else {
            Err(Error::UnexpectedByte {
                expected: "null (0x00)",
                got: b,
                offset: r.pos - 1,
            })
        }
    }

    /// Binary objects have no header.
    fn begin_object_decode(_r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        Ok(0)
    }

    /// Reads field key as string.
    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        Self::read_str(r)
    }

    /// Binary objects have no delimiter.
    fn end_object_decode(_r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        Ok(())
    }

    /// Reads array length from 4 bytes.
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        let len_bytes = r.peek_slice(4);
        if len_bytes.len() < 4 {
            return Err(Error::UnexpectedEof);
        }
        let len =
            u32::from_le_bytes([len_bytes[0], len_bytes[1], len_bytes[2], len_bytes[3]]) as usize;
        r.advance(4);
        Ok(len)
    }

    /// Binary arrays have no delimiter.
    fn end_array_decode(_r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        Ok(())
    }

    /// Skips a binary value based on its type byte.
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        match r.peek() {
            0 => {
                r.advance(1);
                Ok(())
            }
            1 | 2 => {
                r.advance(1);
                Ok(())
            }
            b if b < 0x80 => {
                r.advance(1);
                Ok(())
            }
            b if (0x80..0xC0).contains(&b) => {
                let len = (b & 0x3F) as usize;
                r.advance(1 + len);
                Ok(())
            }
            _ => Err(Error::UnexpectedByte {
                expected: "binary value",
                got: r.peek(),
                offset: r.pos,
            }),
        }
    }
}

/// Writes null as a single 0x00 byte.
#[inline]
pub fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_null(w)
}

/// Writes bool as 0x01 (true) or 0x02 (false).
#[inline]
pub fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_bool(v, w)
}

/// Writes u8 as a single byte.
#[inline]
pub fn write_u8(v: u8, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(v)
}

/// Writes u16 as 2 bytes little-endian.
#[inline]
pub fn write_u16(v: u16, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(&v.to_le_bytes())
}

/// Writes u32 as 4 bytes little-endian.
#[inline]
pub fn write_u32(v: u32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(&v.to_le_bytes())
}

/// Writes u64 as 8 bytes little-endian.
#[inline]
pub fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_u64(v, w)
}

/// Writes i8 as a single byte.
#[inline]
pub fn write_i8(v: i8, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(v as u8)
}

/// Writes i16 as 2 bytes little-endian.
#[inline]
pub fn write_i16(v: i16, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(&v.to_le_bytes())
}

/// Writes i32 as 4 bytes little-endian.
#[inline]
pub fn write_i32(v: i32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(&v.to_le_bytes())
}

/// Writes i64 as 8 bytes little-endian.
#[inline]
pub fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_i64(v, w)
}

/// Writes f32 as 4 bytes little-endian IEEE 754.
#[inline]
pub fn write_f32(v: f32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(&v.to_le_bytes())
}

/// Writes f64 as 8 bytes little-endian IEEE 754.
#[inline]
pub fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_f64(v, w)
}

/// Writes string with 4-byte length prefix + UTF-8 data.
#[inline]
pub fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_str(v, w)
}

/// Writes bytes with 4-byte length prefix.
#[inline]
pub fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
    BinaryFormat::write_bytes(v, w)
}

/// Reads null (expects 0x00 byte).
#[inline]
pub fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    BinaryFormat::read_null(r)
}

/// Reads bool from non-zero byte.
#[inline]
pub fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
    BinaryFormat::read_bool(r)
}

/// Reads u8 from a single byte.
#[inline]
pub fn read_u8(r: &mut ReadBuffer<'_>) -> Result<u8, Error> {
    r.next_byte()
}

/// Reads u16 from 2 bytes little-endian.
#[inline]
pub fn read_u16(r: &mut ReadBuffer<'_>) -> Result<u16, Error> {
    let bytes = r.peek_slice(2);
    if bytes.len() < 2 {
        return Err(Error::UnexpectedEof);
    }
    let v = u16::from_le_bytes([bytes[0], bytes[1]]);
    r.advance(2);
    Ok(v)
}

/// Reads u32 from 4 bytes little-endian.
#[inline]
pub fn read_u32(r: &mut ReadBuffer<'_>) -> Result<u32, Error> {
    let bytes = r.peek_slice(4);
    if bytes.len() < 4 {
        return Err(Error::UnexpectedEof);
    }
    let v = u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    r.advance(4);
    Ok(v)
}

/// Reads u64 from 8 bytes little-endian.
#[inline]
pub fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
    BinaryFormat::read_u64(r)
}

/// Reads i8 from a single byte.
#[inline]
pub fn read_i8(r: &mut ReadBuffer<'_>) -> Result<i8, Error> {
    r.next_byte().map(|b| b as i8)
}

/// Reads i16 from 2 bytes little-endian.
#[inline]
pub fn read_i16(r: &mut ReadBuffer<'_>) -> Result<i16, Error> {
    let bytes = r.peek_slice(2);
    if bytes.len() < 2 {
        return Err(Error::UnexpectedEof);
    }
    let v = i16::from_le_bytes([bytes[0], bytes[1]]);
    r.advance(2);
    Ok(v)
}

/// Reads i32 from 4 bytes little-endian.
#[inline]
pub fn read_i32(r: &mut ReadBuffer<'_>) -> Result<i32, Error> {
    let bytes = r.peek_slice(4);
    if bytes.len() < 4 {
        return Err(Error::UnexpectedEof);
    }
    let v = i32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    r.advance(4);
    Ok(v)
}

/// Reads i64 from 8 bytes little-endian.
#[inline]
pub fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
    BinaryFormat::read_i64(r)
}

/// Reads f32 from 4 bytes little-endian IEEE 754.
#[inline]
pub fn read_f32(r: &mut ReadBuffer<'_>) -> Result<f32, Error> {
    let bytes = r.peek_slice(4);
    if bytes.len() < 4 {
        return Err(Error::UnexpectedEof);
    }
    let v = f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
    r.advance(4);
    Ok(v)
}

/// Reads f64 from 8 bytes little-endian IEEE 754.
#[inline]
pub fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
    BinaryFormat::read_f64(r)
}

/// Reads string from 4-byte length + UTF-8 data.
#[inline]
pub fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
    BinaryFormat::read_str(r)
}

/// Reads bytes from 4-byte length + data.
#[inline]
pub fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    BinaryFormat::read_bytes(r)
}
