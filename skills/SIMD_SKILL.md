---
name: fastserial/simd
description: >
  How to add a new SIMD routine or modify existing ones.
  Covers: AVX2 / SSE4.2 / scalar pattern, safety rules, dispatch, testing.
parent-skill: SKILL.md
---

# SIMD Skill

## The Golden Rule

**Every SIMD function must have a scalar equivalent that produces identical output.**

Always write scalar first. Then AVX2. Then SSE4.2 if needed. The scalar version is the specification.

---

## File Locations

```
src/simd/
  mod.rs      ← public dispatch function goes here
  scalar.rs   ← scalar (portable) implementation
  sse42.rs    ← SSE4.2 implementation (x86_64)
  avx2.rs     ← AVX2 implementation (x86_64)
```

---

## Step 1 — Write Scalar Version (`scalar.rs`)

```rust
// src/simd/scalar.rs

/// Find the first byte that is a quote (0x22) or backslash (0x5C).
/// Returns the index, or `input.len()` if none found.
pub fn scan_quote_or_backslash(input: &[u8]) -> usize {
    input.iter()
         .position(|&b| b == b'"' || b == b'\\')
         .unwrap_or(input.len())
}
```

Scalar must be `pub` — it's used directly when AVX2/SSE4.2 are not available, AND as reference in tests.

---

## Step 2 — Write AVX2 Version (`avx2.rs`)

Template to follow exactly:

```rust
// src/simd/avx2.rs
use std::arch::x86_64::*;

/// AVX2 version: processes 32 bytes per iteration.
///
/// # Safety
/// Caller MUST have verified AVX2 is available via `is_x86_feature_detected!("avx2")`.
/// This function MUST be called only from `mod.rs` dispatch after that check.
#[target_feature(enable = "avx2")]
pub unsafe fn scan_quote_or_backslash(input: &[u8]) -> usize {
    let quote     = _mm256_set1_epi8(b'"'  as i8);
    let backslash = _mm256_set1_epi8(b'\\' as i8);

    let mut i = 0usize;

    // Process 32 bytes per iteration
    while i + 32 <= input.len() {
        // SAFETY: i + 32 <= input.len() guarantees in-bounds
        let chunk = _mm256_loadu_si256(input.as_ptr().add(i) as *const __m256i);

        let eq_q  = _mm256_cmpeq_epi8(chunk, quote);
        let eq_bs = _mm256_cmpeq_epi8(chunk, backslash);
        let combined = _mm256_or_si256(eq_q, eq_bs);

        // movemask: bit N = 1 if byte N matched
        let mask = _mm256_movemask_epi8(combined) as u32;

        if mask != 0 {
            // trailing_zeros gives index of first match within the 32-byte chunk
            return i + mask.trailing_zeros() as usize;
        }

        i += 32;
    }

    // Scalar tail: handle remaining < 32 bytes
    i + super::scalar::scan_quote_or_backslash(&input[i..])
}
```

**Required checklist for every AVX2 function:**
- [ ] `#[target_feature(enable = "avx2")]` on the function
- [ ] `pub unsafe fn` — not safe, callers must check
- [ ] `// SAFETY:` comment on every `unsafe` operation
- [ ] Guard `i + 32 <= input.len()` before `_mm256_loadu_si256`
- [ ] Use `_mm256_loadu_si256` (unaligned) — never `_mm256_load_si256`
- [ ] Scalar tail at the end

---

## Step 3 — Write SSE4.2 Version (`sse42.rs`)

Same pattern, 16-byte lanes:

```rust
// src/simd/sse42.rs
use std::arch::x86_64::*;

#[target_feature(enable = "sse4.2")]
pub unsafe fn scan_quote_or_backslash(input: &[u8]) -> usize {
    let quote     = _mm_set1_epi8(b'"'  as i8);
    let backslash = _mm_set1_epi8(b'\\' as i8);

    let mut i = 0usize;
    while i + 16 <= input.len() {
        // SAFETY: i + 16 <= input.len()
        let chunk = _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i);
        let eq_q  = _mm_cmpeq_epi8(chunk, quote);
        let eq_bs = _mm_cmpeq_epi8(chunk, backslash);
        let mask  = _mm_movemask_epi8(_mm_or_si128(eq_q, eq_bs)) as u32;
        if mask != 0 {
            return i + mask.trailing_zeros() as usize;
        }
        i += 16;
    }
    i + super::scalar::scan_quote_or_backslash(&input[i..])
}
```

---

## Step 4 — Add Dispatch (`mod.rs`)

