---
name: fastserial/encode
description: >
  How to implement Encode for a new type, or modify existing encode logic.
  Covers: struct encode, enum encode, custom field names, flatten, skip.
parent-skill: SKILL.md
---

# Encode Skill

## Decision Tree

```
Is the type a primitive (u8..u64, f32, f64, bool, &str, &[u8])?
  YES → impl Encode directly in src/codec/json.rs (see Primitive Encode)
  NO  →
    Is it a struct or enum the user defines?
      YES → Use #[derive(Encode)] — the proc-macro handles it (see Derive)
      NO  →
        Is it a std type (Option<T>, Vec<T>, HashMap)?
          YES → impl Encode in src/lib.rs blanket impls section (see Blanket)
```

---

## Primitive Encode

Add to `src/codec/json.rs`:

```rust
/// Encodes a u32 as decimal ASCII digits.
/// No allocation — writes directly to `w`.
#[inline(always)]
pub fn write_u32(v: u32, w: &mut impl WriteBuffer) -> Result<(), Error> {
    let mut buf = itoa::Buffer::new();   // stack-allocated, from `itoa` crate
    w.write_bytes(buf.format(v).as_bytes())
}
```

Then add the `Encode` impl in `src/lib.rs`:

```rust
impl Encode for u32 {
    const SCHEMA_HASH: u64 = hash_of_type_name("u32");

    #[inline(always)]
    fn encode<W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        crate::codec::json::write_u32(*self, w)
    }

    #[inline(always)]
    fn encode_with_format<F: Format, W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::write_u64(*self as u64, w)  // promote to u64 for format genericity
    }
}
```

**Do this for every numeric primitive.** The pattern is identical.

---

## Struct Encode via `#[derive(Encode)]`

The proc-macro generates the impl. Users just write:

```rust
#[derive(Encode)]
struct Payload<'a> {
    event_id: u64,
    name:     &'a str,
    tags:     Vec<&'a str>,
}
```

### What the proc-macro generates (src: fastserial-derive/src/encode.rs)

```rust
impl<'a> fastserial::Encode for Payload<'a> {
    const SCHEMA_HASH: u64 = /* compile-time hash */;

    #[inline(always)]
    fn encode<W>(&self, w: &mut W) -> Result<(), fastserial::Error>
    where W: fastserial::io::WriteBuffer
    {
        w.write_bytes(b"{")?;
        w.write_bytes(b"\"event_id\":")?;
        fastserial::codec::json::write_u64(self.event_id, w)?;
        w.write_bytes(b",\"name\":")?;
        fastserial::codec::json::write_str(self.name, w)?;
        w.write_bytes(b",\"tags\":")?;
        {
            w.write_byte(b'[')?;
            for (i, item) in self.tags.iter().enumerate() {
                if i > 0 { w.write_byte(b',')?; }
                fastserial::codec::json::write_str(item, w)?;
            }
            w.write_byte(b']')?;
        }
        w.write_byte(b'}')?;
        Ok(())
    }
    // encode_with_format<F> also generated (format-generic path)
}
```

Key points:
- Field key bytes like `b"\"event_id\":"` are **compile-time constants in `.rodata`**
- `write_u64`, `write_str` are `#[inline(always)]` — compiler fully inlines
- Vec is unrolled into a manual loop with comma logic — no iterator overhead

---

## Attributes in Detail

### `#[serial(rename = "key")]`

```rust
#[derive(Encode)]
struct Req {
    #[serial(rename = "userId")]
    user_id: u64,  // JSON key is "userId", not "user_id"
}
```

In proc-macro (`encode.rs`), when emitting the field key literal:
```rust
let key_bytes = if let Some(rename) = &field.attrs.rename {
    format!("\"{}\":", rename)
} else {
    format!("\"{}\":", field.ident)
};
// emits: w.write_bytes(b"\"userId\":")?;
```

### `#[serial(skip)]`

Field is completely excluded from encode output. Used for cache fields, computed fields.

