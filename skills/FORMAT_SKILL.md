---
name: fastserial/format
description: >
  How to implement a new wire format (e.g., CBOR, Avro, TOML).
  Covers: Format trait, encode side, decode side, feature flag, tests.
parent-skill: SKILL.md
---

# Format Skill

## Overview

A "format" in fastserial is a pluggable codec strategy. All user types implement `Encode`/`Decode` once — the format determines the wire representation.

Adding a format means implementing the `Format` trait. The proc-macro-derived `encode_with_format<F>()` methods call your trait implementations automatically.

---

## Step 1 — Create the Format File

```
src/codec/
  json.rs      ← exists
  binary.rs    ← exists
  msgpack.rs   ← exists
  myformat.rs  ← create this
```

---

## Step 2 — Implement the `Format` Trait

```rust
// src/codec/myformat.rs

use crate::{Error, Format};
use crate::io::{WriteBuffer, ReadBuffer};

pub struct MyFormat;

impl Format for MyFormat {
    // ──────────────────────────────────────────
    // Primitive encode
    // ──────────────────────────────────────────

    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(b"null")  // adjust to your format
    }

    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(if v { b"true" } else { b"false" })
    }

    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    fn write_i64(v: i64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    fn write_f64(v: f64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        if v.is_nan() || v.is_infinite() {
            return Err(Error::InvalidFloat);
        }
        let mut buf = ryu::Buffer::new();
        w.write_bytes(buf.format_finite(v).as_bytes())
    }

    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        // Implement your format's string encoding
        // Example: length-prefixed
        let len = v.len() as u32;
        w.write_bytes(&len.to_le_bytes())?;
        w.write_bytes(v.as_bytes())
    }

    fn write_bytes(v: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        let len = v.len() as u32;
        w.write_bytes(&len.to_le_bytes())?;
        w.write_bytes(v)
    }

    // ──────────────────────────────────────────
    // Structural encode
    // ──────────────────────────────────────────

    fn begin_object(n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        // Some formats (binary, msgpack) use n_fields for the header
        // JSON ignores it (writes `{` and discovers fields dynamically)
        let _ = n_fields;
        w.write_byte(b'{')
    }

    fn write_field_key(key: &[u8], w: &mut impl WriteBuffer) -> Result<(), Error> {
        // Write the field name in your format
        w.write_byte(b'"')?;
        w.write_bytes(key)?;
        w.write_bytes(b"\":")
    }

    fn field_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        // Called between fields (not after the last one)
        w.write_byte(b',')
    }

    fn end_object(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b'}')
    }

    fn begin_array(len: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let _ = len;
        w.write_byte(b'[')
    }

    fn array_separator(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b',')
    }

    fn end_array(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_byte(b']')
    }

    // ──────────────────────────────────────────
    // Primitive decode
    // ──────────────────────────────────────────

    fn read_bool(r: &mut ReadBuffer<'_>) -> Result<bool, Error> {
        todo!("parse bool from your format")
    }

    fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
        todo!()
    }

    fn read_i64(r: &mut ReadBuffer<'_>) -> Result<i64, Error> {
        todo!()
    }

    fn read_f64(r: &mut ReadBuffer<'_>) -> Result<f64, Error> {
        todo!()
    }

    fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        // MUST return a zero-copy borrow if at all possible
        todo!()
    }

    fn read_bytes<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de [u8], Error> {
        todo!()
    }

    fn read_null(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        todo!()
    }

    // ──────────────────────────────────────────
    // Structural decode
    // ──────────────────────────────────────────

    fn begin_object_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        // Return field count if known, or usize::MAX if streaming
        todo!()
    }

    fn read_field_key<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
        // Returns borrowed key — used for field dispatch in decode
        todo!()
    }

    fn end_object_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        todo!()
    }

    fn begin_array_decode(r: &mut ReadBuffer<'_>) -> Result<usize, Error> {
        todo!()
    }

    fn end_array_decode(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        todo!()
    }

    fn skip_value(r: &mut ReadBuffer<'_>) -> Result<(), Error> {
        // Skip one value without decoding — used for unknown fields
        todo!()
    }
}
```

---

## Step 3 — Add Convenience Functions

Create a public module for users:

```rust
// src/codec/myformat.rs (add at bottom)

/// Encode `val` to MyFormat bytes.
pub fn encode<T: crate::Encode>(val: &T) -> Result<Vec<u8>, Error> {
    let mut buf = Vec::with_capacity(256);
    val.encode_with_format::<MyFormat, _>(&mut buf)?;
    Ok(buf)
}

/// Encode `val` into an existing buffer (no allocation).
pub fn encode_into<T: crate::Encode>(val: &T, buf: &mut Vec<u8>) -> Result<(), Error> {
    val.encode_with_format::<MyFormat, _>(buf)
}

/// Decode `T` from MyFormat bytes (zero-copy where possible).
pub fn decode<'de, T: crate::Decode<'de>>(input: &'de [u8]) -> Result<T, Error> {
    let mut r = crate::io::ReadBuffer::new(input);
    T::decode(&mut r)
}
```

---

## Step 4 — Feature Flag

```toml
# Cargo.toml
[features]
default = ["json", "binary"]
json    = []
binary  = []
msgpack = []
myformat = []   # add this

[dependencies]
# format-specific deps go here with cfg
```

```rust
// src/lib.rs
#[cfg(feature = "myformat")]
pub mod myformat {
    pub use crate::codec::myformat::{encode, encode_into, decode, MyFormat};
}
```

---

## Step 5 — Tests

```rust
// tests/roundtrip.rs (add a section)
#[cfg(feature = "myformat")]
mod myformat_tests {
    use fastserial::{Encode, Decode};
    use fastserial::myformat;

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct Simple<'a> {
        id:   u64,
        name: &'a str,
    }

    #[test]
    fn roundtrip_simple() {
        let original = Simple { id: 42, name: "hello" };
        let encoded = myformat::encode(&original).unwrap();
        let decoded: Simple<'_> = myformat::decode(&encoded).unwrap();
        assert_eq!(original, decoded);
    }

    #[test]
    fn roundtrip_option_none() {
        let v: Option<u64> = None;
        let enc = myformat::encode(&v).unwrap();
        let dec: Option<u64> = myformat::decode(&enc).unwrap();
        assert_eq!(dec, None);
    }

    #[test]
    fn roundtrip_vec() {
        let v: Vec<u64> = vec![1, 2, 3, 100, u64::MAX];
        let enc = myformat::encode(&v).unwrap();
        let dec: Vec<u64> = myformat::decode(&enc).unwrap();
        assert_eq!(v, dec);
    }

    #[test]
    fn zero_copy_strings() {
        let input_bytes = myformat::encode(&Simple { id: 1, name: "zero-copy" }).unwrap();
        let decoded: Simple<'_> = myformat::decode(&input_bytes).unwrap();

        // Verify zero-copy: decoded.name should point into input_bytes
        let base = input_bytes.as_ptr() as usize;
        let end  = base + input_bytes.len();
        let name_ptr = decoded.name.as_ptr() as usize;
        assert!(name_ptr >= base && name_ptr < end,
            "name should be borrowed from input buffer, not a fresh allocation");
    }
}
```

---

## Format Implementation Checklist

- [ ] All `Format` trait methods implemented (no `todo!()` remaining)
- [ ] `read_str<'de>` returns zero-copy `&'de str` (no allocation for strings without escapes)
- [ ] `skip_value` correctly skips nested structures (arrays, objects)
- [ ] `encode` and `decode` convenience functions added
- [ ] Feature flag added in `Cargo.toml`
- [ ] Roundtrip tests for: primitives, Option, Vec, nested structs, empty string, empty vec
- [ ] Added to `src/lib.rs` behind `#[cfg(feature)]`
- [ ] Mentioned in README format table
- [ ] Mentioned in `docs/FORMATS.md`
