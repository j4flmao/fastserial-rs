use crate::Error;
use crate::io::{ReadBuffer, WriteBuffer};
use crate::simd;

/// A high-performance JSON encoding and decoding implementation.
///
/// This module provides specialized traits and functions for working with JSON data.
/// It leverages SIMD acceleration for scanning and escaping, and procedural macros
/// for specialized code generation.
///
/// # Examples
///
/// ```rust
/// use fastserial::{Encode, Decode, json};
///
/// #[derive(Encode, Decode, Debug, PartialEq)]
/// struct Point {
///     x: i32,
///     y: i32,
/// }
///
/// # fn main() -> Result<(), fastserial::Error> {
/// let p = Point { x: 1, y: 2 };
/// let json_data = json::encode(&p)?;
/// assert_eq!(String::from_utf8_lossy(&json_data), r#"{"x":1,"y":2}"#);
/// # Ok(())
/// # }
/// ```
pub trait Format {
    /// Encodes a struct using this format.
    ///
    /// This is the entry point for the `Encode` trait to delegate its implementation
    /// to a specific format.
    fn encode_struct<T: crate::Encode, W: WriteBuffer>(val: &T, w: &mut W) -> Result<(), Error> {
        val.encode(w)
    }

    /// Writes a JSON `null` value.
    ///
    /// # Errors
    /// Returns `Error::BufferFull` or `Error::UnexpectedEof` if the buffer cannot fit "null".
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(b"null")
    }

    /// Writes a JSON boolean value (`true` or `false`).
    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(if v { b"true" } else { b"false" })
    }

    /// Writes a JSON unsigned 64-bit integer.
    ///
    /// Uses the `itoa` crate for high-performance integer-to-string conversion.
    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    /// Writes a JSON signed 64-bit integer.
    ///
    /// Uses the `itoa` crate for high-performance integer-to-string conversion.
    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    /// Writes a JSON 64-bit floating point number.
    ///
    /// Uses the `ryu` crate for high-performance float-to-string conversion.
    ///
    /// # Errors
    /// Returns `Error::InvalidFloat` if the value is `NaN` or infinite.
    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v.is_nan() || v.is_infinite() {
            return Err(Error::InvalidFloat);
        }
        let mut buf = ryu::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    /// Encodes a string slice into the JSON format, escaping any special characters.
    ///
    /// # Arguments
    /// * `v` - The string slice to encode.
    /// * `w` - The `WriteBuffer` to write the encoded JSON string to.
    ///
    /// # Returns
    /// `Ok(())` if encoding is successful, or an `Error` otherwise.
    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'"')?;
        let bytes = v.as_bytes();
        let mut start = 0usize;

        for i in 0..bytes.len() {
            let b = bytes[i];
            if b == b'"' || b == b'\\' || b == b'\n' || b == b'\r' || b == b'\t' || b < 0x20 {
                if i > start {
                    w.write_bytes(&bytes[start..i])?;
                }
                match b {
                    b'"' => w.write_bytes(b"\\\"")?,
                    b'\\' => w.write_bytes(b"\\\\")?,
                    b'\n' => w.write_bytes(b"\\n")?,
                    b'\r' => w.write_bytes(b"\\r")?,
                    b'\t' => w.write_bytes(b"\\t")?,
                    _ => {
                        w.write_bytes(b"\\u00")?;
                        w.write_bytes(&hex_digit(b >> 4))?;
                        w.write_bytes(&hex_digit(b & 0x0f))?;
                    }
                }
                start = i + 1;
            }
        }

        if start < bytes.len() {
            w.write_bytes(&bytes[start..])?;
        }

        w.write_byte(b'"')
    }

    /// Writes a slice of bytes as a JSON string, escaping any special characters.
    ///
    /// # Arguments
    /// * `v` - The byte slice to encode.
    /// * `w` - The `WriteBuffer` to write the encoded JSON string to.
    ///
    /// # Returns
    /// `Ok(())` if encoding is successful, or an `Error` otherwise.
    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'"')?;
        let mut start = 0usize;

        for i in 0..v.len() {
            let b = v[i];
            if b == b'"' || b == b'\\' || b < 0x20 {
                if i > start {
                    w.write_bytes(&v[start..i])?;
                }
                match b {
                    b'"' => w.write_bytes(b"\\\"")?,
                    b'\\' => w.write_bytes(b"\\\\")?,
                    _ => {
                        w.write_bytes(b"\\u00")?;
                        w.write_bytes(&hex_digit(b >> 4))?;
                        w.write_bytes(&hex_digit(b & 0x0f))?;
                    }
                }
                start = i + 1;
            }
        }

        if start < v.len() {
            w.write_bytes(&v[start..])?;
        }

        w.write_byte(b'"')
    }

    /// Begins a JSON object.
    fn begin_object(n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let _ = n_fields;
        w.write_byte(b'{')
    }

    /// Writes a JSON object field key and its separator.
    ///
    /// # Arguments
    /// * `key` - The field key as a byte slice.
    /// * `w` - The `WriteBuffer` to write to.
    fn write_field_key(key: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'"')?;
        w.write_bytes(key)?;
        w.write_bytes(b"\":")
    }

    /// Writes a separator between a JSON object key and its value.
    fn field_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b':')
    }

    /// Writes a separator between JSON object fields.
    fn object_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b',')
    }

    /// Ends a JSON object.
    fn end_object(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'}')
    }

    /// Begins a JSON array.
    fn begin_array(len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let _ = len;
        w.write_byte(b'[')
    }

    /// Writes a separator between JSON array elements.
    fn array_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b',')
    }

    /// Ends a JSON array.
    fn end_array(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b']')
    }

    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        skip_whitespace(r);
        match r.peek() {
            b't' => {
                r.expect_bytes(b"true")?;
                Ok(true)
            }
            b'f' => {
                r.expect_bytes(b"false")?;
                Ok(false)
            }
            b => Err(Error::UnexpectedByte {
                expected: "boolean",
                got: b,
                offset: r.pos,
            }),
        }
    }

    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        read_unsigned(r)
    }

    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        read_signed(r)
    }

    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        read_float(r)
    }

    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        read_string(r)
    }

    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        read_bytes_impl(r)
    }

    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        r.expect_bytes(b"null")
    }

    /// Begins decoding a JSON object. Returns the number of fields if known (always 0 for JSON).
    ///
    /// # Spec
    /// According to RFC 8259, an object begins with an opening curly brace `{`.
    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        r.expect_byte(b'{')?;
        Ok(0)
    }

    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        read_string(r)
    }

    /// Ends decoding a JSON object.
    ///
    /// # Spec
    /// According to RFC 8259, an object ends with a closing curly brace `}`.
    fn end_object_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        r.expect_byte(b'}')
    }

    /// Begins decoding a JSON array. Returns the length if known (always 0 for JSON).
    ///
    /// # Spec
    /// According to RFC 8259, an array begins with an opening square bracket `[`.
    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        r.expect_byte(b'[')?;
        Ok(0)
    }

    /// Ends decoding a JSON array.
    ///
    /// # Spec
    /// According to RFC 8259, an array ends with a closing square bracket `]`.
    fn end_array_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        r.expect_byte(b']')
    }

    /// Decodes a value from the JSON buffer.
    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        skip_value(r)
    }
}

