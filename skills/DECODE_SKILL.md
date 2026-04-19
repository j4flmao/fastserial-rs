---
name: fastserial/decode
description: >
  How to implement Decode<'de> for a new type, or modify existing decode logic.
  Covers: zero-copy lifetimes, primitive decode, struct decode, perfect hash dispatch.
parent-skill: SKILL.md
---

# Decode Skill

## The Lifetime Rule — Read This First

`Decode<'de>` means the decoded value **may borrow from the input buffer**.

```
ReadBuffer<'de>  holds  &'de [u8]
                              │
                              └── decoded &'de str borrows from here
                              └── decoded &'de [u8] borrows from here
```

**Rule:** If your decoded type contains any `&'de str` or `&'de [u8]`, the lifetime `'de` must appear in the type's generic parameters.

```rust
// CORRECT — borrows from input, zero allocation
struct Event<'de> {
    name: &'de str,
}
impl<'de> Decode<'de> for Event<'de> { ... }

// ALSO CORRECT — owned, allocates String
struct EventOwned {
    name: String,
}
impl<'de> Decode<'de> for EventOwned { ... }
// (still needs 'de on the impl even though Event doesn't use it)
```

---

## Primitive Decode

Add to `src/codec/json.rs`:

```rust
/// Decode a u64 from decimal ASCII.
/// Advances `r.pos` past the number.
pub fn read_u64(r: &mut ReadBuffer<'_>) -> Result<u64, Error> {
    let start = r.pos;
    while r.pos < r.data.len() && r.data[r.pos].is_ascii_digit() {
        r.pos += 1;
    }
    if r.pos == start {
        return Err(Error::UnexpectedByte {
            expected: "digit",
            got: r.data.get(start).copied().unwrap_or(0),
            offset: start,
        });
    }
    // Parse accumulated digits — no allocation
    let digits = &r.data[start..r.pos];
    let mut n = 0u64;
    for &b in digits {
        n = n.checked_mul(10)
             .and_then(|n| n.checked_add((b - b'0') as u64))
             .ok_or(Error::NumberOverflow { type_name: "u64" })?;
    }
    Ok(n)
}
```

Then the `Decode` impl in `src/lib.rs`:

```rust
impl<'de> Decode<'de> for u64 {
    #[inline(always)]
    fn decode(r: &mut ReadBuffer<'de>) -> Result<Self, Error> {
        crate::codec::json::skip_whitespace(r);
        crate::codec::json::read_u64(r)
    }
}
```

---

## Zero-copy String Decode

**Most important function in the library.** Returns `&'de str` — borrows directly from the input slice.

```rust
/// Decode a JSON string as a borrowed &'de str.
/// Returns Error if the string contains escape sequences
/// (caller must use read_str_unescape for those).
pub fn read_str_borrowed<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;

    // SIMD-accelerated scan for quote or backslash
    let rel = simd::scan_quote_or_backslash(&r.data[r.pos..]);

    if r.data.get(r.pos + rel) == Some(&b'\\') {
        // Has escapes — fall back to allocating path
        return Err(Error::NeedsUnescape { start });
    }

    // No escapes — return borrowed slice
    let end = r.pos + rel;
    r.expect_at(end, b'"')?;
    r.pos = end + 1;

    // SAFETY: we scanned from valid UTF-8 input; no escapes means bytes are
    // identical to the source, which was validated at construction.
    Ok(unsafe { std::str::from_utf8_unchecked(&r.data[start..end]) })
}
```

For the common case (no escapes), this is literally: scan for `"`, return sub-slice. One SIMD call, zero copies.

---

## Struct Decode via `#[derive(Decode)]`

The proc-macro generates:

