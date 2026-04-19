use fastserial::simd;

// ─── scan_quote_or_backslash ────────────────────────────────────────────────

#[test]
fn test_scan_empty_input() {
    assert_eq!(simd::scan_quote_or_backslash(b""), 0);
}

#[test]
fn test_scan_no_special_chars() {
    let input = b"abcdefghijklmnop";
    assert_eq!(simd::scan_quote_or_backslash(input), input.len());
}

#[test]
fn test_scan_quote_at_start() {
    assert_eq!(simd::scan_quote_or_backslash(b"\"hello"), 0);
}

#[test]
fn test_scan_quote_at_end() {
    assert_eq!(simd::scan_quote_or_backslash(b"hello\""), 5);
}

#[test]
fn test_scan_backslash_at_start() {
    assert_eq!(simd::scan_quote_or_backslash(b"\\hello"), 0);
}

#[test]
fn test_scan_backslash_at_end() {
    assert_eq!(simd::scan_quote_or_backslash(b"hello\\"), 5);
}

#[test]
fn test_scan_quote_before_backslash() {
    assert_eq!(simd::scan_quote_or_backslash(b"abc\"def\\ghi"), 3);
}

#[test]
fn test_scan_backslash_before_quote() {
    assert_eq!(simd::scan_quote_or_backslash(b"abc\\def\"ghi"), 3);
}

#[test]
fn test_scan_only_quote() {
    assert_eq!(simd::scan_quote_or_backslash(b"\""), 0);
}

#[test]
fn test_scan_only_backslash() {
    assert_eq!(simd::scan_quote_or_backslash(b"\\"), 0);
}

#[test]
fn test_scan_large_no_match() {
    let input = vec![b'a'; 256];
    assert_eq!(simd::scan_quote_or_backslash(&input), 256);
}

#[test]
fn test_scan_large_with_quote_at_end() {
    let mut input = vec![b'a'; 255];
    input.push(b'"');
    assert_eq!(simd::scan_quote_or_backslash(&input), 255);
}

#[test]
fn test_scan_boundary_15() {
    let mut input = vec![b'x'; 15];
    input.push(b'"');
    assert_eq!(simd::scan_quote_or_backslash(&input), 15);
}

#[test]
fn test_scan_boundary_16() {
    let mut input = vec![b'x'; 16];
    input.push(b'"');
    assert_eq!(simd::scan_quote_or_backslash(&input), 16);
}

#[test]
fn test_scan_boundary_17() {
    let mut input = vec![b'x'; 17];
    input.push(b'"');
    assert_eq!(simd::scan_quote_or_backslash(&input), 17);
}

#[test]
fn test_scan_boundary_63() {
    let mut input = vec![b'x'; 63];
    input.push(b'\\');
    assert_eq!(simd::scan_quote_or_backslash(&input), 63);
}

#[test]
fn test_scan_boundary_64() {
    let mut input = vec![b'x'; 64];
    input.push(b'\\');
    assert_eq!(simd::scan_quote_or_backslash(&input), 64);
}

// ─── skip_whitespace ────────────────────────────────────────────────────────

#[test]
fn test_skip_whitespace_empty() {
    assert_eq!(simd::skip_whitespace(b""), 0);
}

#[test]
fn test_skip_whitespace_no_ws() {
    assert_eq!(simd::skip_whitespace(b"hello"), 0);
}

#[test]
fn test_skip_whitespace_spaces() {
    assert_eq!(simd::skip_whitespace(b"   hello"), 3);
}

#[test]
fn test_skip_whitespace_tabs() {
    assert_eq!(simd::skip_whitespace(b"\t\thello"), 2);
}

#[test]
fn test_skip_whitespace_newlines() {
    assert_eq!(simd::skip_whitespace(b"\n\nhello"), 2);
}

#[test]
fn test_skip_whitespace_carriage_return() {
    assert_eq!(simd::skip_whitespace(b"\r\nhello"), 2);
}

#[test]
fn test_skip_whitespace_mixed() {
    assert_eq!(simd::skip_whitespace(b" \t\n\r hello"), 5);
}

#[test]
fn test_skip_whitespace_all_whitespace() {
    let input = b"     ";
    assert_eq!(simd::skip_whitespace(input), input.len());
}

#[test]
fn test_skip_whitespace_large() {
    let mut input = vec![b' '; 200];
    input.push(b'x');
    assert_eq!(simd::skip_whitespace(&input), 200);
}

#[test]
fn test_skip_whitespace_boundary_31() {
    let mut input = vec![b' '; 31];
    input.push(b'x');
    assert_eq!(simd::skip_whitespace(&input), 31);
}

#[test]
fn test_skip_whitespace_boundary_32() {
    let mut input = vec![b' '; 32];
    input.push(b'x');
    assert_eq!(simd::skip_whitespace(&input), 32);
}

#[test]
fn test_skip_whitespace_boundary_33() {
    let mut input = vec![b' '; 33];
    input.push(b'x');
    assert_eq!(simd::skip_whitespace(&input), 33);
}

// ─── is_all_ascii ───────────────────────────────────────────────────────────

#[test]
fn test_is_all_ascii_empty() {
    assert!(simd::is_all_ascii(b""));
}

#[test]
fn test_is_all_ascii_basic() {
    assert!(simd::is_all_ascii(b"hello world"));
}

#[test]
fn test_is_all_ascii_full_range() {
    let input: Vec<u8> = (0..128).collect();
    assert!(simd::is_all_ascii(&input));
}

#[test]
fn test_is_all_ascii_with_non_ascii() {
    assert!(!simd::is_all_ascii(b"\x80"));
    assert!(!simd::is_all_ascii(b"\xff"));
    assert!(!simd::is_all_ascii("ñ".as_bytes()));
}

#[test]
fn test_is_all_ascii_mixed() {
    assert!(!simd::is_all_ascii("hello café".as_bytes()));
}

#[test]
fn test_is_all_ascii_large_ascii() {
    let input = vec![b'a'; 1000];
    assert!(simd::is_all_ascii(&input));
}

#[test]
fn test_is_all_ascii_large_with_non_ascii_at_end() {
    let mut input = vec![b'a'; 999];
    input.push(0x80);
    assert!(!simd::is_all_ascii(&input));
}

#[test]
fn test_is_all_ascii_boundary_31() {
    let input = vec![b'x'; 31];
    assert!(simd::is_all_ascii(&input));
}

#[test]
fn test_is_all_ascii_boundary_32() {
    let input = vec![b'x'; 32];
    assert!(simd::is_all_ascii(&input));
}

#[test]
fn test_is_all_ascii_boundary_33() {
    let input = vec![b'x'; 33];
    assert!(simd::is_all_ascii(&input));
}