fn hex_digit(b: u8) -> [u8; 1] {
    [if b < 10 { b + b'0' } else { b - 10 + b'a' }]
}

/// Skips all leading whitespace characters in the JSON buffer.
///
/// This includes space (`0x20`), horizontal tab (`\t`), newline (`\n`), and
/// carriage return (`\r`). It uses SIMD acceleration if available.
#[inline]
pub fn skip_whitespace(r: &mut ReadBuffer<'_>) {
    let n = simd::skip_whitespace(&r.data[r.pos..]);
    r.pos += n;
}

/// Decodes an unsigned 64-bit integer from the JSON buffer.
///
/// This skips leading whitespace and then parses a sequence of ASCII digits.
///
/// # Returns
/// `Ok(u64)` if parsing is successful, or an `Error` if the value is invalid or overflows.
pub fn read_unsigned(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
    skip_whitespace(r);
    let start = r.pos;
    while r.pos < r.data.len() && r.data[r.pos].is_ascii_digit() {
        r.pos += 1;
    }
    if r.pos == start {
        return Err(Error::UnexpectedByte {
            expected: "digit",
            got: r.peek(),
            offset: r.pos,
        });
    }

    let mut n = 0u64;
    for &b in &r.data[start..r.pos] {
        n = n
            .checked_mul(10)
            .and_then(|n| n.checked_add((b - b'0') as u64))
            .ok_or(Error::NumberOverflow { type_name: "u64" })?;
    }
    Ok(n)
}