```rust
// src/simd/mod.rs

mod scalar;
mod sse42;
mod avx2;

// Re-export the public dispatch function (safe API surface)

/// Find first quote or backslash in `input`.
/// Automatically uses AVX2 > SSE4.2 > scalar based on CPU.
#[inline]
pub fn scan_quote_or_backslash(input: &[u8]) -> usize {
    match simd_level() {
        LEVEL_AVX2  => unsafe { avx2::scan_quote_or_backslash(input) },
        LEVEL_SSE42 => unsafe { sse42::scan_quote_or_backslash(input) },
        _           => scalar::scan_quote_or_backslash(input),
    }
}
```

This is the **only** function external code should call. The `unsafe` is contained inside `mod.rs`.

---

## Step 5 — Write Tests

**Mandatory test structure for every SIMD function:**

```rust
// src/simd/mod.rs (at the bottom)
#[cfg(test)]
mod tests {
    use super::*;

    // Helper: run scalar AND simd, assert same result
    fn check(input: &[u8], expected: usize) {
        assert_eq!(
            scalar::scan_quote_or_backslash(input),
            expected,
            "scalar failed on {input:?}"
        );
        if std::arch::is_x86_feature_detected!("avx2") {
            assert_eq!(
                unsafe { avx2::scan_quote_or_backslash(input) },
                expected,
                "avx2 failed on {input:?}"
            );
        }
        if std::arch::is_x86_feature_detected!("sse4.2") {
            assert_eq!(
                unsafe { sse42::scan_quote_or_backslash(input) },
                expected,
                "sse42 failed on {input:?}"
            );
        }
    }

    #[test]
    fn empty_input() {
        check(b"", 0);
    }

    #[test]
    fn no_special_chars() {
        check(b"hello world", 11);
    }

    #[test]
    fn quote_at_start() {
        check(b"\"hello", 0);
    }

    #[test]
    fn backslash_at_start() {
        check(b"\\hello", 0);
    }

    #[test]
    fn quote_in_middle() {
        check(b"hello\"world", 5);
    }

    // Boundary: exactly 31 bytes before the quote (tests scalar tail of AVX2)
    #[test]
    fn boundary_31_plus_quote() {
        let mut input = vec![b'a'; 31];
        input.push(b'"');
        check(&input, 31);
    }

    // Boundary: exactly 32 bytes before the quote (tests second AVX2 iteration)
    #[test]
    fn boundary_32_plus_quote() {
        let mut input = vec![b'a'; 32];
        input.push(b'"');
        check(&input, 32);
    }

    // Boundary: 33 bytes before quote (tests scalar tail after first full AVX2 chunk)
    #[test]
    fn boundary_33_plus_quote() {
        let mut input = vec![b'a'; 33];
        input.push(b'"');
        check(&input, 33);
    }

    // Large input — quote at the very end
    #[test]
    fn large_input_quote_at_end() {
        let mut input = vec![b'x'; 1000];
        input.push(b'"');
        check(&input, 1000);
    }

    // No match in large input
    #[test]
    fn large_input_no_match() {
        let input = vec![b'x'; 1000];
        check(&input, 1000);
    }
}
```

---

## Common Mistakes

| Mistake | Consequence | Fix |
|---------|-------------|-----|
| Using `_mm256_load_si256` | SIGBUS on unaligned input | Use `_mm256_loadu_si256` always |
| Missing scalar tail | Wrong results for inputs not divisible by 32 | Always add `i + super::scalar::fn(&input[i..])` after loop |
| Calling AVX2 fn without detection | SIGILL on non-AVX2 CPU | Only call from `mod.rs` dispatch, never directly |
| Missing `#[target_feature]` | UB: compiler may generate illegal instructions | Required on every AVX2/SSE4.2 fn |
| Wrong mask interpretation | Off-by-one in position | `trailing_zeros()` = first set bit = first match |
| `as u32` on `_mm256_movemask_epi8` | Correct — it returns i32, cast to u32 for bit ops | Always cast with `as u32` |

---

## Performance Verification

After adding a SIMD function, run:

```bash
cargo bench -- simd/scan_quote
```

Expected speedups vs scalar on large inputs:
- SSE4.2: 1.6–2.0×
- AVX2: 2.8–4.0×

If speedup is less than 1.5× for SSE4.2 or 2.5× for AVX2, the function likely has:
- Excessive overhead in the scalar tail (input size too small for benchmark)
- Missed optimization (check `cargo asm` output)
- Wrong intrinsic choice
