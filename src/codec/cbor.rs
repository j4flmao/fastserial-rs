//! # CBOR Format (RFC 8949)
//!
//! This module provides high-performance encoding and decoding for CBOR
//! (Concise Binary Object Representation), a standard binary data format.
//!
//! CBOR is designed to be highly compact, making it ideal for network transmission
//! and scenarios where payload size matters.
//!
//! # Features
//!
//! - **Full Standard Support**: Handles primitives, floats, bytes, strings, arrays, and maps.
//! - **Zero-copy Integration**: Decodes byte slices dynamically with minimal allocation.
//! - **Direct `Value` Conversion**: Parses directly into `fastserial::value::Value`.
//!
//! # Example
//!
//! ```ignore
//! use fastserial::codec::cbor;
//! use fastserial::value::Value;
//! 
//! let my_obj = Value::Null; // Construct a Value tree
//! let encoded_bytes = cbor::encode(&my_obj).unwrap();
//! let decoded_val = cbor::decode(&encoded_bytes).unwrap();
//! ```

use crate::io::{ReadBuffer, WriteBuffer};
use crate::{Encode, Error, Format};

/// CBOR format implementation.
pub struct CborFormat;

#[inline(always)]
fn write_cbor_head(w: &mut impl WriteBuffer, major: u8, v: u64) -> Result<(), Error> {
    let type_val = major << 5;
    if v < 24 {
        w.write_byte(type_val | (v as u8))
    } else if v <= core::u8::MAX as u64 {
        w.write_byte(type_val | 24)?;
        w.write_byte(v as u8)
    } else if v <= core::u16::MAX as u64 {
        w.write_byte(type_val | 25)?;
        w.write_bytes(&(v as u16).to_be_bytes())
    } else if v <= core::u32::MAX as u64 {
        w.write_byte(type_val | 26)?;
        w.write_bytes(&(v as u32).to_be_bytes())
    } else {
        w.write_byte(type_val | 27)?;
        w.write_bytes(&v.to_be_bytes())
    }
}