/// Decodes a signed 64-bit integer from the JSON buffer.
///
/// This handles optional leading minus signs and delegates to `read_unsigned`.
///
/// # Returns
/// `Ok(i64)` if parsing is successful, or an `Error` if the value is invalid or overflows.
pub fn read_signed(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
    skip_whitespace(r);
    let neg = r.peek() == b'-';
    if neg {
        r.pos += 1;
        let n = read_unsigned(r)?;
        if n > (i64::MAX as u64) + 1 {
            return Err(Error::NumberOverflow { type_name: "i64" });
        }
        if n == (i64::MAX as u64) + 1 {
            Ok(i64::MIN)
        } else {
            Ok(-(n as i64))
        }
    } else {
        let n = read_unsigned(r)?;
        if n > i64::MAX as u64 {
            return Err(Error::NumberOverflow { type_name: "i64" });
        }
        Ok(n as i64)
    }
}

/// Decodes a 64-bit floating point number from the JSON buffer.
///
/// This handles optional leading signs, decimal points, and scientific notation.
///
/// # Returns
/// `Ok(f64)` if parsing is successful, or an `Error` if the value is invalid.
pub fn read_float(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
    skip_whitespace(r);
    let start = r.pos;

    if r.pos < r.data.len() && (r.data[r.pos] == b'+' || r.data[r.pos] == b'-') {
        r.pos += 1;
    }

    let mut has_dot = false;
    let mut has_exp = false;

    let mut has_digits = false;
    let mut has_fractional_digits = false;

    while r.pos < r.data.len() {
        let b = r.data[r.pos];
        match b {
            b'0'..=b'9' => {
                has_digits = true;
                if has_dot && !has_exp {
                    has_fractional_digits = true;
                }
                r.pos += 1;
            }
            b'.' if !has_dot && !has_exp => {
                has_dot = true;
                r.pos += 1;
                if r.pos >= r.data.len() || !r.data[r.pos].is_ascii_digit() {
                    return Err(Error::InvalidFloat);
                }
            }
            b'e' | b'E' if !has_exp => {
                if !has_digits || (has_dot && !has_fractional_digits) {
                    return Err(Error::InvalidFloat);
                }
                has_exp = true;
                r.pos += 1;
                // Check if there's at least one digit after 'e' or 'E'
                if r.pos < r.data.len() && (r.data[r.pos] == b'+' || r.data[r.pos] == b'-') {
                    r.pos += 1;
                }
                if r.pos >= r.data.len() || !r.data[r.pos].is_ascii_digit() {
                    return Err(Error::InvalidFloat);
                }
            }
            _ => break,
        }
    }

    if r.pos == start {
        return Err(Error::InvalidFloat);
    }

    let slice = core::str::from_utf8(&r.data[start..r.pos])
        .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?;

    slice.parse::<f64>().map_err(|_| Error::InvalidFloat)
}

/// Decodes a string from the JSON buffer, borrowing from the input if no escapes are present.
///
/// # Returns
/// `Ok(&str)` if successful, or an `Error` if the string contains escapes (cannot borrow) or is invalid.
pub fn read_string<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
    match read_string_cow(r)? {
        alloc::borrow::Cow::Borrowed(s) => Ok(s),
        alloc::borrow::Cow::Owned(_) => Err(Error::UnexpectedByte {
            expected: "unescaped string",
            got: b'\\',
            offset: r.pos,
        }),
    }
}

