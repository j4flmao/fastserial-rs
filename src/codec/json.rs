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

        loop {
            let remaining = &bytes[start..];
            if remaining.is_empty() {
                return w.write_byte(b'"');
            }

            let escape_pos = simd::scan_escape_chars(remaining);

            if escape_pos == remaining.len() {
                w.write_bytes(remaining)?;
                return w.write_byte(b'"');
            }

            if escape_pos > 0 {
                w.write_bytes(&remaining[..escape_pos])?;
            }

            let b = remaining[escape_pos];
            match b {
                b'"' => w.write_bytes(b"\\\"")?,
                b'\\' => w.write_bytes(b"\\\\")?,
                b'\n' => w.write_bytes(b"\\n")?,
                b'\r' => w.write_bytes(b"\\r")?,
                b'\t' => w.write_bytes(b"\\t")?,
                _ => {
                    w.write_bytes(b"\\u00")?;
                    w.write_bytes(&[fast_hex_digit(b >> 4), fast_hex_digit(b & 0x0f)])?;
                }
            }
            start += escape_pos + 1;
        }
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
            _b => Err(Error::UnexpectedByte),
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

static HEX_DIGIT_TABLE: [u8; 16] = [
    b'0', b'1', b'2', b'3', b'4', b'5', b'6', b'7', b'8', b'9', b'a', b'b', b'c', b'd', b'e', b'f',
];

#[inline]
fn fast_hex_digit(b: u8) -> u8 {
    HEX_DIGIT_TABLE[(b & 0x0f) as usize]
}

/// Returns true if `b` is one of the four JSON whitespace characters
/// (space, tab, line-feed, carriage-return).
#[inline(always)]
const fn is_json_ws(b: u8) -> bool {
    // Branchless check: 0x09, 0x0A, 0x0D, 0x20.
    // We use a tiny lookup mask trick on bytes < 0x21.
    matches!(b, b' ' | b'\t' | b'\n' | b'\r')
}

/// Skips JSON whitespace at the current read position.
///
/// Hot-path optimization: 99% of compact JSON has no whitespace between
/// structural tokens, so we inline a single-byte check before falling
/// back to the SIMD-accelerated scanner.
#[inline(always)]
pub fn skip_whitespace(r: &mut ReadBuffer<'_>) {
    if r.pos < r.data.len() {
        let b = unsafe { *r.data.get_unchecked(r.pos) };
        if !is_json_ws(b) {
            return;
        }
    } else {
        return;
    }
    let n = simd::skip_whitespace(&r.data[r.pos..]);
    r.pos += n;
}

/// Fast path: expects `:` and skips following whitespace.
#[inline(always)]
pub fn skip_colon(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    if r.pos < r.data.len() {
        let b = unsafe { *r.data.get_unchecked(r.pos) };
        if b == b':' {
            r.pos += 1;
            if r.pos < r.data.len() {
                let b2 = unsafe { *r.data.get_unchecked(r.pos) };
                if b2 == b' ' {
                    r.pos += 1;
                } else if is_json_ws(b2) {
                    skip_whitespace(r);
                }
            }
            return Ok(());
        }
    }
    r.expect_byte(b':')?;
    skip_whitespace(r);
    Ok(())
}

/// Fast path: skips whitespace, expects `,` or `close`, and skips following whitespace.
/// Returns `true` if comma found, `false` if `close` found.
#[inline(always)]
pub fn skip_comma_or_close(r: &mut ReadBuffer<'_>, close: u8) -> Result<bool, Error> {
    if r.pos < r.data.len() {
        let b = unsafe { *r.data.get_unchecked(r.pos) };
        if b == b',' {
            r.pos += 1;
            if r.pos < r.data.len() {
                let b2 = unsafe { *r.data.get_unchecked(r.pos) };
                if b2 == b' ' {
                    r.pos += 1;
                } else if is_json_ws(b2) {
                    skip_whitespace(r);
                }
            }
            return Ok(true);
        } else if b == close {
            return Ok(false);
        } else if is_json_ws(b) {
            // fallthrough to slow path
        } else {
            return Err(Error::UnexpectedByte);
        }
    }

    skip_whitespace(r);
    let b = r.peek();
    if b == b',' {
        r.advance(1);
        skip_whitespace(r);
        Ok(true)
    } else if b == close {
        Ok(false)
    } else {
        Err(Error::UnexpectedByte)
    }
}

