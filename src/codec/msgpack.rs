//! # MsgPack Format
//!
//! This module provides MessagePack serialization format.
//!
//! MessagePack is an efficient binary serialization format.
//! See <https://msgpack.org/> for the specification.

use crate::io::{ReadBuffer, WriteBuffer};
use crate::{Decode, Encode, Error, Format};

/// MessagePack format implementation.
///
/// Encodes/decodes values using the MessagePack specification.
pub struct MsgPackFormat;

impl Format for MsgPackFormat {
    /// Returns "msgpack".
    fn name(&self) -> &'static str {
        "msgpack"
    }

    /// Encodes a struct directly.
    fn encode_struct<T: Encode + ?Sized, W: WriteBuffer>(val: &T, w: &mut W) -> Result<(), Error> {
        val.encode(w)
    }

    /// Writes null as 0xC0.
    #[inline(always)]
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0xC0)
    }

    /// Writes bool as 0xC3 (true) or 0xC2 (false).
    #[inline(always)]
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(if v { 0xC3 } else { 0xC2 })
    }

    /// Writes u64 with optimal byte size.
    #[inline(always)]
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v < 0x80 {
            w.write_byte(v as u8)?;
        } else if v < 0x100 {
            w.write_byte(0xCC)?;
            w.write_byte(v as u8)?;
        } else if v < 0x10000 {
            w.write_byte(0xCD)?;
            w.write_bytes(&(v as u16).to_be_bytes())?;
        } else if v < 0x1_0000_0000 {
            w.write_byte(0xCE)?;
            w.write_bytes(&(v as u32).to_be_bytes())?;
        } else {
            w.write_byte(0xCF)?;
            w.write_bytes(&v.to_be_bytes())?;
        }
        Ok(())
    }

    #[inline(always)]
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v >= -(1i64 << 5) && v < (1i64 << 7) {
            w.write_byte((v & 0xFF) as u8)?;
        } else if v >= i64::from(i16::MIN) && v <= i64::from(i16::MAX) {
            w.write_byte(0xD1)?;
            w.write_bytes(&(v as i16).to_be_bytes())?;
        } else if v >= i64::from(i32::MIN) && v <= i64::from(i32::MAX) {
            w.write_byte(0xD2)?;
            w.write_bytes(&(v as i32).to_be_bytes())?;
        } else {
            w.write_byte(0xD3)?;
            w.write_bytes(&v.to_be_bytes())?;
        }
        Ok(())
    }

    #[inline(always)]
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v.is_nan() || v.is_infinite() {
            return Err(Error::InvalidFloat);
        }
        w.write_byte(0xCB)?;
        w.write_bytes(&v.to_be_bytes())
    }

    #[inline(always)]
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = v.len();
        if len < 32 {
            w.write_byte((0xA0 | len) as u8)?;
        } else if len < 0x100 {
            w.write_byte(0xD9)?;
            w.write_byte(len as u8)?;
        } else if len < 0x10000 {
            w.write_byte(0xDA)?;
            w.write_bytes(&(len as u16).to_be_bytes())?;
        } else {
            w.write_byte(0xDB)?;
            w.write_bytes(&(len as u32).to_be_bytes())?;
        }
        w.write_bytes(v.as_bytes())
    }

    #[inline(always)]
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = v.len();
        if len < 0x100 {
            w.write_byte(0xC4)?;
            w.write_byte(len as u8)?;
        } else if len < 0x10000 {
            w.write_byte(0xC5)?;
            w.write_bytes(&(len as u16).to_be_bytes())?;
        } else {
            w.write_byte(0xC6)?;
            w.write_bytes(&(len as u32).to_be_bytes())?;
        }
        w.write_bytes(v)
    }

    #[inline(always)]
    fn begin_object(_n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(0x80)
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
    fn end_object(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn begin_array(len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if len < 16 {
            w.write_byte((0x90 | len) as u8)?;
        } else if len < 0x10000 {
            w.write_byte(0xDC)?;
            w.write_bytes(&(len as u16).to_be_bytes())?;
        } else {
            w.write_byte(0xDD)?;
            w.write_bytes(&(len as u32).to_be_bytes())?;
        }
        Ok(())
    }

    #[inline(always)]
    fn array_separator(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn end_array(_w: &mut impl WriteBuffer) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        match r.next_byte()? {
            0xC2 => Ok(false),
            0xC3 => Ok(true),
            b => Err(Error::UnexpectedByte {
                expected: "bool",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        match r.next_byte()? {
            b if b < 0x80 => Ok(b as u64),
            0xCC => Ok(r.next_byte()? as u64),
            0xCD => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as u64)
            }
            0xCE => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as u64)
            }
            0xCF => {
                let bytes = r.peek_slice(8);
                if bytes.len() < 8 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(8);
                Ok(u64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            }
            b => Err(Error::UnexpectedByte {
                expected: "unsigned integer",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        let b = r.next_byte()?;
        if b < 0xE0 {
            return Ok(b as i8 as i64);
        }
        match b {
            0xD0 => Ok(r.next_byte()? as i8 as i64),
            0xD1 => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                Ok(i16::from_be_bytes([bytes[0], bytes[1]]) as i64)
            }
            0xD2 => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                Ok(i32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as i64)
            }
            0xD3 => {
                let bytes = r.peek_slice(8);
                if bytes.len() < 8 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(8);
                Ok(i64::from_be_bytes([
                    bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
                ]))
            }
            _ => Err(Error::UnexpectedByte {
                expected: "signed integer",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        r.expect_byte(0xCB)?;
        let bytes = r.peek_slice(8);
        if bytes.len() < 8 {
            return Err(Error::UnexpectedEof);
        }
        r.advance(8);
        Ok(f64::from_be_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    #[inline(always)]
    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        let len = match r.next_byte()? {
            b if b < 0xC0 => {
                if b < 0xA0 {
                    return Err(Error::UnexpectedByte {
                        expected: "fixstr",
                        got: b,
                        offset: r.get_pos() - 1,
                    });
                }
                (b & 0x1F) as usize
            }
            0xD9 => r.next_byte()? as usize,
            0xDA => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize
            }
            0xDB => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
            b => {
                return Err(Error::UnexpectedByte {
                    expected: "string",
                    got: b,
                    offset: r.get_pos() - 1,
                });
            }
        };

        let data = r.peek_slice(len);
        if data.len() < len {
            return Err(Error::UnexpectedEof);
        }
        let s = core::str::from_utf8(data).map_err(|_| Error::InvalidUtf8 {
            byte_offset: r.get_pos(),
        })?;
        r.advance(len);
        Ok(s)
    }

    #[inline(always)]
    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        let len = match r.next_byte()? {
            0xC4 => r.next_byte()? as usize,
            0xC5 => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                u16::from_be_bytes([bytes[0], bytes[1]]) as usize
            }
            0xC6 => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize
            }
            b => {
                return Err(Error::UnexpectedByte {
                    expected: "binary",
                    got: b,
                    offset: r.get_pos() - 1,
                });
            }
        };

        let data = r.peek_slice(len);
        if data.len() < len {
            return Err(Error::UnexpectedEof);
        }
        r.advance(len);
        Ok(data)
    }

    #[inline(always)]
    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        match r.next_byte()? {
            0xC0 => Ok(()),
            b => Err(Error::UnexpectedByte {
                expected: "null",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        match r.next_byte()? {
            b if b < 0x90 => {
                if b < 0x80 {
                    Ok((b & 0x0F) as usize)
                } else {
                    Err(Error::UnexpectedByte {
                        expected: "fixmap",
                        got: b,
                        offset: r.get_pos() - 1,
                    })
                }
            }
            0x80 => Ok(0),
            0xDE => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as usize)
            }
            0xDF => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize)
            }
            b => Err(Error::UnexpectedByte {
                expected: "map",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        Self::read_str(r)
    }

    #[inline(always)]
    fn end_object_decode(_r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        match r.next_byte()? {
            b if b < 0x90 => Ok((b & 0x0F) as usize),
            0xDC => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(2);
                Ok(u16::from_be_bytes([bytes[0], bytes[1]]) as usize)
            }
            0xDD => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                r.advance(4);
                Ok(u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize)
            }
            b => Err(Error::UnexpectedByte {
                expected: "array",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }

    #[inline(always)]
    fn end_array_decode(_r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        Ok(())
    }

    #[inline(always)]
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        let b = r.next_byte()?;
        match b {
            0xC0 => Ok(()),
            0xC2 | 0xC3 => Ok(()),
            0xC4..=0xC6 => {
                let len = match b {
                    0xC4 => 1,
                    0xC5 => 2,
                    _ => 4,
                };
                r.advance(len + 1);
                Ok(())
            }
            0xC7..=0xC9 => {
                let len = match b {
                    0xC7 => 1,
                    0xC8 => 2,
                    _ => 4,
                };
                r.advance(len);
                let ext_len = match r.next_byte()? {
                    n => n as usize,
                };
                r.advance(ext_len + 1);
                Ok(())
            }
            0xCA => {
                r.advance(4);
                Ok(())
            }
            0xCB => {
                r.advance(8);
                Ok(())
            }
            0xCC => {
                r.advance(1);
                Ok(())
            }
            0xCD => {
                r.advance(2);
                Ok(())
            }
            0xCE => {
                r.advance(4);
                Ok(())
            }
            0xCF => {
                r.advance(8);
                Ok(())
            }
            0xD0 => {
                r.advance(1);
                Ok(())
            }
            0xD1 => {
                r.advance(2);
                Ok(())
            }
            0xD2 => {
                r.advance(4);
                Ok(())
            }
            0xD3 => {
                r.advance(8);
                Ok(())
            }
            0xD4..=0xD8 => {
                r.advance(2);
                Ok(())
            }
            0xD9 => {
                let len = r.next_byte()? as usize;
                r.advance(len);
                Ok(())
            }
            0xDA => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                let len = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                r.advance(2 + len);
                Ok(())
            }
            0xDB => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                let len = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                r.advance(4 + len);
                Ok(())
            }
            0xDC => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                let count = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                r.advance(2);
                for _ in 0..count {
                    Self::skip_value(r)?;
                }
                Ok(())
            }
            0xDD => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                let count = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                r.advance(4);
                for _ in 0..count {
                    Self::skip_value(r)?;
                }
                Ok(())
            }
            0xDE => {
                let bytes = r.peek_slice(2);
                if bytes.len() < 2 {
                    return Err(Error::UnexpectedEof);
                }
                let count = u16::from_be_bytes([bytes[0], bytes[1]]) as usize;
                r.advance(2);
                for _ in 0..count * 2 {
                    Self::skip_value(r)?;
                }
                Ok(())
            }
            0xDF => {
                let bytes = r.peek_slice(4);
                if bytes.len() < 4 {
                    return Err(Error::UnexpectedEof);
                }
                let count = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
                r.advance(4);
                for _ in 0..count * 2 {
                    Self::skip_value(r)?;
                }
                Ok(())
            }
            b if b < 0x80 => Ok(()),
            b if b < 0x90 => {
                let count = (b & 0x0F) as usize;
                for _ in 0..count * 2 {
                    Self::skip_value(r)?;
                }
                Ok(())
            }
            b if b < 0xA0 => Ok(()),
            b if b < 0xC0 => {
                let len = (b & 0x1F) as usize;
                r.advance(len);
                Ok(())
            }
            b => Err(Error::UnexpectedByte {
                expected: "msgpack value",
                got: b,
                offset: r.get_pos() - 1,
            }),
        }
    }
}

pub mod encode {
    use super::*;

    pub fn encode<T: Encode>(val: &T) -> Result<alloc::vec::Vec<u8>, Error> {
        let mut buf = alloc::vec::Vec::with_capacity(256);
        val.encode(&mut buf)?;
        Ok(buf)
    }

    pub fn encode_into<T: Encode>(val: &T, buf: &mut alloc::vec::Vec<u8>) -> Result<(), Error> {
        val.encode(buf)
    }
}

pub mod decode {
    use super::*;

    pub fn decode<'de, T: Decode<'de>>(input: &'de [u8]) -> Result<T, Error> {
        let mut r = crate::io::ReadBuffer::new(input);
        T::decode(&mut r)
    }
}
