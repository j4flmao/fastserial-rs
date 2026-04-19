# Proc-macro Codegen

> How `#[derive(Encode, Decode)]` generates hyper-optimized code.

## Overview

The `fastserial-derive` crate is a separate proc-macro crate. It parses Rust struct/enum definitions using `syn` and emits specialized `impl` blocks that are faster than any hand-written generic visitor pattern.

---

## Crate Structure

```
fastserial-derive/
├── Cargo.toml
└── src/
    ├── lib.rs       ← proc_macro entry points
    ├── encode.rs    ← Encode derive logic
    ├── decode.rs    ← Decode derive logic
    ├── attrs.rs     ← #[serial(...)] attribute parsing
    ├── schema.rs    ← SCHEMA_HASH computation
    └── phf.rs       ← compile-time perfect hash for field names
```

---

## Cargo.toml

```toml
[package]
name    = "fastserial-derive"
version = "0.1.0"
edition = "2021"
rust-version = "1.94"

[lib]
proc-macro = true

[dependencies]
syn      = { version = "2",   features = ["full"] }
quote    = "1"
proc-macro2 = "1"
```

---

## `#[derive(Encode)]` — What Gets Generated

### Input

```rust
#[derive(Encode)]
struct Event<'a> {
    id:        u64,
    #[fastserial(rename = "n")]
    name:      &'a str,
    timestamp: u64,
}
```

### Generated output (approximate)

```rust
impl<'a> fastserial::Encode for Event<'a> {
    // Compile-time constant: field layout fingerprint
    const SCHEMA_HASH: u64 = 14_580_293_847_120_938_475u64;

    #[inline(always)]
    fn encode<W>(&self, w: &mut W) -> ::core::result::Result<(), fastserial::Error>
    where
        W: fastserial::io::WriteBuffer,
    {
        // JSON format (default). Format-specific methods called directly.
        // All field key bytes are &'static [u8] in .rodata — zero runtime cost.
        w.write_bytes(b"{")?;
        w.write_bytes(b"\"id\":")?;
        fastserial::codec::json::write_u64(self.id, w)?;
        w.write_bytes(b",\"n\":")?;             // renamed field
        fastserial::codec::json::write_str(self.name, w)?;
        w.write_bytes(b",\"timestamp\":")?;
        fastserial::codec::json::write_u64(self.timestamp, w)?;
        w.write_bytes(b"}")?;
        Ok(())
    }

    fn encode_with_format<F, W>(
        &self,
        w: &mut W,
    ) -> ::core::result::Result<(), fastserial::Error>
    where
        F: fastserial::Format,
        W: fastserial::io::WriteBuffer,
    {
        F::begin_object(3usize, w)?;
        F::write_field_key(b"id", w)?;
        fastserial::Encode::encode(&self.id, w)?;
        F::write_field_key(b"n", w)?;
        fastserial::Encode::encode(&self.name, w)?;
        F::write_field_key(b"timestamp", w)?;
        fastserial::Encode::encode(&self.timestamp, w)?;
        F::end_object(w)?;
        Ok(())
    }
}
```

Key codegen decisions:
- `encode()` is format-hardcoded (JSON default) — zero abstraction overhead
- `encode_with_format<F>()` is the generic path — used when format is not known at derive time
- Field key bytes are `b"..."` literals in `.rodata`
- `#[inline(always)]` on the hot method — the compiler inlines this at call sites

---

## `#[derive(Decode)]` — What Gets Generated

### Generated decode (approximate)

