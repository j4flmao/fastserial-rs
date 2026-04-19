---
name: fastserial
description: >
  Use this skill whenever working on the fastserial-rs codebase.
  Covers: implementing Encode/Decode for new types, adding SIMD routines,
  implementing new formats, writing benchmarks, using the proc-macro system.
  Read this file first, then load the sub-skill for your specific task.
rust-version: "1.94"
---

# fastserial — Agent Coding Skill

## Read This First

fastserial is a zero-copy serialization library. Every decision optimizes for:
1. **Zero allocation on decode** when using `&'de str` / `&'de [u8]`
2. **No virtual dispatch** in the hot path — no `dyn Trait`
3. **SIMD acceleration** hidden behind a safe API

---

## Sub-skills — Load the Right One

| Task | Skill file |
|------|-----------|
| Implement `Encode`/`Decode` for a new type | [`ENCODE_SKILL.md`](ENCODE_SKILL.md) |
| Add or modify `Decode` | [`DECODE_SKILL.md`](DECODE_SKILL.md) |
| Add a SIMD routine | [`SIMD_SKILL.md`](SIMD_SKILL.md) |
| Add a new wire format | [`FORMAT_SKILL.md`](FORMAT_SKILL.md) |

---

## Crate Layout (quick reference)

```
src/
  lib.rs            ← public API
  codec/json.rs     ← JSON encode/decode primitives
  codec/binary.rs   ← binary encode/decode
  codec/msgpack.rs  ← MessagePack
  simd/mod.rs       ← dispatch (AVX2 / SSE4.2 / scalar)
  simd/avx2.rs      ← AVX2 implementations
  simd/sse42.rs     ← SSE4.2 implementations
  simd/scalar.rs    ← portable fallback
  schema/types.rs   ← FieldType enum
  io/writer.rs      ← WriteBuffer trait
  io/reader.rs      ← ReadBuffer<'de>
fastserial-derive/
  src/encode.rs     ← #[derive(Encode)] codegen
  src/decode.rs     ← #[derive(Decode)] codegen
  src/attrs.rs      ← #[serial(...)] attribute parsing
```

---

## Non-negotiable Rules

- **No `unwrap()` in library code** — always `?` or explicit error
- **No `dyn Trait` in hot path** — every encoder/decoder must be monomorphized
- **Every `unsafe` block** must have `// SAFETY:` comment
- **Every `#[target_feature]` fn** must be called only after runtime detection
- **MSRV is Rust 1.94** — do not use features from later versions
- **No new dependencies** without maintainer approval — check `Cargo.toml` first

---

## Key Types

```rust
// Core encode trait
pub trait Encode {
    const SCHEMA_HASH: u64;
    fn encode<W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error>;
    fn encode_with_format<F: Format, W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error>;
}

// Core decode trait ('de = lifetime of input buffer)
pub trait Decode<'de>: Sized {
    fn decode(r: &mut ReadBuffer<'de>) -> Result<Self, Error>;
}

// Output buffer abstraction
pub trait WriteBuffer {
    fn write_byte(&mut self, b: u8) -> Result<(), Error>;
    fn write_bytes(&mut self, bs: &[u8]) -> Result<(), Error>;
    fn reserve(&mut self, hint: usize) {}
}

// Input buffer with lifetime tracking
pub struct ReadBuffer<'de> {
    data: &'de [u8],
    pos: usize,
}
```

---

## Common Patterns

### Encode a primitive

```rust
// In src/codec/json.rs
#[inline(always)]
pub fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();
    w.write_bytes(buf.format(v).as_bytes())
}
```

### Decode a borrowed string (zero-copy)

```rust
// Returns &'de str — borrows from the ReadBuffer's input slice
pub fn read_str<'de>(r: &mut ReadBuffer<'de>) -> Result<&'de str, Error> {
    r.expect_byte(b'"')?;
    let start = r.pos;
    let end = simd::scan_quote_or_backslash(&r.data[r.pos..]);
    // ... handle escapes ...
    let s = std::str::from_utf8(&r.data[start..start + end])
        .map_err(|_| Error::InvalidUtf8 { byte_offset: start })?;
    r.pos = start + end + 1; // skip closing quote
    Ok(s)
}
```

### Check if type needs allocation

- `&'de str` → zero-copy (borrowed from input)
- `&'de [u8]` → zero-copy (borrowed from input)
- `String` → allocates (one `String::from_utf8`)
- `Vec<T>` → allocates
- `u8..u64`, `i8..i64`, `f32`, `f64`, `bool` → stack only

---

## Error Handling

```rust
// Return errors with context
return Err(Error::UnexpectedByte {
    expected: "opening '{'",
    got: byte,
    offset: r.pos,
});

// Propagate with ?
let v = read_u64(r)?;

// Missing field error
f_id.ok_or_else(|| Error::missing_field("id"))?
```

---

## Testing

Every code path must have a test:
```bash
cargo test                      # all tests
cargo test -- zero_copy         # specific module
cargo test --features json-strict  # feature-gated tests
```

New primitives: test against scalar + SIMD, empty input, boundary lengths (0, 1, 31, 32, 33, 64 bytes).
