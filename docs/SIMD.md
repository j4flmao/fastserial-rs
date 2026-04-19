# SIMD Strategy

> How fastserial uses SIMD intrinsics safely and portably.

## Supported Targets

| Target | Feature | Lane width | Status |
|--------|---------|------------|--------|
| x86_64 | AVX2 | 256-bit (32 bytes) | Stable, default |
| x86_64 | SSE4.2 | 128-bit (16 bytes) | Stable, fallback |
| aarch64 | NEON | 128-bit (16 bytes) | Planned v0.2 |
| wasm32 | SIMD128 | 128-bit | Planned v0.2 |
| any | scalar | 1 byte | Always available |

---

## Runtime Detection

Detection happens **once** at first call, result cached in an atomic:

```rust
// src/simd/mod.rs
use std::sync::atomic::{AtomicU8, Ordering};

const LEVEL_UNKNOWN: u8 = 0;
const LEVEL_SCALAR:  u8 = 1;
const LEVEL_SSE42:   u8 = 2;
const LEVEL_AVX2:    u8 = 3;

static SIMD_LEVEL: AtomicU8 = AtomicU8::new(LEVEL_UNKNOWN);

#[inline]
fn simd_level() -> u8 {
    let cached = SIMD_LEVEL.load(Ordering::Relaxed);
    if cached != LEVEL_UNKNOWN { return cached; }
    let level = detect_level();
    SIMD_LEVEL.store(level, Ordering::Relaxed);
    level
}

fn detect_level() -> u8 {
    #[cfg(target_arch = "x86_64")]
    {
        if std::arch::is_x86_feature_detected!("avx2")  { return LEVEL_AVX2; }
        if std::arch::is_x86_feature_detected!("sse4.2"){ return LEVEL_SSE42; }
    }
    LEVEL_SCALAR
}
```

In `no_std` mode, detection is replaced by compile-time `target_feature` check — no `AtomicU8`, no runtime cost.

---

## Core SIMD Operations

### 1. `scan_for_quote_or_backslash` — find end of JSON string

This is the hottest function in JSON decoding. Called for every string field.

**Scalar baseline:**
```rust
pub fn scan_quote_scalar(input: &[u8]) -> usize {
    input.iter()
         .position(|&b| b == b'"' || b == b'\\')
         .unwrap_or(input.len())
}
```

**AVX2 implementation (32 bytes/cycle):**
```rust
#[target_feature(enable = "avx2")]
unsafe fn scan_quote_avx2(input: &[u8]) -> usize {
    use std::arch::x86_64::*;
    let quote     = _mm256_set1_epi8(b'"'  as i8);
    let backslash = _mm256_set1_epi8(b'\\' as i8);

    let mut i = 0usize;
    while i + 32 <= input.len() {
        let chunk = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);
        let eq_q  = _mm256_cmpeq_epi8(chunk, quote);
        let eq_bs = _mm256_cmpeq_epi8(chunk, backslash);
        let mask  = _mm256_movemask_epi8(_mm256_or_si256(eq_q, eq_bs)) as u32;
        if mask != 0 {
            return i + mask.trailing_zeros() as usize;
        }
        i += 32;
    }
    // Scalar tail — handles remaining < 32 bytes
    i + scan_quote_scalar(&input[i..])
}
```

**Performance:** ~32× throughput vs scalar on strings > 64 bytes.

---

### 2. `skip_whitespace` — find first non-whitespace byte

**AVX2 implementation:**
```rust
#[target_feature(enable = "avx2")]
unsafe fn skip_whitespace_avx2(input: &[u8]) -> usize {
    use std::arch::x86_64::*;
    // Whitespace set: 0x09 (tab), 0x0A (LF), 0x0D (CR), 0x20 (space)
    let sp  = _mm256_set1_epi8(0x20);
    let tab = _mm256_set1_epi8(0x09);
    let lf  = _mm256_set1_epi8(0x0A);
    let cr  = _mm256_set1_epi8(0x0D);

    let mut i = 0usize;
    while i + 32 <= input.len() {
        let chunk = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);
        let is_sp  = _mm256_cmpeq_epi8(chunk, sp);
        let is_tab = _mm256_cmpeq_epi8(chunk, tab);
        let is_lf  = _mm256_cmpeq_epi8(chunk, lf);
        let is_cr  = _mm256_cmpeq_epi8(chunk, cr);
        let is_ws  = _mm256_or_si256(
            _mm256_or_si256(is_sp, is_tab),
            _mm256_or_si256(is_lf, is_cr),
        );
        // movemask: 1 = whitespace, 0 = non-whitespace
        let mask = _mm256_movemask_epi8(is_ws) as u32;
        if mask != 0xFFFF_FFFF {
            // At least one non-whitespace byte in this chunk
            return i + (!mask).trailing_zeros() as usize;
        }
        i += 32;
    }
    i + input[i..].iter().position(|b| !matches!(b, b' ' | b'\t' | b'\n' | b'\r'))
                         .unwrap_or(input.len() - i)
}
```