```rust
impl<'de> fastserial::Decode<'de> for Payload<'de> {
    fn decode(r: &mut fastserial::io::ReadBuffer<'de>)
        -> Result<Self, fastserial::Error>
    {
        // State variables — all None initially
        let mut f_event_id: Option<u64>       = None;
        let mut f_name:     Option<&'de str>   = None;
        let mut f_tags:     Option<Vec<&'de str>> = None;

        fastserial::codec::json::expect_byte(b'{', r)?;
        fastserial::codec::json::skip_whitespace(r);

        loop {
            if r.peek() == b'}' { r.advance(1); break; }

            // Read field key — zero-copy borrow
            let key = fastserial::codec::json::read_key(r)?;
            fastserial::codec::json::expect_byte(b':', r)?;
            fastserial::codec::json::skip_whitespace(r);

            // Perfect hash — no string comparison
            match fastserial::phf::lookup(key.as_bytes(), SCHEMA_HASH, 3) {
                0 => f_event_id = Some(fastserial::Decode::decode(r)?),
                1 => f_name     = Some(fastserial::Decode::decode(r)?),
                2 => f_tags     = Some(fastserial::Decode::decode(r)?),
                _ => fastserial::codec::json::skip_value(r)?,
            }

            // Skip comma or detect closing brace
            fastserial::codec::json::skip_comma_or_close(r, b'}')?;
        }

        Ok(Payload {
            event_id: f_event_id.ok_or_else(|| Error::missing_field("event_id"))?,
            name:     f_name    .ok_or_else(|| Error::missing_field("name"))?,
            tags:     f_tags    .ok_or_else(|| Error::missing_field("tags"))?,
        })
    }
}
```

---

## Perfect Hash Field Dispatch

`fastserial::phf::lookup` is the key to eliminating string comparisons.

### How it works

At derive time, the proc-macro computes a function `h(key) → field_index` that maps each known field name to a unique index with zero collisions for that struct's field set.

```rust
// src/phf.rs
#[inline(always)]
pub fn lookup(key: &[u8], schema_hash: u64, n_fields: usize) -> usize {
    let h = fnv1a(key) ^ schema_hash;
    // Modulo n_fields — works collision-free for the compile-time field set
    (h as usize) % n_fields
}
```

The proc-macro verifies at compile time that no two fields in the struct map to the same index. If they would, it adjusts `schema_hash` until no collision exists.

### Fallback for unknown fields

The `_ =>` arm calls `skip_value()` — reads and discards the value without allocation. This is the forward-compatibility path.

---

## `#[serial(default)]` fields

When a field is marked `#[serial(default)]`, the proc-macro changes:

```rust
// Without #[serial(default)]:
f_flags.ok_or_else(|| Error::missing_field("flags"))?

// With #[serial(default)]:
f_flags.unwrap_or_default()
```

---

## `ReadBuffer<'de>` API

```rust
impl<'de> ReadBuffer<'de> {
    pub fn new(data: &'de [u8]) -> Self { Self { data, pos: 0 } }

    /// Current byte without advancing
    pub fn peek(&self) -> u8 {
        self.data.get(self.pos).copied().unwrap_or(0)
    }

    /// Advance pos and return byte
    pub fn next_byte(&mut self) -> Result<u8, Error> {
        self.data.get(self.pos)
            .copied()
            .map(|b| { self.pos += 1; b })
            .ok_or(Error::UnexpectedEof)
    }

    /// Expect a specific byte at current pos
    pub fn expect_byte(&mut self, expected: u8) -> Result<(), Error> {
        let got = self.next_byte()?;
        if got != expected {
            Err(Error::UnexpectedByte { expected: "...", got, offset: self.pos - 1 })
        } else {
            Ok(())
        }
    }

    /// Skip n bytes
    pub fn advance(&mut self, n: usize) {
        self.pos = (self.pos + n).min(self.data.len());
    }

    /// Return sub-slice from start to current pos (borrowed)
    pub fn slice_from(&self, start: usize) -> &'de [u8] {
        &self.data[start..self.pos]
    }
}
```

---

## Testing Decode

```rust
#[test]
fn decode_zero_copy() {
    let input = br#"{"event_id":42,"name":"login","tags":["auth","web"]}"#;
    let p: Payload<'_> = fastserial::json::decode(input).unwrap();

    assert_eq!(p.event_id, 42);
    assert_eq!(p.name, "login");
    assert_eq!(p.tags, vec!["auth", "web"]);

    // Verify zero-copy: name is a sub-slice of input
    let input_start = input.as_ptr() as usize;
    let name_start  = p.name.as_ptr() as usize;
    assert!(name_start >= input_start && name_start < input_start + input.len(),
            "name should borrow from input buffer");
}

#[test]
fn decode_missing_required_field() {
    let input = br#"{"event_id":1}"#;
    let result: Result<Payload<'_>, _> = fastserial::json::decode(input);
    assert!(matches!(result, Err(fastserial::Error::MissingField { name }) if name == "name"));
}

#[test]
fn decode_unknown_fields_skipped() {
    let input = br#"{"event_id":1,"name":"x","tags":[],"extra":"ignored"}"#;
    let p: Payload<'_> = fastserial::json::decode(input).unwrap();
    assert_eq!(p.event_id, 1);
}
```