/// SWAR (SIMD-Within-A-Register) decimal-integer parser.
///
/// Parses up to 8 ASCII digits at once using bit tricks. Returns
/// `(value, digit_count)`. If the chunk is not pure digits the count
/// reflects the number of leading digits.
#[inline(always)]
fn parse_8_digits_swar(chunk: u64) -> (u64, u32) {
    // Convert ASCII digits to numeric values in-place.
    // chunk = bbbbbbbb where each `b` is a byte.
    // Subtract '0' from each byte: any byte > 9 indicates non-digit.
    let zeros = 0x3030_3030_3030_3030u64;
    let nines = 0x3939_3939_3939_3939u64;

    // Detect non-digit bytes: any byte that is not in '0'..='9'.
    let lt0 = chunk.wrapping_sub(zeros);
    let gt9 = nines.wrapping_sub(chunk);
    let invalid = (lt0 | gt9) & 0x8080_8080_8080_8080u64;

    if invalid != 0 {
        // Non-digit found — count leading good digits.
        let pos = invalid.trailing_zeros() / 8;
        // Compute partial value over the leading `pos` digits using scalar code.
        let bytes = chunk.to_le_bytes();
        let mut val: u64 = 0;
        for i in 0..pos {
            val = val * 10 + (bytes[i as usize] - b'0') as u64;
        }
        return (val, pos);
    }

    // All 8 bytes are digits: classic SWAR multiply-add reduction.
    // Stage 1: pair adjacent digits (d0 d1 d2 d3 d4 d5 d6 d7) → (d0*10+d1, ..., d6*10+d7)
    let chunk = chunk - 0x3030_3030_3030_3030u64;
    let chunk = (chunk * 10 + (chunk >> 8)) & 0x00FF_00FF_00FF_00FFu64;
    let chunk = (chunk * 100 + (chunk >> 16)) & 0x0000_FFFF_0000_FFFFu64;
    let chunk = chunk * 10000 + (chunk >> 32);
    (chunk & 0xFFFF_FFFF, 8)
}

/// Reads an unsigned 64-bit decimal integer from the JSON buffer.
///
/// Uses a SWAR fast-path that parses 8 ASCII digits per cycle, then falls back
/// to a scalar tail. Each accumulator step is `checked_mul` / `checked_add`,
/// so any input that exceeds `u64::MAX` (e.g. an attacker sending a 25-digit
/// number) returns `Error::NumberOverflow` instead of silently wrapping.
#[inline(always)]
pub fn read_unsigned(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
    skip_whitespace(r);
    let start = r.pos;
    let data = &*r.data;

    let mut n: u64 = 0;
    let mut pos = start;

    const POW10: [u64; 9] = [
        1, 10, 100, 1000, 10000, 100000, 1000000, 10000000, 100000000,
    ];

    // SWAR 8-digits per cycle
    while pos + 8 <= data.len() {
        let chunk_slice: &[u8; 8] = data[pos..pos + 8].try_into().unwrap();
        let chunk = u64::from_le_bytes(*chunk_slice);
        let (val, digits) = parse_8_digits_swar(chunk);

        if digits > 0 {
            if let Some(next_n) = n
                .checked_mul(POW10[digits as usize])
                .and_then(|v| v.checked_add(val))
            {
                n = next_n;
            } else {
                return Err(Error::NumberOverflow);
            }
            pos += digits as usize;
        }

        if digits < 8 {
            break;
        }
    }

    // Scalar tail
    while pos < data.len() {
        let b = unsafe { *data.get_unchecked(pos) };
        if !b.is_ascii_digit() {
            break;
        }
        let digit = (b - b'0') as u64;

        if let Some(next_n) = n.checked_mul(10).and_then(|v| v.checked_add(digit)) {
            n = next_n;
        } else {
            return Err(Error::NumberOverflow);
        }
        pos += 1;
    }

    r.pos = pos;

    if pos == start {
        return Err(Error::UnexpectedByte);
    }

    Ok(n)
}