/// Decodes a string from the JSON buffer, returning a `Cow<'de, str>`.
///
/// If the string contains no escapes, it returns a `Borrowed` slice.
/// If it contains escapes, it returns an `Owned` string with the unescaped content.
pub fn read_string_cow<'de>(
    r: &mut ReadBuffer<'de>,
) -> Result<alloc::borrow::Cow<'de, str>, Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(&r.data[r.pos..]);

    if r.pos + end >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }

    if r.data[r.pos + end] == b'"' {
        let slice = core::str::from_utf8(&r.data[start..start + end])
            .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?;
        r.pos = start + end + 1;
        return Ok(alloc::borrow::Cow::Borrowed(slice));
    }

    // Has backslash, need to unescape
    let mut s = alloc::string::String::with_capacity(end + 16);
    s.push_str(
        core::str::from_utf8(&r.data[start..start + end])
            .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?,
    );

    r.pos += end;

    while r.pos < r.data.len() {
        let b = r.data[r.pos];
        if b == b'"' {
            r.pos += 1;
            return Ok(alloc::borrow::Cow::Owned(s));
        }

        if b == b'\\' {
            r.pos += 1;
            let esc = r.next_byte()?;
            match esc {
                b'"' => s.push('"'),
                b'\\' => s.push('\\'),
                b'/' => s.push('/'),
                b'b' => s.push('\x08'),
                b'f' => s.push('\x0c'),
                b'n' => s.push('\n'),
                b'r' => s.push('\r'),
                b't' => s.push('\t'),
                b'u' => {
                    let mut code = 0u32;
                    for _ in 0..4 {
                        let hex = r.next_byte()?;
                        let digit = match hex {
                            b'0'..=b'9' => (hex - b'0') as u32,
                            b'a'..=b'f' => (hex - b'a' + 10) as u32,
                            b'A'..=b'F' => (hex - b'A' + 10) as u32,
                            _ => {
                                return Err(Error::UnexpectedByte {
                                    expected: "hex digit",
                                    got: hex,
                                    offset: r.pos - 1,
                                });
                            }
                        };
                        code = (code << 4) | digit;
                    }

                    if let Some(c) = core::char::from_u32(code) {
                        s.push(c);
                    } else {
                        return Err(Error::InvalidUtf8 {
                            byte_offset: r.pos - 6,
                        });
                    }
                }
                _ => {
                    return Err(Error::UnexpectedByte {
                        expected: "escape sequence",
                        got: esc,
                        offset: r.pos - 1,
                    });
                }
            }
        } else {
            let chunk_start = r.pos;
            let next = simd::scan_quote_or_backslash(&r.data[r.pos..]);
            s.push_str(
                core::str::from_utf8(&r.data[chunk_start..chunk_start + next]).map_err(|_| {
                    Error::InvalidUtf8 {
                        byte_offset: chunk_start,
                    }
                })?,
            );
            r.pos += next;
        }
    }

    Err(Error::UnexpectedEof)
}

pub fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    read_bytes_impl(r)
}

pub fn read_bytes_impl<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(&r.data[r.pos..]);

    if r.pos + end >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }

    r.expect_at(r.pos + end, b'"')?;
    r.pos = start;
    let result = &r.data[start..start + end];
    r.pos = start + end + 1;
    Ok(result)
}

/// Skips a single JSON value (primitive, object, or array) from the buffer.
///
/// This is used to ignore unknown fields during deserialization.
pub fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    skip_whitespace(r);
    let b = r.peek();
    match b {
        b'n' => r.expect_bytes(b"null"),
        b't' => r.expect_bytes(b"true"),
        b'f' => r.expect_bytes(b"false"),
        b'0'..=b'9' | b'-' => {
            read_float(r)?;
            Ok(())
        }
        b'"' => {
            read_string_cow(r)?;
            Ok(())
        }
        b'[' => {
            r.advance(1);
            let mut depth = 1;
            while depth > 0 {
                let b = r.next_byte()?;
                if b == b'[' {
                    depth += 1;
                } else if b == b']' {
                    depth -= 1;
                } else if b == b'"' {
                    r.pos -= 1;
                    read_string_cow(r)?;
                }
            }
            Ok(())
        }
        b'{' => {
            r.advance(1);
            let mut depth = 1;
            while depth > 0 {
                let b = r.next_byte()?;
                if b == b'{' {
                    depth += 1;
                } else if b == b'}' {
                    depth -= 1;
                } else if b == b'"' {
                    r.pos -= 1;
                    read_string_cow(r)?;
                }
            }
            Ok(())
        }
        _ => Err(Error::UnexpectedByte {
            expected: "value",
            got: b,
            offset: r.pos,
        }),
    }
}

