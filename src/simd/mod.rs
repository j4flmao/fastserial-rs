#[cfg(target_arch = "x86_64")]
pub mod avx2;
pub mod scalar;
#[cfg(target_arch = "x86_64")]
pub mod sse42;

use core::sync::atomic::{AtomicU8, Ordering};

const LEVEL_UNKNOWN: u8 = 0;
const LEVEL_SCALAR: u8 = 1;
const LEVEL_SSE42: u8 = 2;
const LEVEL_AVX2: u8 = 3;

static SIMD_LEVEL: AtomicU8 = AtomicU8::new(LEVEL_UNKNOWN);

#[inline]
fn simd_level() -> u8 {
    let cached = SIMD_LEVEL.load(Ordering::Relaxed);
    if cached != LEVEL_UNKNOWN {
        return cached;
    }
    let level = detect_level();
    SIMD_LEVEL.store(level, Ordering::Relaxed);
    level
}

/// Detects the highest available SIMD instruction set on the current CPU.
///
/// Returns a level constant (LEVEL_SCALAR, LEVEL_SSE42, or LEVEL_AVX2).
#[cfg(target_arch = "x86_64")]
#[cfg(feature = "std")]
fn detect_level() -> u8 {
    if std::arch::is_x86_feature_detected!("avx2") {
        return LEVEL_AVX2;
    }
    if std::arch::is_x86_feature_detected!("sse4.2") {
        return LEVEL_SSE42;
    }
    LEVEL_SCALAR
}

/// Fallback detection for non-x86 platforms or no-std environments.
///
/// Always returns LEVEL_SCALAR as specialized SIMD implementations are
/// currently only available for x86_64.
#[cfg(not(all(target_arch = "x86_64", feature = "std")))]
fn detect_level() -> u8 {
    LEVEL_SCALAR
}

#[inline]
pub fn scan_quote_or_backslash(input: &[u8]) -> usize {
    match simd_level() {
        #[cfg(target_arch = "x86_64")]
        LEVEL_AVX2 => unsafe { avx2::scan_quote_or_backslash(input) },
        #[cfg(target_arch = "x86_64")]
        LEVEL_SSE42 => unsafe { sse42::scan_quote_or_backslash(input) },
        _ => scalar::scan_quote_or_backslash(input),
    }
}

#[inline]
pub fn scan_escape_chars(input: &[u8]) -> usize {
    match simd_level() {
        #[cfg(target_arch = "x86_64")]
        LEVEL_AVX2 => unsafe { avx2::scan_escape_chars(input) },
        #[cfg(target_arch = "x86_64")]
        LEVEL_SSE42 => unsafe { sse42::scan_escape_chars(input) },
        _ => scalar::scan_escape_chars(input),
    }
}

#[inline]
pub fn skip_whitespace(input: &[u8]) -> usize {
    match simd_level() {
        #[cfg(target_arch = "x86_64")]
        LEVEL_AVX2 => unsafe { avx2::skip_whitespace(input) },
        #[cfg(target_arch = "x86_64")]
        LEVEL_SSE42 => unsafe { sse42::skip_whitespace(input) },
        _ => scalar::skip_whitespace(input),
    }
}

#[inline]
pub fn is_all_ascii(input: &[u8]) -> bool {
    match simd_level() {
        #[cfg(target_arch = "x86_64")]
        LEVEL_AVX2 => unsafe { avx2::is_all_ascii(input) },
        #[cfg(target_arch = "x86_64")]
        LEVEL_SSE42 => unsafe { sse42::is_all_ascii(input) },
        _ => scalar::is_all_ascii(input),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    #[test]
    fn scan_quote_empty() {
        assert_eq!(scalar::scan_quote_or_backslash(b""), 0);
        assert_eq!(scan_quote_or_backslash(b""), 0);
    }

    #[test]
    fn scan_quote_no_match() {
        let input = b"hello world";
        assert_eq!(scalar::scan_quote_or_backslash(input), input.len());
        assert_eq!(scan_quote_or_backslash(input), input.len());
    }

    #[test]
    fn scan_quote_at_start() {
        let input = b"\"hello";
        assert_eq!(scalar::scan_quote_or_backslash(input), 0);
        assert_eq!(scan_quote_or_backslash(input), 0);
    }

    #[test]
    fn scan_quote_in_middle() {
        let input = b"hello\"world";
        assert_eq!(scalar::scan_quote_or_backslash(input), 5);
        assert_eq!(scan_quote_or_backslash(input), 5);
    }

    #[test]
    fn scan_backslash_in_middle() {
        let input = b"hello\\world";
        assert_eq!(scalar::scan_quote_or_backslash(input), 5);
        assert_eq!(scan_quote_or_backslash(input), 5);
    }

    #[test]
    fn boundary_31() {
        let mut input = vec![b'a'; 31];
        input.push(b'"');
        assert_eq!(scalar::scan_quote_or_backslash(&input), 31);
        assert_eq!(scan_quote_or_backslash(&input), 31);
    }

    #[test]
    fn boundary_32() {
        let mut input = vec![b'a'; 32];
        input.push(b'"');
        assert_eq!(scalar::scan_quote_or_backslash(&input), 32);
        assert_eq!(scan_quote_or_backslash(&input), 32);
    }

    #[test]
    fn boundary_33() {
        let mut input = vec![b'a'; 33];
        input.push(b'"');
        assert_eq!(scalar::scan_quote_or_backslash(&input), 33);
        assert_eq!(scan_quote_or_backslash(&input), 33);
    }

    #[test]
    fn large_input() {
        let mut input = vec![b'x'; 1000];
        input.push(b'"');
        assert_eq!(scalar::scan_quote_or_backslash(&input), 1000);
        assert_eq!(scan_quote_or_backslash(&input), 1000);
    }
}
