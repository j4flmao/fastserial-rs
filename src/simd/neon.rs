//! # NEON SIMD Implementation
//!
//! ARM NEON implementations for 64-bit AArch64 platforms.

#![cfg(target_arch = "aarch64")]
#![allow(unused_imports)]

use core::arch::aarch64::*;

/// Scans for the first quote or backslash using NEON 128-bit instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn scan_quote_or_backslash(input: &[u8]) -> usize {
    if input.len() < 16 {
        return input
            .iter()
            .position(|&b| b == b'"' || b == b'\\')
            .unwrap_or(input.len());
    }

    let quote = vdupq_n_u8(b'"');
    let backslash = vdupq_n_u8(b'\\');
    let chunks = input.chunks_exact(16);

    for (i, chunk) in chunks.enumerate() {
        let bytes: [u8; 16] = core::array::from_fn(|j| chunk[j]);
        let wide = vld1q_u8(bytes.as_ptr());

        let quote_mask = vceqq_u8(wide, quote);
        let backslash_mask = vceqq_u8(wide, backslash);

        let combined = vorrq_u8(quote_mask, backslash_mask);
        let movemask = vgetq_lane_u64(vreinterpretq_u64_u8(combined), 0) as u32;
        let high = vgetq_lane_u64(vreinterpretq_u64_u8(combined), 1) as u32;
        let full_mask = movemask | (high << 32);

        if full_mask != 0 {
            return i * 16 + full_mask.trailing_zeros() as usize;
        }
    }

    let remainder = input.len() % 16;
    if remainder > 0 {
        let start = input.len() - remainder;
        input[start..]
            .iter()
            .position(|&b| b == b'"' || b == b'\\')
            .unwrap_or(remainder)
            + start
    } else {
        input.len()
    }
}

/// Skips leading whitespace using NEON 128-bit instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn skip_whitespace(input: &[u8]) -> usize {
    if input.len() < 16 {
        return input
            .iter()
            .position(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r')
            .unwrap_or(input.len());
    }

    let space = vdupq_n_u8(b' ');
    let tab = vdupq_n_u8(b'\t');
    let newline = vdupq_n_u8(b'\n');
    let cr = vdupq_n_u8(b'\r');

    let chunks = input.chunks_exact(16);

    for (i, chunk) in chunks.enumerate() {
        let bytes: [u8; 16] = core::array::from_fn(|j| chunk[j]);
        let wide = vld1q_u8(bytes.as_ptr());

        let space_mask = vceqq_u8(wide, space);
        let tab_mask = vceqq_u8(wide, tab);
        let newline_mask = vceqq_u8(wide, newline);
        let cr_mask = vceqq_u8(wide, cr);

        let combined = vorrq_u8(
            vorrq_u8(space_mask, tab_mask),
            vorrq_u8(newline_mask, cr_mask),
        );
        let movemask = vgetq_lane_u64(vreinterpretq_u64_u8(combined), 0) as u32;
        let high = vgetq_lane_u64(vreinterpretq_u64_u8(combined), 1) as u32;
        let full_mask = movemask | (high << 32);

        let not_full = !full_mask;
        if not_full != 0 {
            return i * 16 + not_full.trailing_zeros() as usize;
        }
    }

    let remainder = input.len() % 16;
    if remainder > 0 {
        let start = input.len() - remainder;
        input[start..]
            .iter()
            .position(|&b| b != b' ' && b != b'\t' && b != b'\n' && b != b'\r')
            .unwrap_or(remainder)
            + start
    } else {
        input.len()
    }
}

/// Checks if all bytes are ASCII using NEON instructions.
///
/// # Safety
/// Must be called on aarch64 target.
#[inline(always)]
pub unsafe fn is_all_ascii(input: &[u8]) -> bool {
    if input.len() < 16 {
        return input.iter().all(|&b| b < 0x80);
    }

    let chunks = input.chunks_exact(16);

    for chunk in chunks {
        let bytes: [u8; 16] = core::array::from_fn(|j| chunk[j]);
        let wide = vld1q_u8(bytes.as_ptr());
        let high = vreinterpretq_u8_u64(vshrq_n_u64(vreinterpretq_u64_u8(wide), 7));

        if vgetq_lane_u64(vreinterpretq_u64_u8(high), 0) != 0 {
            return false;
        }
        if vgetq_lane_u64(vreinterpretq_u64_u8(high), 1) != 0 {
            return false;
        }
    }

    let remainder = input.len() % 16;
    if remainder > 0 {
        let start = input.len() - remainder;
        input[start..].iter().all(|&b| b < 0x80)
    } else {
        true
    }
}