#[inline(always)]
pub fn read_signed(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
    skip_whitespace(r);
    let data = &*r.data;
    let neg = r.pos < data.len() && data[r.pos] == b'-';
    if neg {
        r.pos += 1;
        let n = read_unsigned(r)?;
        if n > (i64::MAX as u64) + 1 {
            return Err(Error::NumberOverflow);
        }
        if n == (i64::MAX as u64) + 1 {
            Ok(i64::MIN)
        } else {
            Ok(-(n as i64))
        }
    } else {
        let n = read_unsigned(r)?;
        if n > i64::MAX as u64 {
            return Err(Error::NumberOverflow);
        }
        Ok(n as i64)
    }
}

#[inline(always)]
pub fn read_float(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
    skip_whitespace(r);
    let start = r.pos;
    let data = &*r.data;

    // Quick check for sign
    if r.pos < data.len() && (data[r.pos] == b'+' || data[r.pos] == b'-') {
        r.pos += 1;
    }

    // Parse integer part (digits before decimal or exponent)
    let mut has_dot = false;
    let mut has_exp = false;
    let mut has_digits = false;

    // Process unrolled digit groups for speed
    while r.pos < data.len() {
        let b = data[r.pos];

        if b.is_ascii_digit() {
            has_digits = true;
            r.pos += 1;
        } else if b == b'.' && !has_dot && !has_exp && has_digits {
            has_dot = true;
            r.pos += 1;
            if r.pos >= data.len() || !data[r.pos].is_ascii_digit() {
                return Err(Error::InvalidFloat);
            }
        } else if (b == b'e' || b == b'E') && !has_exp && has_digits {
            has_exp = true;
            r.pos += 1;
            if r.pos < data.len() && (data[r.pos] == b'+' || data[r.pos] == b'-') {
                r.pos += 1;
            }
            if r.pos >= data.len() || !data[r.pos].is_ascii_digit() {
                return Err(Error::InvalidFloat);
            }
        } else {
            break;
        }
    }

    if r.pos == start {
        return Err(Error::InvalidFloat);
    }

    let slice = core::str::from_utf8(&data[start..r.pos]).map_err(|_| Error::InvalidUtf8)?;

    slice.parse::<f64>().map_err(|_| Error::InvalidFloat)
}

/// Reads a JSON string, returning a zero-copy `&'de str` borrowed from the input.
///
/// This function is **only** safe to use when the underlying string contains no
/// escape sequences (no `\"`, `\\`, `\n`, `\uXXXX`, …). If an escape is found,
/// it returns [`Error::EscapeInBorrowedString`] so the caller can fall back to
/// an owned `String` / `Cow<str>` decode.
///
/// # Why not unescape silently?
///
/// Returning a `&'de str` means the bytes must already exist contiguously in
/// the input buffer. An escaped string like `"a\"b"` decodes to the 3-byte
/// value `a"b`, which is **not** a sub-slice of the input — the input
/// literally contains `a\"b` (4 bytes). Pretending to "borrow" while silently
/// keeping the literal escape was a correctness bug present before v0.2.
#[inline(always)]
pub fn read_string<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
    r.expect_byte(b'"')?;
    let _start = r.pos;
    let remaining = r.remaining_slice();
    let end = simd::scan_quote_or_backslash(remaining);
    let abs = r.pos + end;
    if abs >= r.data.len() {
        r.pos = r.data.len();
        return Err(Error::UnexpectedEof);
    }

    match r.data[abs] {
        b'"' => {
            let bytes = r.peek_slice(end);
            let slice = if r.is_utf8_validated {
                unsafe { core::str::from_utf8_unchecked(bytes) }
            } else {
                core::str::from_utf8(bytes).map_err(|_| Error::InvalidUtf8)?
            };
            r.pos = abs + 1;
            Ok(slice)
        }
        b'\\' => Err(Error::EscapeInBorrowedString),
        _other => Err(Error::UnexpectedByte),
    }
}

