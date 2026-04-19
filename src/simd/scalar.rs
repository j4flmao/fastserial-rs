//! # Scalar SIMD Implementation
//!
//! Fallback implementations for platforms without SIMD support.

/// Scans for the first quote (`"`) or backslash (`\`) character.
///
/// # Arguments
/// * `input` - The byte slice to scan.
///
/// # Returns
/// The index of the first match, or input length if not found.
#[inline]
pub fn scan_quote_or_backslash(input: &[u8]) -> usize {
    input
        .iter()
        .position(|&b| b == b'"' || b == b'\\')
        .unwrap_or(input.len())
}

/// Skips leading whitespace characters.
///
/// Skips space (0x20), tab (0x09), newline (0x0A), and carriage return (0x0D).
///
/// # Arguments
/// * `input` - The byte slice to scan.
///
/// # Returns
/// The number of whitespace bytes to skip.
#[inline]
pub fn skip_whitespace(input: &[u8]) -> usize {
    input
        .iter()
        .position(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r')
        .unwrap_or(input.len())
}

/// Checks if all bytes are valid ASCII (value < 0x80).
///
/// # Arguments
/// * `input` - The byte slice to check.
///
/// # Returns
/// `true` if all bytes are ASCII.
#[inline]
pub fn is_all_ascii(input: &[u8]) -> bool {
    input.iter().all(|&b| b < 0x80)
}