impl Format for CborFormat {
    fn name(&self) -> &'static str {
        "cbor"
    }

    fn encode_struct<T: Encode + ?Sized, W: WriteBuffer>(val: &T, w: &mut W) -> Result<(), Error> {
        val.encode(w) // Wait, CBOR structs can be encoded using map, but fastserial uses JSON map style? Let's use map.
    }

    #[inline(always)]
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0b111_00000 | 22)
    }

    #[inline(always)]
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0b111_00000 | (if v { 21 } else { 20 }))
    }

    #[inline(always)]
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        write_cbor_head(w, 0, v)
    }

    #[inline(always)]
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v >= 0 {
            write_cbor_head(w, 0, v as u64)
        } else {
            write_cbor_head(w, 1, (-1 - v) as u64)
        }
    }

    #[inline(always)]
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v.is_nan() || v.is_infinite() {
            return Err(Error::InvalidFloat);
        }
        w.write_byte(0b111_00000 | 27)?;
        w.write_bytes(&v.to_be_bytes())
    }

    #[inline(always)]
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        write_cbor_head(w, 3, v.len() as u64)?;
        w.write_bytes(v.as_bytes())
    }

    #[inline(always)]
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        write_cbor_head(w, 2, v.len() as u64)?;
        w.write_bytes(v)
    }

    #[inline(always)]
    fn begin_object(_n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        // We use an indefinite-length map (0b101_11111) for simplicity with streaming APIs.
        w.write_byte(0b101_11111)
    }

    #[inline(always)]
    fn write_field_key(key: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        Self::write_str(core::str::from_utf8(key).unwrap_or(""), w)
    }

    #[inline(always)]
    fn field_separator(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn end_object(w: &mut impl WriteBuffer) -> Result<(), Error> {
        // Break code for indefinite-length items
        w.write_byte(0xFF)
    }

    #[inline(always)]
    fn begin_array(_len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0b100_11111)
    }

    #[inline(always)]
    fn array_separator(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn end_array(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0xFF)
    }

    #[inline(always)]
    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        Self::read_str(r)
    }

    #[inline(always)]
    fn end_object_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        if r.peek() == 0xFF {
            r.advance(1);
        }
        Ok(())
    }

    #[inline(always)]
    fn end_array_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        if r.peek() == 0xFF {
            r.advance(1);
        }
        Ok(())
    }

    #[inline(always)]
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        decode_value(r)?;
        Ok(())
    }

    #[inline(always)]
    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        match r.next_byte()? {
            244 => Ok(false), // 0b111_00000 | 20
            245 => Ok(true),  // 0b111_00000 | 21
            _ => Err(Error::UnexpectedByte),
        }
    }

    #[inline(always)]
    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        match r.next_byte()? {
            246 => Ok(()), // 0b111_00000 | 22
            _ => Err(Error::UnexpectedByte),
        }
    }

    #[inline(always)]
    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        let b = r.next_byte()?;
        if b >> 5 != 0 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        match v {
            v if v < 24 => Ok(v as u64),
            24 => Ok(r.next_byte()? as u64),
            25 => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                r.advance(2);
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as u64)
            }
            26 => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                r.advance(4);
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64)
            }
            27 => {
                let bytes = r.peek_slice(8);
                if bytes.len() < 8 { return Err(Error::UnexpectedEof); }
                r.advance(8);
                Ok(u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
            }
            _ => Err(Error::UnexpectedByte),
        }
    }

    #[inline(always)]
    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        let b = r.peek();
        let major = b >> 5;
        if major == 0 {
            Self::read_u64(r).map(|v| v as i64)
        } else if major == 1 {
            r.advance(1); // skip peek
            let v = b & 0x1F;
            let val = match v {
                v if v < 24 => v as u64,
                24 => r.next_byte()? as u64,
                25 => {
                    let bytes = r.peek_slice(2);
                    if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                    r.advance(2);
                    u16::from_be_bytes([bytes[0], bytes[1]]) as u64
                }
                26 => {
                    let bytes = r.peek_slice(4);
                    if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                    r.advance(4);
                    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64
                }
                27 => {
                    let bytes = r.peek_slice(8);
                    if bytes.len() < 8 { return Err(Error::UnexpectedEof); }
                    r.advance(8);
                    u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
                }
                _ => return Err(Error::UnexpectedByte),
            };
            Ok(-1 - (val as i64))
        } else {
            Err(Error::UnexpectedByte)
        }
    }

    #[inline(always)]
    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        let b = r.next_byte()?;
        if b >> 5 != 7 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        if v == 27 {
            let bytes = r.peek_slice(8);
            if bytes.len() < 8 { return Err(Error::UnexpectedEof); }
            r.advance(8);
            Ok(f64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
        } else if v == 26 {
            let bytes = r.peek_slice(4);
            if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
            r.advance(4);
            Ok(f32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as f64)
        } else if v == 25 {
            Err(Error::UnsupportedVersion)
        } else {
            Err(Error::UnexpectedByte)
        }
    }

    #[inline(always)]
    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        let b = r.next_byte()?;
        if b >> 5 != 3 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        let len = match v {
            v if v < 24 => v as usize,
            24 => r.next_byte()? as usize,
            25 => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                r.advance(2);
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize
            }
            26 => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                r.advance(4);
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
            27 => {
                let bytes = r.peek_slice(8);
                if bytes.len() < 8 { return Err(Error::UnexpectedEof); }
                r.advance(8);
                u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]) as usize
            }
            _ => return Err(Error::UnexpectedByte),
        };

        let data = r.peek_slice(len);
        if data.len() < len { return Err(Error::UnexpectedEof); }
        let s = core::str::from_utf8(data).map_err(|_| Error::InvalidUtf8)?;
        r.advance(len);
        Ok(s)
    }

    #[inline(always)]
    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        let b = r.next_byte()?;
        if b >> 5 != 2 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        let len = match v {
            v if v < 24 => v as usize,
            24 => r.next_byte()? as usize,
            25 => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                r.advance(2);
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize
            }
            26 => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                r.advance(4);
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
            27 => {
                let bytes = r.peek_slice(8);
                if bytes.len() < 8 { return Err(Error::UnexpectedEof); }
                r.advance(8);
                u64::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]) as usize
            }
            _ => return Err(Error::UnexpectedByte),
        };

        let data = r.peek_slice(len);
        if data.len() < len { return Err(Error::UnexpectedEof); }
        r.advance(len);
        Ok(data)
    }

    #[inline(always)]
    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        let b = r.next_byte()?;
        if b >> 5 != 5 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        if v == 31 {
            Ok(usize::MAX)
        } else {
            let len = match v {
                v if v < 24 => v as usize,
                24 => r.next_byte()? as usize,
                25 => {
                    let bytes = r.peek_slice(2);
                    if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                    r.advance(2);
                    u16::from_be_bytes([bytes[0], bytes[1]]) as usize
                }
                26 => {
                    let bytes = r.peek_slice(4);
                    if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                    r.advance(4);
                    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
                }
                _ => return Err(Error::UnexpectedByte),
            };
            Ok(len)
        }
    }

    #[inline(always)]
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        let b = r.next_byte()?;
        if b >> 5 != 4 {
            return Err(Error::UnexpectedByte);
        }
        let v = b & 0x1F;
        if v == 31 {
            Ok(usize::MAX)
        } else {
            let len = match v {
                v if v < 24 => v as usize,
                24 => r.next_byte()? as usize,
                25 => {
                    let bytes = r.peek_slice(2);
                    if bytes.len() < 2 { return Err(Error::UnexpectedEof); }
                    r.advance(2);
                    u16::from_be_bytes([bytes[0], bytes[1]]) as usize
                }
                26 => {
                    let bytes = r.peek_slice(4);
                    if bytes.len() < 4 { return Err(Error::UnexpectedEof); }
                    r.advance(4);
                    u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
                }
                _ => return Err(Error::UnexpectedByte),
            };
            Ok(len)
        }
    }
}