#[inline(always)]
pub fn read_key_fast<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    r.expect_byte(b'"')?;

    // Use SIMD to find quote or backslash faster
    let remaining = r.remaining_slice();
    let end = simd::scan_quote_or_backslash(remaining);
    let abs = r.pos + end;

    if abs >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }

    let ch = r.data[abs];

    if ch == b'"' {
        // Found end of key
        let result = r.peek_slice(end);
        r.pos = abs + 1;
        return Ok(result);
    }

    // Found backslash in key - not allowed for keys
    if ch == b'\\' {
        return Err(Error::UnexpectedByte);
    }

    Err(Error::UnexpectedEof)
}

/// Decodes a JSON string directly into an owned `String`, taking the
/// allocation-free fast path when the string contains no escape sequences.
///
/// This helper exists to avoid the double-indirection cost of
/// `read_string_cow().into_owned()`, which is what `String::decode` was
/// doing previously: it forced a `Cow::Borrowed` to be re-checked and then
/// allocated through a generic path even though we know the destination
/// is `String`.
#[inline(always)]
pub fn read_string_owned<'de>(r: &mut ReadBuffer<'de>) -> Result<alloc::string::String, Error> {
    let open_pos = r.pos;
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(r.remaining_slice());

    if r.pos + end >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }

    // Fast path: no escapes — single allocation, single memcpy.
    if r.data[r.pos + end] == b'"' {
        let bytes = r.peek_slice(end);
        let slice = r.as_str(bytes)?;
        r.pos = start + end + 1;
        return Ok(alloc::string::String::from(slice));
    }

    // Slow path: contains escapes — rewind to the opening quote and
    // delegate to read_string_cow which has the full unescape state machine.
    r.pos = open_pos;
    Ok(read_string_cow(r)?.into_owned())
}

/// Decodes a string from the JSON buffer, returning a `Cow<'de, str>`.
///
/// If the string contains no escapes, it returns a `Borrowed` slice.
/// If it contains escapes, it returns an `Owned` string with the unescaped content.
#[inline(always)]
pub fn read_string_cow<'de>(
    r: &mut ReadBuffer<'de>,
) -> Result<alloc::borrow::Cow<'de, str>, Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(r.remaining_slice());

    if r.pos + end >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }

    if r.data[r.pos + end] == b'"' {
        let slice = if r.is_utf8_validated {
            unsafe { core::str::from_utf8_unchecked(r.peek_slice(end)) }
        } else {
            core::str::from_utf8(r.peek_slice(end)).map_err(|_| Error::InvalidUtf8)?
        };
        r.pos = start + end + 1;
        return Ok(alloc::borrow::Cow::Borrowed(slice));
    }
    let mut write_pos = start + end;
    r.pos += end;

    while r.pos < r.data.len() {
        let b = r.data[r.pos];
        if b == b'"' {
            r.pos += 1;
            let key_bytes = unsafe {
                core::slice::from_raw_parts(r.data.as_ptr().add(start), write_pos - start)
            };
            let slice = if r.is_utf8_validated {
                unsafe { core::str::from_utf8_unchecked(key_bytes) }
            } else {
                core::str::from_utf8(key_bytes).map_err(|_| Error::InvalidUtf8)?
            };
            return Ok(alloc::borrow::Cow::Borrowed(slice));
        }

        if b == b'\\' {
            r.pos += 1;
            if r.pos >= r.data.len() {
                return Err(Error::UnexpectedEof);
            }
            let esc = r.data[r.pos];
            r.pos += 1;
            match esc {
                b'"' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'"');
                    }
                    write_pos += 1;
                }
                b'\\' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\\');
                    }
                    write_pos += 1;
                }
                b'/' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'/');
                    }
                    write_pos += 1;
                }
                b'b' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\x08');
                    }
                    write_pos += 1;
                }
                b'f' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\x0c');
                    }
                    write_pos += 1;
                }
                b'n' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\n');
                    }
                    write_pos += 1;
                }
                b'r' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\r');
                    }
                    write_pos += 1;
                }
                b't' => {
                    unsafe {
                        (r.data.as_ptr() as *mut u8).add(write_pos).write(b'\t');
                    }
                    write_pos += 1;
                }
                b'u' => {
                    if r.pos + 4 > r.data.len() {
                        return Err(Error::UnexpectedEof);
                    }
                    let hex = &r.data[r.pos..r.pos + 4];
                    let code = unescape_hex(hex)?;
                    if let Some(c) = core::char::from_u32(code) {
                        let mut buf = [0; 4];
                        let s_char = c.encode_utf8(&mut buf);
                        let bytes = s_char.as_bytes();
                        unsafe {
                            std::ptr::copy_nonoverlapping(
                                bytes.as_ptr(),
                                (r.data.as_ptr() as *mut u8).add(write_pos),
                                bytes.len(),
                            );
                        }
                        write_pos += bytes.len();
                    } else {
                        return Err(Error::InvalidUtf8);
                    }
                    r.pos += 4;
                }
                _ => {
                    return Err(Error::UnexpectedByte);
                }
            }
        } else {
            let chunk_start = r.pos;
            let next = simd::scan_quote_or_backslash(&r.data[r.pos..]);
            if next > 0 {
                // copy the chunk down if write_pos != chunk_start
                if write_pos != chunk_start {
                    unsafe {
                        std::slice::from_raw_parts_mut(r.data.as_ptr() as *mut u8, r.data.len())
                    }
                    .copy_within(chunk_start..chunk_start + next, write_pos);
                }
                write_pos += next;
                r.pos += next;
            } else {
                unsafe {
                    (r.data.as_ptr() as *mut u8).add(write_pos).write(b);
                }
                write_pos += 1;
                r.pos += 1;
            }
        }
    }

    Err(Error::UnexpectedEof)
}