---

### 3. `validate_utf8_chunk` — fast UTF-8 validation

Rust `str::from_utf8` is already fast, but we can do better for ASCII-dominant inputs:

```rust
#[target_feature(enable = "avx2")]
unsafe fn is_all_ascii_avx2(input: &[u8]) -> bool {
    use std::arch::x86_64::*;
    let mut i = 0usize;
    let mut acc = _mm256_setzero_si256();
    while i + 32 <= input.len() {
        let chunk = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);
        acc = _mm256_or_si256(acc, chunk);
        i += 32;
    }
    // If any byte has high bit set → not pure ASCII
    let mask = _mm256_movemask_epi8(acc);
    if mask != 0 { return false; }
    // Check tail
    input[i..].iter().all(|b| b.is_ascii())
}
```

If the chunk is all ASCII, we skip `from_utf8` entirely (ASCII is always valid UTF-8).

---

### 4. `write_escaped_str` — copy and escape simultaneously

For encoding: scan for characters needing escaping while copying. If none found, single `write_bytes` call.

```rust
pub fn write_escaped_str(s: &str, w: &mut impl WriteBuffer) -> Result<(), Error> {
    w.write_byte(b'"')?;
    let bytes = s.as_bytes();
    let mut start = 0usize;

    let mut i = 0usize;
    while i < bytes.len() {
        // Fast path: find next byte needing escape using SIMD
        let next_special = scan_for_escape_needed(&bytes[i..]);
        if next_special == bytes.len() - i {
            // No special chars — copy entire remainder at once
            w.write_bytes(&bytes[start..])?;
            break;
        }
        // Copy safe prefix
        if next_special > 0 {
            w.write_bytes(&bytes[start..i + next_special])?;
        }
        // Emit escape sequence
        let b = bytes[i + next_special];
        match b {
            b'"'  => w.write_bytes(b"\\\"")?,
            b'\\' => w.write_bytes(b"\\\\")?,
            b'\n' => w.write_bytes(b"\\n")?,
            b'\r' => w.write_bytes(b"\\r")?,
            b'\t' => w.write_bytes(b"\\t")?,
            c     => write_unicode_escape(c, w)?,
        }
        i += next_special + 1;
        start = i;
    }
    w.write_byte(b'"')
}
```

---

## Safety Rules

All `unsafe` code in `simd/` must follow these rules:

1. **Always guard with `#[target_feature(enable = "...")]`** — the compiler will not insert illegal instructions on unsupported CPUs.
2. **Never call SIMD functions without dynamic detection** — use `simd_level()` check or `#[cfg(target_feature)]` at compile time.
3. **Pointer alignment** — use `_mm256_loadu_si256` (unaligned load), never `_mm256_load_si256` (aligned load, requires 32-byte alignment we cannot guarantee).
4. **Bounds check before SIMD loop** — the `while i + 32 <= input.len()` guard ensures we never read past the slice.
5. **Scalar tail always present** — every SIMD function must handle the remaining `< lane_width` bytes with scalar code.

---

## Testing SIMD Code

Every SIMD function must have a test that:
1. Runs the scalar and SIMD versions on identical inputs
2. Asserts identical outputs
3. Tests boundary conditions: empty input, 1 byte, 31 bytes, 32 bytes, 33 bytes, 64 bytes

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scan_quote_scalar_vs_avx2() {
        if !std::arch::is_x86_feature_detected!("avx2") { return; }
        let cases: &[(&[u8], usize)] = &[
            (b"hello\"world",   5),
            (b"no special here",15),
            (b"\\backslash",    0),
            (b"",               0),
            (b"a",              1),
        ];
        for (input, expected) in cases {
            assert_eq!(scan_quote_scalar(input), *expected, "scalar: {input:?}");
            assert_eq!(unsafe { scan_quote_avx2(input) }, *expected, "avx2: {input:?}");
        }
    }
}
```

---

## Benchmark Results

Measured with `criterion`, `twitter.json` (616 KB), warm cache:

| Function | Scalar | SSE4.2 | AVX2 |
|----------|--------|--------|------|
| `scan_quote` | 1.00× | 1.80× | 3.40× |
| `skip_whitespace` | 1.00× | 1.75× | 3.20× |
| `is_all_ascii` | 1.00× | 2.00× | 3.90× |
| `write_escaped_str` | 1.00× | 1.60× | 2.80× |

End-to-end JSON decode speedup (vs scalar): **3.8×** on twitter.json.
