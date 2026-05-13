//! # NEON SIMD Implementation
//!
//! ARM NEON implementations for 64-bit AArch64 platforms.
//!
//! NEON is part of the AArch64 baseline ABI, so we don't need a runtime
//! feature check — every aarch64 CPU running Rust has it.

#![cfg(target_arch = "aarch64")]
#![allow(unused_imports)]

use core::arch::aarch64::*;

/// Compresses a 128-bit comparison mask (16 lanes of 0x00/0xFF) into a 16-bit
/// bitmask, one bit per lane. This is the NEON equivalent of x86's
/// `_mm_movemask_epi8` and avoids the slow scalar-extraction pattern.
#[inline(always)]
unsafe fn movemask_u8(mask: uint8x16_t) -> u32 {
    // shift-right-narrow: each pair of bytes (16 lanes → 8 lanes of u8)
    // collapses to a 4-bit nibble per byte. Packed result fits in 64 bits.
    let nibbles = vshrn_n_u16(vreinterpretq_u16_u8(mask), 4);
    let bits = vget_lane_u64(vreinterpret_u64_u8(nibbles), 0);
    bits as u32
}

/// Scans for the first quote or backslash using NEON 128-bit instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn scan_quote_or_backslash(input: &[u8]) -> usize {
    let quote = vdupq_n_u8(b'"');
    let backslash = vdupq_n_u8(b'\\');

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let wide = vld1q_u8(input.as_ptr().add(i));
        let combined = vorrq_u8(vceqq_u8(wide, quote), vceqq_u8(wide, backslash));
        let mask = movemask_u8(combined);
        if mask != 0 {
            // Each lane contributes 4 bits (nibble), so divide by 4.
            return i + (mask.trailing_zeros() as usize) / 4;
        }
        i += 16;
    }
    i + super::scalar::scan_quote_or_backslash(&input[i..])
}

/// Scans for any JSON-escape character using NEON.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn scan_escape_chars(input: &[u8]) -> usize {
    let quote = vdupq_n_u8(b'"');
    let backslash = vdupq_n_u8(b'\\');
    let lf = vdupq_n_u8(b'\n');
    let cr = vdupq_n_u8(b'\r');
    let tab = vdupq_n_u8(b'\t');

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let wide = vld1q_u8(input.as_ptr().add(i));
        let combined = vorrq_u8(
            vorrq_u8(
                vorrq_u8(vceqq_u8(wide, quote), vceqq_u8(wide, backslash)),
                vorrq_u8(vceqq_u8(wide, lf), vceqq_u8(wide, cr)),
            ),
            vceqq_u8(wide, tab),
        );
        let mask = movemask_u8(combined);
        if mask != 0 {
            return i + (mask.trailing_zeros() as usize) / 4;
        }
        i += 16;
    }
    i + super::scalar::scan_escape_chars(&input[i..])
}

/// Skips leading whitespace using NEON 128-bit instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn skip_whitespace(input: &[u8]) -> usize {
    let space = vdupq_n_u8(b' ');
    let tab = vdupq_n_u8(b'\t');
    let lf = vdupq_n_u8(b'\n');
    let cr = vdupq_n_u8(b'\r');

    let mut i = 0usize;
    while i + 16 <= input.len() {
        let wide = vld1q_u8(input.as_ptr().add(i));
        let is_ws = vorrq_u8(
            vorrq_u8(vceqq_u8(wide, space), vceqq_u8(wide, tab)),
            vorrq_u8(vceqq_u8(wide, lf), vceqq_u8(wide, cr)),
        );
        let mask = movemask_u8(is_ws);
        // movemask_u8 produces 4 bits per lane → 64 bits for 16 lanes when full.
        if mask != u32::MAX {
            return i + (!mask).trailing_zeros() as usize / 4;
        }
        i += 16;
    }
    i + super::scalar::skip_whitespace(&input[i..])
}

/// Checks if all bytes are ASCII using NEON instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn is_all_ascii(input: &[u8]) -> bool {
    let mut acc = vdupq_n_u8(0);
    let mut i = 0usize;
    while i + 16 <= input.len() {
        let wide = vld1q_u8(input.as_ptr().add(i));
        acc = vorrq_u8(acc, wide);
        i += 16;
    }
    if vmaxvq_u8(acc) >= 0x80 {
        return false;
    }
    input[i..].iter().all(|&b| b < 0x80)
}