#[inline(always)]
fn unescape_hex(hex: &[u8]) -> Result<u32, Error> {
    let mut code = 0u32;
    for &b in hex {
        let digit = match b {
            b'0'..=b'9' => (b - b'0') as u32,
            b'a'..=b'f' => (b - b'a' + 10) as u32,
            b'A'..=b'F' => (b - b'A' + 10) as u32,
            _ => {
                return Err(Error::UnexpectedByte);
            }
        };
        code = (code << 4) | digit;
    }
    Ok(code)
}

pub fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    read_bytes_impl(r)
}

pub fn read_bytes_impl<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(r.remaining_slice());

    if r.pos + end >= r.data.len() {
        return Err(Error::UnexpectedEof);
    }
    r.expect_at(r.pos + end, b'"')?;
    let result = r.peek_slice(end);
    r.pos = start + end + 1;
    Ok(result)
}

/// Skips a single JSON value (any kind) starting at the current read position.
///
/// Used by the derive macro when an unknown field is encountered. Correctly
/// handles escape sequences inside strings — including the tricky cases
/// `"\\\""` (escaped backslash followed by quote terminator) and
/// `"\\\\\""` — by tracking backslash parity instead of looking at a single
/// previous byte.
#[inline(always)]
pub fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    skip_whitespace(r);
    match r.peek() {
        b'n' => r.expect_bytes(b"null"),
        b't' => r.expect_bytes(b"true"),
        b'f' => r.expect_bytes(b"false"),
        b'0'..=b'9' | b'-' => {
            r.pos += 1;
            while r.pos < r.data.len() {
                let c = r.data[r.pos];
                if !matches!(c, b'0'..=b'9' | b'.' | b'e' | b'E' | b'+' | b'-') {
                    break;
                }
                r.pos += 1;
            }
            Ok(())
        }
        b'"' => skip_string(r),
        b'[' => skip_container(r, b'[', b']'),
        b'{' => skip_container(r, b'{', b'}'),
        _b => Err(Error::UnexpectedByte),
    }
}