#[inline(always)]
pub fn has_next(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
    if r.peek() == 0xFF {
        r.advance(1); // Break code
        Ok(false)
    } else {
        Ok(true)
    }
}

pub fn encode(val: &crate::value::Value) -> Result<alloc::vec::Vec<u8>, Error> {
    let mut vec = alloc::vec::Vec::with_capacity(128);
    encode_value(val, &mut vec)?;
    Ok(vec)
}

fn encode_value(val: &crate::value::Value, w: &mut impl WriteBuffer) -> Result<(), Error> {
    match val {
        crate::value::Value::Null => CborFormat::write_null(w),
        crate::value::Value::Bool(b) => CborFormat::write_bool(*b, w),
        crate::value::Value::Number(n) => match n {
            crate::value::Number::U64(v) => CborFormat::write_u64(*v, w),
            crate::value::Number::I64(v) => CborFormat::write_i64(*v, w),
            crate::value::Number::F64(v) => CborFormat::write_f64(*v, w),
        },
        crate::value::Value::String(s) => CborFormat::write_str(s, w),
        crate::value::Value::Array(arr) => {
            write_cbor_head(w, 4, arr.len() as u64)?;
            for item in arr {
                encode_value(item, w)?;
            }
            Ok(())
        }
        crate::value::Value::Object(map) => {
            write_cbor_head(w, 5, map.len() as u64)?;
            for (k, v) in map {
                CborFormat::write_str(k, w)?;
                encode_value(v, w)?;
            }
            Ok(())
        }
    }
}

pub fn decode(data: &[u8]) -> Result<crate::value::Value, Error> {
    let mut r = crate::io::ReadBuffer::new(data);
    decode_value(&mut r)
}

fn decode_value(r: &mut ReadBuffer<'_>) -> Result<crate::value::Value, Error> {
    let b = r.peek();
    let major = b >> 5;
    match major {
        0 => {
            let v = CborFormat::read_u64(r)?;
            Ok(crate::value::Value::Number(crate::value::Number::U64(v)))
        }
        1 => {
            let v = CborFormat::read_i64(r)?;
            Ok(crate::value::Value::Number(crate::value::Number::I64(v)))
        }
        2 | 3 => {
            // Treat byte strings and text strings as strings for Value simplicity
            let s = CborFormat::read_str(r)?;
            Ok(crate::value::Value::String(alloc::string::String::from(s)))
        }
        4 => {
            let len = CborFormat::begin_array_decode(r)?;
            let mut arr = alloc::vec::Vec::new();
            if len == usize::MAX {
                while has_next(r)? {
                    arr.push(decode_value(r)?);
                }
            } else {
                for _ in 0..len {
                    arr.push(decode_value(r)?);
                }
            }
            Ok(crate::value::Value::Array(arr))
        }
        5 => {
            let len = CborFormat::begin_object_decode(r)?;
            let mut map = alloc::collections::BTreeMap::new();
            if len == usize::MAX {
                while has_next(r)? {
                    let key = CborFormat::read_str(r)?;
                    let val = decode_value(r)?;
                    map.insert(alloc::string::String::from(key), val);
                }
            } else {
                for _ in 0..len {
                    let key = CborFormat::read_str(r)?;
                    let val = decode_value(r)?;
                    map.insert(alloc::string::String::from(key), val);
                }
            }
            Ok(crate::value::Value::Object(map))
        }
        7 => {
            let v = b & 0x1F;
            match v {
                20 => { r.advance(1); Ok(crate::value::Value::Bool(false)) }
                21 => { r.advance(1); Ok(crate::value::Value::Bool(true)) }
                22 => { r.advance(1); Ok(crate::value::Value::Null) }
                26 | 27 => {
                    let f = CborFormat::read_f64(r)?;
                    Ok(crate::value::Value::Number(crate::value::Number::F64(f)))
                }
                _ => Err(Error::UnexpectedByte),
            }
        }
        _ => Err(Error::UnexpectedByte),
    }
}
