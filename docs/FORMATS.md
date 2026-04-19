# Format Specifications

> Wire format details for JSON, MessagePack, and native binary.

## Supported Formats

| Format | Feature flag | Extension | Human-readable | Zero-copy decode | Schema |
|--------|-------------|-----------|----------------|------------------|--------|
| JSON | `json` (default) | `.json` | yes | partial (`&str`) | no |
| MessagePack | `msgpack` | `.msgpack` | no | yes (`&[u8]`, `&str`) | no |
| Binary (native) | `binary` (default) | `.fbin` | no | yes (full) | via `SCHEMA_HASH` |

---

## JSON Format

### Encoding decisions

| Type | Wire representation |
|------|---------------------|
| `bool` | `true` / `false` |
| `u8..u64` | decimal integer, no leading zeros |
| `i8..i64` | decimal integer, `-` prefix for negative |
| `f32`, `f64` | shortest representation (Grisu3 / Ryu) |
| `&str`, `String` | `"..."` with JSON escape sequences |
| `&[u8]`, `Vec<u8>` | base64url (no padding), `"..."` |
| `Option<T>` | `null` or encoded `T` |
| `Vec<T>` | `[...]` |
| struct | `{...}` |
| enum (default tag) | `{"VariantName":{...}}` |
| `()` | `null` |

### Float encoding: Ryu algorithm

We use the Ryu algorithm (via the `ryu` crate) for float → string conversion. This produces the **shortest** decimal representation that round-trips exactly. Never emits `1.0000000000000002` when `1.0` is correct.

```rust
// Ryu output examples
1.0_f64    → "1.0"
0.1_f64    → "0.1"        // NOT "0.10000000000000001"
1e100_f64  → "1e100"
f64::NAN   → Error (JSON spec: NaN not allowed)
f64::INFINITY → Error
```

`NaN` and `Inf` return `Error::InvalidFloat` by default. Enable `features = ["json-allow-nan"]` to emit `null` instead.

### Escape sequences

Mandatory escapes:
- `"` → `\"`
- `\` → `\\`
- `\n` (0x0A) → `\n`
- `\r` (0x0D) → `\r`
- `\t` (0x09) → `\t`
- Control chars 0x00–0x1F → `\uXXXX`

Optional (not emitted, accepted on decode):
- `/` → `\/` (accepted but not emitted)
- All other Unicode: emitted as-is (UTF-8), never escaped as `\uXXXX` unless it's a control char

### Decode leniency

By default, decoding is lenient:
- Trailing commas in arrays/objects: accepted
- Comments (`//`, `/* */`): not accepted (enable `features = ["json-comments"]`)
- Extra fields in object: silently skipped
- `null` for non-Option field: uses `Default::default()` if `#[fastserial(default)]`, else error

Strict mode (enable with `features = ["json-strict"]`):
- Trailing commas: error
- Unknown fields: error (same as `#[fastserial(deny_unknown_fields)]`)

---

## MessagePack Format

We implement the [MessagePack spec 2.0](https://github.com/msgpack/msgpack/blob/master/spec.md).

### Type mapping

| Rust type | MsgPack format |
|-----------|---------------|
| `bool` | `true` / `false` |
| `u8` | positive fixint or uint8 |
| `u16` | uint16 |
| `u32` | uint32 |
| `u64` | uint64 |
| `i8..i64` | int8..int64 (or negative fixint) |
| `f32` | float32 |
| `f64` | float64 |
| `&str`, `String` | fixstr / str8 / str16 / str32 |
| `&[u8]`, `Vec<u8>` | bin8 / bin16 / bin32 |
| `Option<T>` | nil or encoded T |
| `Vec<T>` | fixarray / array16 / array32 |
| struct | fixmap / map16 / map32, string keys |

### Zero-copy with MessagePack

`str` and `bin` values are returned as borrowed slices from the input buffer:

```rust
// &'de str: borrows from input — zero allocation
let s: &str = fastserial::msgpack::decode_str(&buf)?;

// Vec<u8>: allocates — use &[u8] to avoid
let bytes_ref: &[u8] = fastserial::msgpack::decode_bytes_ref(&buf)?;
```

---

## Binary Format (native `.fbin`)

Our native binary format is designed for maximum speed and zero-copy on both encode and decode. It is **not** self-describing — the schema must be known by both sides (enforced via `SCHEMA_HASH`).

### File layout

```
┌─────────────────────────────────────────────────────┐
│ Magic: 0x46 0x42 0x49 0x4E ("FBIN")        4 bytes  │
│ Version: 0x01 0x00                          2 bytes  │
│ SCHEMA_HASH (little-endian u64)             8 bytes  │
│ Flags (reserved, 0x00)                      2 bytes  │
├─────────────────────────────────────────────────────┤
│ Data payload (encoded fields, in order)     N bytes  │
└─────────────────────────────────────────────────────┘
```

### Field encoding

Fields are written **in declaration order**, no field names, no separators:

| Type | Encoding |
|------|----------|
| `u8` | 1 byte |
| `u16` | 2 bytes little-endian |
| `u32` | 4 bytes LE |
| `u64` | 8 bytes LE |
| `i8..i64` | same size as u, two's complement |
| `f32` | 4 bytes IEEE 754 LE |
| `f64` | 8 bytes IEEE 754 LE |
| `bool` | 1 byte: 0x00 or 0x01 |
| `&str` | u32 length (LE) + UTF-8 bytes |
| `&[u8]` | u32 length (LE) + bytes |
| `Option<T>` | 0x00 (None) or 0x01 + encoded T |
| `Vec<T>` | u32 length (LE) + N encoded elements |
| struct | encoded fields in order (no delimiters) |
| enum | u16 variant index (LE) + encoded payload |

### Compatibility rules

**Never break binary compatibility within a SCHEMA_HASH version.** To change a schema:
1. Create a new struct version (e.g., `UserV2`)
2. It will have a different `SCHEMA_HASH`
3. Add migration: `UserV2::from_v1(v1: UserV1) -> UserV2`
4. Keep `UserV1` decode support for old files

---

## Adding a New Format

Implement the `Format` trait in a new file `src/codec/myformat.rs`:

```rust
pub struct MyFormat;

impl fastserial::Format for MyFormat {
    fn write_null(w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(b"nil")
    }

    fn write_bool(v: bool, w: &mut impl WriteBuffer) -> Result<(), Error> {
        w.write_bytes(if v { b"yes" } else { b"no" })
    }

    fn write_u64(v: u64, w: &mut impl WriteBuffer) -> Result<(), Error> {
        let mut buf = itoa::Buffer::new();
        w.write_bytes(buf.format(v).as_bytes())
    }

    fn write_str(v: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
        // your format's string encoding
        todo!()
    }

    fn begin_object(n_fields: usize, w: &mut impl WriteBuffer) -> Result<(), Error> {
        todo!()
    }

    // ... implement all required methods
}
```

Then expose it:
```rust
// src/lib.rs
#[cfg(feature = "myformat")]
pub mod myformat {
    pub use crate::codec::myformat::MyFormat;
    pub fn encode<T: Encode>(val: &T) -> Result<Vec<u8>, Error> {
        let mut buf = Vec::new();
        val.encode_with_format::<MyFormat, _>(&mut buf)?;
        Ok(buf)
    }
}
```