/// Skips a JSON string starting at the opening `"`. On return, `r.pos` is
/// positioned just past the closing `"`.
#[inline(always)]
fn skip_string(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    debug_assert_eq!(r.peek(), b'"');
    r.pos += 1; // opening quote
    loop {
        let rem = &r.data[r.pos..];
        let n = simd::scan_quote_or_backslash(rem);
        if n >= rem.len() {
            return Err(Error::UnexpectedEof);
        }
        r.pos += n;
        match r.data[r.pos] {
            b'"' => {
                r.pos += 1;
                return Ok(());
            }
            b'\\' => {
                // Skip the backslash + escaped character. `\uXXXX` is 5 trailing
                // bytes, anything else is 1.
                if r.pos + 1 >= r.data.len() {
                    return Err(Error::UnexpectedEof);
                }
                if r.data[r.pos + 1] == b'u' {
                    if r.pos + 6 > r.data.len() {
                        return Err(Error::UnexpectedEof);
                    }
                    r.pos += 6;
                } else {
                    r.pos += 2;
                }
            }
            _ => unreachable!(),
        }
    }
}

/// Skips a JSON container (`[...]` or `{...}`) by tracking depth.
#[inline(always)]
fn skip_container(r: &mut ReadBuffer<'_>, open: u8, close: u8) -> Result<(), Error> {
    debug_assert_eq!(r.peek(), open);
    r.pos += 1;
    let mut depth: u32 = 1;
    while depth > 0 {
        if r.pos >= r.data.len() {
            return Err(Error::UnexpectedEof);
        }
        let b = r.data[r.pos];
        if b == open {
            depth += 1;
            r.pos += 1;
        } else if b == close {
            depth -= 1;
            r.pos += 1;
        } else if b == b'"' {
            skip_string(r)?;
        } else {
            r.pos += 1;
        }
    }
    Ok(())
}

/// Estimates the element count of a JSON array starting at `data[pos]`,
/// where `data[pos]` is the byte right after the opening `[`.
///
/// The estimate counts top-level commas (depth 0) plus one. It bounds work
/// to `max_scan` bytes so we never make decoding slower than the original
/// O(N) loop. The result is a hint, never authoritative — push() will still
/// grow as needed if reality exceeds the estimate.
///
/// Strings are skipped as a single token via SIMD `scan_quote_or_backslash`.
#[inline]
pub fn estimate_array_len(data: &[u8], pos: usize, max_scan: usize) -> usize {
    let end = (pos + max_scan).min(data.len());
    let mut p = pos;
    let mut depth: i32 = 0;
    let mut count: usize = 1;
    let mut saw_value = false;

    while p < end {
        let b = data[p];
        match b {
            b'[' | b'{' => {
                depth += 1;
                saw_value = true;
                p += 1;
            }
            b']' | b'}' => {
                if depth == 0 {
                    // End of *this* array — return immediately.
                    return if saw_value { count } else { 0 };
                }
                depth -= 1;
                p += 1;
            }
            b',' if depth == 0 => {
                count += 1;
                p += 1;
            }
            b'"' => {
                p += 1;
                // Skip string body using SIMD.
                let rem = &data[p..end];
                let q = simd::scan_quote_or_backslash(rem);
                p += q;
                // Handle backslash escapes by walking forward (rare).
                while p < end && data[p] == b'\\' {
                    p += 2; // skip backslash + escaped char
                    let rem = &data[p.min(end)..end];
                    p += simd::scan_quote_or_backslash(rem);
                }
                if p < end {
                    p += 1; // skip closing quote
                }
                saw_value = true;
            }
            b' ' | b'\t' | b'\n' | b'\r' => {
                p += 1;
            }
            _ => {
                saw_value = true;
                p += 1;
            }
        }
    }

    // Truncated scan: extrapolate by ratio.
    let scanned = p - pos;
    if scanned == 0 || scanned >= data.len() - pos {
        return count;
    }
    let total = data.len() - pos;
    let approx = (count as u64).saturating_mul(total as u64) / scanned as u64;
    (approx as usize).min(1 << 20) // cap at ~1M to avoid pathological reserves
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

    loop {
        let remaining = &bytes[start..];
        if remaining.is_empty() {
            return w.write_byte(b'"');
        }

        let escape_pos = simd::scan_escape_chars(remaining);

        if escape_pos == remaining.len() {
            w.write_bytes(remaining)?;
            return w.write_byte(b'"');
        }

        if escape_pos > 0 {
            w.write_bytes(&remaining[..escape_pos])?;
        }

        let b = remaining[escape_pos];
        match b {
            b'"' => w.write_bytes(b"\\\"")?,
            b'\\' => w.write_bytes(b"\\\\")?,
            b'\n' => w.write_bytes(b"\\n")?,
            b'\r' => w.write_bytes(b"\\r")?,
            b'\t' => w.write_bytes(b"\\t")?,
            _ => {
                w.write_bytes(b"\\u00")?;
                w.write_bytes(&[fast_hex_digit(b >> 4), fast_hex_digit(b & 0x0f)])?;
            }
        }
        start += escape_pos + 1;
    }
}