```rust
#[derive(Encode)]
struct User {
    id: u64,
    #[serial(skip)]
    _computed: Option<String>,  // never written
}
```

### `#[serial(flatten)]`

Inlines a nested struct's fields at the current level:

```rust
#[derive(Encode)]
struct Request {
    method: &'static str,
    #[serial(flatten)]
    auth: AuthHeaders,  // auth's fields appear at the same JSON level
}

#[derive(Encode)]
struct AuthHeaders {
    token: String,
    expires: u64,
}

// Output: {"method":"GET","token":"abc","expires":9999}
// NOT:    {"method":"GET","auth":{"token":"abc","expires":9999}}
```

### `#[serial(skip_if = "path::to::fn")]`

Conditionally skip a field:

```rust
#[derive(Encode)]
struct Config {
    #[serial(skip_if = "Option::is_none")]
    debug_mode: Option<bool>,  // omitted when None
}
```

---

## Enum Encode

```rust
#[derive(Encode)]
#[serial(tag = "type")]
enum Command {
    Start { delay_ms: u64 },
    Stop,
    Retry { count: u8, backoff_ms: u64 },
}
```

Generated for `Command::Start { delay_ms: 100 }`:
```json
{"type":"Start","delay_ms":100}
```

Generated for `Command::Stop`:
```json
{"type":"Stop"}
```

### Tagging strategies

| Attribute | Output for `Start { delay_ms: 100 }` |
|-----------|--------------------------------------|
| `#[serial(tag = "type")]` | `{"type":"Start","delay_ms":100}` |
| `#[serial(tag = "t", content = "d")]` | `{"t":"Start","d":{"delay_ms":100}}` |
| `#[serial(untagged)]` | `{"delay_ms":100}` |
| (none — default) | `{"Start":{"delay_ms":100}}` |

---

## Blanket Impls (src/lib.rs)

For `Option<T>`, `Vec<T>`, etc.:

```rust
impl<T: Encode> Encode for Option<T> {
    const SCHEMA_HASH: u64 = 0; // not used for container types

    #[inline(always)]
    fn encode<W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        match self {
            None    => w.write_bytes(b"null"),
            Some(v) => v.encode(w),
        }
    }

    #[inline(always)]
    fn encode_with_format<F: Format, W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        match self {
            None    => F::write_null(w),
            Some(v) => v.encode_with_format::<F, W>(w),
        }
    }
}

impl<T: Encode> Encode for Vec<T> {
    const SCHEMA_HASH: u64 = 0;

    #[inline(always)]
    fn encode<W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        w.write_byte(b'[')?;
        for (i, item) in self.iter().enumerate() {
            if i > 0 { w.write_byte(b',')?; }
            item.encode(w)?;
        }
        w.write_byte(b']')
    }

    fn encode_with_format<F: Format, W: WriteBuffer>(&self, w: &mut W) -> Result<(), Error> {
        F::begin_array(self.len(), w)?;
        for item in self {
            item.encode_with_format::<F, W>(w)?;
        }
        F::end_array(w)
    }
}
```

---

## Testing Encode

Every encode impl needs a roundtrip test:

```rust
#[test]
fn encode_payload() {
    let p = Payload { event_id: 42, name: "login", tags: vec!["auth"] };
    let json = fastserial::json::encode(&p).unwrap();
    assert_eq!(json, br#"{"event_id":42,"name":"login","tags":["auth"]}"#);
}

#[test]
fn encode_into_fixed_buffer() {
    let p = Payload { event_id: 1, name: "x", tags: vec![] };
    let mut buf = [0u8; 64];
    let n = fastserial::json::encode_into(&p, &mut buf[..]).unwrap();
    assert_eq!(&buf[..n], br#"{"event_id":1,"name":"x","tags":[]}"#);
}

#[test]
fn encode_option_none() {
    let v: Option<u64> = None;
    let json = fastserial::json::encode(&v).unwrap();
    assert_eq!(json, b"null");
}
```