#[inline]
pub fn skip_comma_or_close(r: &mut ReadBuffer<'_>, _close: u8) -> Result<(), Error> {
    skip_whitespace(r);
    if r.peek() == b',' {
        r.advance(1);
        skip_whitespace(r);
    }
    Ok(())
}

pub fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_u32(v: u32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_u16(v: u16, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_u8(v: u8, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_i32(v: i32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_i16(v: i16, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_i8(v: i8, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::InvalidFloat);
    }
    let mut buf = ryu::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_f32(v: f32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    if v.is_nan() || v.is_infinite() {
        return Err(Error::InvalidFloat);
    }
    let mut buf = ryu::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}

pub fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(if v { b"true" } else { b"false" })
}

pub fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_bytes(b"null")
}

pub fn write_str(s: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(b'"')?;
    let bytes = s.as_bytes();
    let mut start = 0usize;

    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        let need_escape =
            b == b'"' || b == b'\\' || b == b'\n' || b == b'\r' || b == b'\t' || b < 0x20;

        if need_escape {
            if i > start {
                w.write_bytes(&bytes[start..i])?;
            }
            match b {
                b'"' => w.write_bytes(b"\\\"")?,
                b'\\' => w.write_bytes(b"\\\\")?,
                b'\n' => w.write_bytes(b"\\n")?,
                b'\r' => w.write_bytes(b"\\r")?,
                b'\t' => w.write_bytes(b"\\t")?,
                _ => {
                    w.write_bytes(b"\\u00")?;
                    w.write_bytes(&hex_digit(b >> 4))?;
                    w.write_bytes(&hex_digit(b & 0x0f))?;
                }
            }
            start = i + 1;
        }
        i += 1;
    }

    if start < bytes.len() {
        w.write_bytes(&bytes[start..])?;
    }

    w.write_byte(b'"')
}

pub fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(b'"')?;
    let mut start = 0usize;

    for i in 0..v.len() {
        let b = v[i];
        if b == b'"' || b == b'\\' || b < 0x20 {
            if i > start {
                w.write_bytes(&v[start..i])?;
            }
            match b {
                b'"' => w.write_bytes(b"\\\"")?,
                b'\\' => w.write_bytes(b"\\\\")?,
                _ => {
                    w.write_bytes(b"\\u00")?;
                    w.write_bytes(&hex_digit(b >> 4))?;
                    w.write_bytes(&hex_digit(b & 0x0f))?;
                }
            }
            start = i + 1;
        }
    }

    if start < v.len() {
        w.write_bytes(&v[start..])?;
    }

    w.write_byte(b'"')
}

pub fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
    skip_whitespace(r);
    read_unsigned(r)
}

pub fn read_u32(r: &mut ReadBuffer<'_>) -> Result<u32, Error> {
    read_unsigned(r).map(|v| v as u32)
}

pub fn read_u16(r: &mut ReadBuffer<'_>) -> Result<u16, Error> {
    read_unsigned(r).map(|v| v as u16)
}

pub fn read_u8(r: &mut ReadBuffer<'_>) -> Result<u8, Error> {
    read_unsigned(r).map(|v| v as u8)
}

pub fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
    read_signed(r)
}

pub fn read_i32(r: &mut ReadBuffer<'_>) -> Result<i32, Error> {
    read_signed(r).map(|v| v as i32)
}

pub fn read_i16(r: &mut ReadBuffer<'_>) -> Result<i16, Error> {
    read_signed(r).map(|v| v as i16)
}

pub fn read_i8(r: &mut ReadBuffer<'_>) -> Result<i8, Error> {
    read_signed(r).map(|v| v as i8)
}

pub fn read_f32(r: &mut ReadBuffer<'_>) -> Result<f32, Error> {
    read_float(r).map(|v| v as f32)
}

pub fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
    read_float(r)
}

pub fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
    skip_whitespace(r);
    match r.peek() {
        b't' => {
            r.expect_bytes(b"true")?;
            Ok(true)
        }
        b'f' => {
            r.expect_bytes(b"false")?;
            Ok(false)
        }
        b => Err(Error::UnexpectedByte {
            expected: "boolean",
            got: b,
            offset: r.pos,
        }),
    }
}

pub fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    skip_whitespace(r);
    r.expect_bytes(b"null")
}