pub fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(b'"')?;
    let mut start = 0usize;

    loop {
        let remaining = &v[start..];
        if remaining.is_empty() {
            return w.write_byte(b'"');
        }

        let escape_pos = simd::scan_escape_chars(remaining);

        if escape_pos == remaining.len() {
            w.write_bytes(remaining)?;
            return w.write_byte(b'"');
        }

        if escape_pos > 0 {
            w.write_bytes(&remaining[..escape_pos])?;
        }

        let b = remaining[escape_pos];
        match b {
            b'"' => w.write_bytes(b"\\\"")?,
            b'\\' => w.write_bytes(b"\\\\")?,
            _ => {
                w.write_bytes(b"\\u00")?;
                w.write_bytes(&[fast_hex_digit(b >> 4), fast_hex_digit(b & 0x0f)])?;
            }
        }
        start += escape_pos + 1;
    }
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
        _b => Err(Error::UnexpectedByte),
    }
}

pub fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
    skip_whitespace(r);
    r.expect_bytes(b"null")
}

/// Performs binary search on a sorted array of keys for O(log n) lookup.
///
/// This function is used by the derive macro for optimized field matching
/// when decoding JSON objects. Fields are sorted alphabetically during code
/// generation, then binary searched at runtime.
///
/// # Arguments
/// * `key` - The key bytes to search for
/// * `sorted_keys` - A pre-sorted array of field names
///
/// # Returns
/// The index of the matching key, or `None` if not found.
#[inline(always)]
pub fn binary_search_key(key: &[u8], sorted_keys: &[&str]) -> Option<usize> {
    if sorted_keys.is_empty() {
        return None;
    }

    let mut left = 0usize;
    let mut right = sorted_keys.len();

    while left < right {
        let mid = left + (right - left) / 2;
        let mid_key = sorted_keys[mid].as_bytes();

        match key.cmp(mid_key) {
            core::cmp::Ordering::Equal => return Some(mid),
            core::cmp::Ordering::Less => {
                if mid == 0 {
                    break;
                }
                right = mid;
            }
            core::cmp::Ordering::Greater => left = mid + 1,
        }
    }

    None
}

/// Computes a quick hash value for a byte slice using the FNV-like algorithm.
///
/// This is used as a preprocessing step for perfect hash lookup. The algorithm
/// uses a starting value of 5381 and multiplies by 33 for each byte, which
/// provides good distribution for short strings.
///
/// # Complexity
/// O(n) where n is the length of the key.
#[inline(always)]
fn quick_hash(key: &[u8]) -> u32 {
    let mut hash: u32 = 5381;
    for &b in key {
        hash = hash.wrapping_mul(33).wrapping_add(b as u32);
    }
    hash
}

/// Performs perfect hash lookup for O(1) average case performance.
///
/// Uses a simple modulo-based hash to index into the keys array. This provides
/// constant-time lookup when there are no hash collisions. For production
/// use, a better hash function with collision handling would be needed.
///
/// # Arguments
/// * `key` - The key bytes to look up
/// * `keys` - The array of keys to search in
///
/// # Returns
/// The index if found, None otherwise.
#[inline(always)]
pub fn perfect_hash_lookup(key: &[u8], keys: &[&str]) -> Option<usize> {
    let h = quick_hash(key) as usize;
    let idx = h % keys.len();
    if keys.get(idx).map(|k| k.as_bytes()) == Some(key) {
        Some(idx)
    } else {
        None
    }
}
