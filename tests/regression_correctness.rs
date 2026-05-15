//! Regression tests for correctness bugs fixed in the v0.2 cleanup.
//!
//! Each test here pins a real bug that existed before the fix. If any of
//! these regress, real downstream code corrupts data silently.

use fastserial::Error;
use fastserial::json::decode;

// ============================================================================
// 1. Borrowed `&str` must not silently embed escape sequences.
//
// Before the fix, `&'de str` decode silently returned the raw bytes including
// `\"`, `\\`, `\n`, etc. — so `"a\"b"` decoded to a 4-char string `a\"b`
// instead of either erroring or unescaping. Now the decoder errors
// explicitly so users know to switch to `String` or `Cow<str>`.
// ============================================================================

#[test]
fn borrowed_str_rejects_escape_quote() {
    let input = br#""a\"b""#; // JSON for the 3-char string `a"b`
    let result: Result<&str, Error> = decode(input);
    assert!(matches!(result, Err(Error::EscapeInBorrowedString { .. })));
}

#[test]
fn borrowed_str_rejects_escape_backslash() {
    let input = br#""a\\b""#;
    let result: Result<&str, Error> = decode(input);
    assert!(matches!(result, Err(Error::EscapeInBorrowedString { .. })));
}

#[test]
fn borrowed_str_rejects_unicode_escape() {
    let input = br#""\u0041""#;
    let result: Result<&str, Error> = decode(input);
    assert!(matches!(result, Err(Error::EscapeInBorrowedString { .. })));
}

#[test]
fn borrowed_str_accepts_no_escape() {
    let input = br#""hello world""#;
    let s: &str = decode(input).unwrap();
    assert_eq!(s, "hello world");
}

#[test]
fn owned_string_unescapes_correctly() {
    // The owned-String path must keep working — it allocates and unescapes.
    let s: String = decode(br#""a\"b""#).unwrap();
    assert_eq!(s, "a\"b");
    let s: String = decode(br#""line1\nline2""#).unwrap();
    assert_eq!(s, "line1\nline2");
}

// ============================================================================
// 2. read_unsigned must reject overflow instead of silently wrapping.
// ============================================================================

#[test]
fn unsigned_overflow_errors_25_digits() {
    // 25-digit number > u64::MAX ≈ 1.8e19
    let input = b"1234567890123456789012345";
    let result: Result<u64, Error> = decode(input);
    assert!(matches!(
        result,
        Err(Error::NumberOverflow { type_name: "u64" })
    ));
}

#[test]
fn unsigned_overflow_errors_just_above_max() {
    // u64::MAX = 18446744073709551615; one more than that overflows.
    let input = b"18446744073709551616";
    let result: Result<u64, Error> = decode(input);
    assert!(matches!(
        result,
        Err(Error::NumberOverflow { type_name: "u64" })
    ));
}

#[test]
fn unsigned_max_value_works() {
    let v: u64 = decode(b"18446744073709551615").unwrap();
    assert_eq!(v, u64::MAX);
}

#[test]
fn unsigned_long_zeros_no_false_overflow() {
    // 30 leading zeros + a value — must NOT overflow because each `* 10` of
    // zero is still zero.
    let v: u64 = decode(b"000000000000000000000000000042").unwrap();
    assert_eq!(v, 42);
}

// ============================================================================
// 3. skip_value must respect string-escape state when scanning unknown fields.
//
// Before the fix, the depth-tracking loop used `data[pos-1] == '\\'` to test
// whether a quote was escaped, which is wrong for `\\"` (escaped backslash
// followed by closing quote): pos-1 IS '\\' but the quote actually closes the
// string. Without this fix, decoding a struct with an unknown field whose
// value contains `\\"` could read past the string and corrupt downstream
// fields.
// ============================================================================

#[test]
fn skip_value_handles_escaped_backslash_followed_by_quote() {
    use fastserial::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct OnlyId {
        id: u32,
    }

    // The unknown "note" field's value is the 1-char string `\` (a single
    // backslash). In JSON wire form that's `"\\"` — two backslashes between
    // quotes. After skipping it we must still find `id` correctly.
    let input = br#"{"note":"\\","id":7}"#;
    let v: OnlyId = decode(input).unwrap();
    assert_eq!(v, OnlyId { id: 7 });
}

#[test]
fn skip_value_handles_unicode_escape_in_unknown_field() {
    use fastserial::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct OnlyId {
        id: u32,
    }

    let input = br#"{"label":"\u0041BC","id":99}"#;
    let v: OnlyId = decode(input).unwrap();
    assert_eq!(v, OnlyId { id: 99 });
}

#[test]
fn skip_value_handles_nested_array_with_strings() {
    use fastserial::{Decode, Encode};

    #[derive(Encode, Decode, Debug, PartialEq)]
    struct OnlyId {
        id: u32,
    }

    let input = br#"{"tags":[["a\"b","c"],{"x":1}],"id":1}"#;
    let v: OnlyId = decode(input).unwrap();
    assert_eq!(v, OnlyId { id: 1 });
}