```rust
impl<'de> fastserial::Decode<'de> for Event<'de> {
    fn decode(r: &mut fastserial::io::ReadBuffer<'de>)
        -> ::core::result::Result<Self, fastserial::Error>
    {
        // State: Option<T> for each field, filled as we encounter them
        let mut f_id:        Option<u64>     = None;
        let mut f_name:      Option<&'de str> = None;
        let mut f_timestamp: Option<u64>     = None;

        fastserial::codec::json::expect_byte(b'{', r)?;

        loop {
            fastserial::codec::json::skip_whitespace(r);
            if fastserial::codec::json::peek(r) == b'}' {
                r.advance(1);
                break;
            }

            // Read field key as borrowed &'de str — zero allocation
            let key = fastserial::codec::json::read_key(r)?;
            fastserial::codec::json::expect_byte(b':', r)?;

            // Perfect hash dispatch — no string comparison in hot path
            match fastserial::phf::lookup_2(key, 14_580_293_847_120_938_475u64) {
                0 => f_id        = Some(fastserial::Decode::decode(r)?),
                1 => f_name      = Some(fastserial::Decode::decode(r)?),
                2 => f_timestamp = Some(fastserial::Decode::decode(r)?),
                _ => fastserial::codec::json::skip_value(r)?,
            }

            fastserial::codec::json::skip_comma_or_end(r)?;
        }

        Ok(Event {
            id:        f_id       .ok_or(fastserial::Error::missing_field("id"))?,
            name:      f_name     .ok_or(fastserial::Error::missing_field("n"))?,
            timestamp: f_timestamp.ok_or(fastserial::Error::missing_field("timestamp"))?,
        })
    }
}
```

### Perfect hash for field matching

`phf::lookup_2` computes: `(hash(key_bytes) ^ schema_hash) % n_fields`. The macro pre-computes a mapping table at compile time that has zero collision for the known field set.

This replaces:
```rust
// SLOW — string comparisons
match key {
    "id"        => ...,
    "n"         => ...,
    "timestamp" => ...,
    _           => ...,
}
```

With:
```rust
// FAST — hash comparison (single multiply + xor + mod)
match phf::lookup_2(key, SCHEMA_HASH) {
    0 => ..., 1 => ..., 2 => ..., _ => ...,
}
```

---

## Attribute Reference

| Attribute | On | Effect |
|-----------|-----|--------|
| `#[fastserial(rename = "key")]` | field | Use `key` as the JSON/encoded key |
| `#[fastserial(skip)]` | field | Exclude from encode; use `Default::default()` on decode |
| `#[fastserial(default)]` | field | Use `Default::default()` if field missing on decode |
| `#[fastserial(flatten)]` | field | Inline nested struct's fields at this level |
| `#[fastserial(alias = "old_key")]` | field | Accept `old_key` as alternative on decode (for migrations) |
| `#[fastserial(format = "json")]` | struct | Force specific format for `encode()` fast path |
| `#[fastserial(deny_unknown_fields)]` | struct | Return error on unknown fields instead of skipping |

---

## Enum Derive

```rust
#[derive(Encode, Decode)]
#[fastserial(tag = "type")]   // internally tagged: {"type":"Login","user_id":42}
enum Event {
    Login  { user_id: u64 },
    Logout { user_id: u64, reason: String },
    Error  { code: u16 },
}
```

Supported tagging strategies (mirroring serde):

| Strategy | Example output |
|----------|----------------|
| `#[fastserial(tag = "t")]` | `{"t":"Login","user_id":42}` |
| `#[fastserial(tag = "t", content = "d")]` | `{"t":"Login","d":{"user_id":42}}` |
| `#[fastserial(untagged)]` | `{"user_id":42}` |
| (default) | `{"Login":{"user_id":42}}` |

---

## SCHEMA_HASH Computation

`SCHEMA_HASH` is a deterministic 64-bit fingerprint of the type's schema. It enables:
- Cross-language compatibility checks (embed hash in binary files, verify on load)
- Perfect hash seed for field dispatch
- Forward/backward compatibility warnings

Algorithm:
```
hash = FNV-1a(type_name)
for each field (in declaration order):
    hash = hash XOR FNV-1a(encoded_field_name)
    hash = hash XOR FNV-1a(type_name_of_field)
```

This is computed entirely by the proc-macro at compile time — zero runtime cost.

---

## Adding a New Derive Attribute

1. Add the attribute variant to `attrs.rs`:
   ```rust
   pub enum SerialAttr {
       Rename(String),
       Skip,
       // Add here:
       Deprecated { since: String, note: String },
   }
   ```

2. Parse it in `attrs.rs::parse_field_attrs()`

3. Use it in `encode.rs` or `decode.rs` code generation:
   ```rust
   if let Some(SerialAttr::Deprecated { ref note, .. }) = field.attrs.deprecated {
       tokens.extend(quote! {
           // emit deprecation warning in generated code
           #[allow(deprecated)]
       });
   }
   ```

4. Add test in `fastserial-derive/tests/derive_tests.rs`
