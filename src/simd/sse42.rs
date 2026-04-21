/// Scans the input for the first occurrence of a double quote (`"`) or a backslash (`\`)
/// using SSE4.2 SIMD instructions.
///
/// # Arguments
/// * `input` - The byte slice to scan.
///
/// # Returns
/// The index of the first quote or backslash, or the length of the input if none are found.
///
/// # Safety
/// This function is unsafe because:
/// 1. It uses SSE4.2 intrinsic instructions (`_mm_*`).
/// 2. The caller must ensure that the CPU supports SSE4.2 before calling this function.
#[target_feature(enable = "sse4.2")]
#[allow(unused_unsafe)]
pub unsafe fn scan_quote_or_backslash(input: &[u8]) -> usize {
    use core::arch::x86_64::*;
    let quote = unsafe { _mm_set1_epi8(b'"' as i8) };
    let backslash = unsafe { _mm_set1_epi8(b'\\' as i8) };

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let chunk = unsafe { _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i) };
        let eq_q = unsafe { _mm_cmpeq_epi8(chunk, quote) };
        let eq_bs = unsafe { _mm_cmpeq_epi8(chunk, backslash) };
        let mask = unsafe { _mm_movemask_epi8(_mm_or_si128(eq_q, eq_bs)) as u32 };
        if mask != 0 {
            return i + mask.trailing_zeros() as usize;
        }
        i += 16;
    }
    i + super::scalar::scan_quote_or_backslash(&input[i..])
}

/// Scans for escape characters using SSE4.2 SIMD.
///
/// # Safety
/// This function is unsafe because it uses SSE4.2 intrinsic instructions.
/// The caller must ensure the CPU supports SSE4.2.
#[target_feature(enable = "sse4.2")]
#[allow(unused_unsafe)]
pub unsafe fn scan_escape_chars(input: &[u8]) -> usize {
    use core::arch::x86_64::*;
    let quote = unsafe { _mm_set1_epi8(b'"' as i8) };
    let backslash = unsafe { _mm_set1_epi8(b'\\' as i8) };
    let lf = unsafe { _mm_set1_epi8(b'\n' as i8) };
    let cr = unsafe { _mm_set1_epi8(b'\r' as i8) };
    let tab = unsafe { _mm_set1_epi8(b'\t' as i8) };

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let chunk = unsafe { _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i) };
        let eq_q = unsafe { _mm_cmpeq_epi8(chunk, quote) };
        let eq_bs = unsafe { _mm_cmpeq_epi8(chunk, backslash) };
        let eq_lf = unsafe { _mm_cmpeq_epi8(chunk, lf) };
        let eq_cr = unsafe { _mm_cmpeq_epi8(chunk, cr) };
        let eq_tab = unsafe { _mm_cmpeq_epi8(chunk, tab) };
        let has_escape = unsafe {
            _mm_or_si128(
                _mm_or_si128(_mm_or_si128(eq_q, eq_bs), _mm_or_si128(eq_lf, eq_cr)),
                eq_tab,
            )
        };
        let mask = unsafe { _mm_movemask_epi8(has_escape) as u32 };
        if mask != 0 {
            return i + mask.trailing_zeros() as usize;
        }
        i += 16;
    }
    i + super::scalar::scan_escape_chars(&input[i..])
}

/// Skips all leading JSON whitespace characters (space, tab, newline, carriage return)
/// using SSE4.2 SIMD instructions.
///
/// # Arguments
/// * `input` - The byte slice to skip whitespace in.
///
/// # Returns
/// The number of leading whitespace bytes skipped.
///
/// # Safety
/// This function is unsafe because:
/// 1. It uses SSE4.2 intrinsic instructions (`_mm_*`).
/// 2. The caller must ensure that the CPU supports SSE4.2 before calling this function.
#[target_feature(enable = "sse4.2")]
#[allow(unused_unsafe)]
pub unsafe fn skip_whitespace(input: &[u8]) -> usize {
    use core::arch::x86_64::*;
    let sp = unsafe { _mm_set1_epi8(b' ' as i8) };
    let tab = unsafe { _mm_set1_epi8(b'\t' as i8) };
    let lf = unsafe { _mm_set1_epi8(b'\n' as i8) };
    let cr = unsafe { _mm_set1_epi8(b'\r' as i8) };

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let chunk = unsafe { _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i) };
        let is_sp = unsafe { _mm_cmpeq_epi8(chunk, sp) };
        let is_tab = unsafe { _mm_cmpeq_epi8(chunk, tab) };
        let is_lf = unsafe { _mm_cmpeq_epi8(chunk, lf) };
        let is_cr = unsafe { _mm_cmpeq_epi8(chunk, cr) };
        let is_ws =
            unsafe { _mm_or_si128(_mm_or_si128(is_sp, is_tab), _mm_or_si128(is_lf, is_cr)) };
        let mask = unsafe { _mm_movemask_epi8(is_ws) as u32 };
        if mask != 0xFFFF {
            return i + (!mask).trailing_zeros() as usize;
        }
        i += 16;
    }
    i + super::scalar::skip_whitespace(&input[i..])
}

/// Checks if all bytes in the input slice are valid ASCII characters
/// using SSE4.2 SIMD instructions.
///
/// # Arguments
/// * `input` - The byte slice to check.
///
/// # Returns
/// `true` if all bytes are ASCII, `false` otherwise.
///
/// # Safety
/// This function is unsafe because:
/// 1. It uses SSE4.2 intrinsic instructions (`_mm_*`).
/// 2. The caller must ensure that the CPU supports SSE4.2 before calling this function.
#[target_feature(enable = "sse4.2")]
#[allow(unused_unsafe)]
pub unsafe fn is_all_ascii(input: &[u8]) -> bool {
    use core::arch::x86_64::*;
    let mut i = 0usize;
    let mut acc = unsafe { _mm_setzero_si128() };
    while i + 16 <= input.len() {
        let chunk = unsafe { _mm_loadu_si128(input.as_ptr().add(i) as *const __m128i) };
        acc = unsafe { _mm_or_si128(acc, chunk) };
        i += 16;
    }
    let mask = unsafe { _mm_movemask_epi8(acc) };
    if mask != 0 {
        return false;
    }
    input[i..].iter().all(|b| b.is_ascii())
}
