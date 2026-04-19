use fastserial::Error;
use fastserial::json::{decode, encode};

#[test]
fn test_whitespace_handling() {
    let json = b"  42  ";
    let decoded: u64 = decode(json).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_whitespace_handling_with_tabs() {
    let json = b"\t\n42\r\n";
    let decoded: u64 = decode(json).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_whitespace_handling_leading() {
    let json = b"   123";
    let decoded: u64 = decode(json).unwrap();
    assert_eq!(decoded, 123);
}

#[test]
fn test_whitespace_handling_trailing() {
    let json = b"456   ";
    let decoded: u64 = decode(json).unwrap();
    assert_eq!(decoded, 456);
}

#[test]
fn test_number_variants() {
    let json = b"42";
    let n: u64 = decode(json).unwrap();
    assert_eq!(n, 42);

    let json = b"-123";
    let n: i64 = decode(json).unwrap();
    assert_eq!(n, -123);

    let json = b"0";
    let n: u64 = decode(json).unwrap();
    assert_eq!(n, 0);

    let json = b"18446744073709551615";
    let n: u64 = decode(json).unwrap();
    assert_eq!(n, u64::MAX);
}

#[test]
fn test_float_handling() {
    let json = b"3.125";
    let n: f64 = decode(json).unwrap();
    assert_eq!(n, 3.125);

    let json = b"0.5";
    let n: f64 = decode(json).unwrap();
    assert_eq!(n, 0.5);
}

#[test]
fn test_boolean_variants() {
    let json = b"true";
    let b: bool = decode(json).unwrap();
    assert!(b);

    let json = b"false";
    let b: bool = decode(json).unwrap();
    assert!(!b);
}

#[test]
fn test_null_handling() {
    let json = b"null";
    let result: Result<(), Error> = decode(json);
    assert!(result.is_ok());
}

#[test]
fn test_empty_vec() {
    let v: Vec<u64> = vec![];
    let json = encode(&v).unwrap();
    assert_eq!(json, b"[]");

    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_strings() {
    let v: Vec<&str> = vec!["a", "b", "c"];
    let json = encode(&v).unwrap();
    assert_eq!(json, br#"["a","b","c"]"#);

    let decoded: Vec<&str> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_numbers() {
    let v: Vec<i32> = vec![1, -2, 3, -4, 5];
    let json = encode(&v).unwrap();
    let decoded: Vec<i32> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_integers() {
    let v: Vec<i32> = vec![1, 2, 3];
    let json = encode(&v).unwrap();
    let decoded: Vec<i32> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_string_with_unicode() {
    let s: &str = "hello world with accents";
    let json = encode(&s).unwrap();
    let decoded: &str = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_emoji() {
    let s: &str = "hello 👋 world";
    let json = encode(&s).unwrap();
    let decoded: &str = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_control_chars() {
    let s: &str = "test\x00null";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_escaped_backslash() {
    let s: &str = "path\\to\\file";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_escaped_quotes() {
    let s: &str = "say \"hello\"";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_error_unexpected_byte() {
    let json = b"abc";
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_error_eof() {
    let json = b"";
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_error_invalid_utf8() {
    let json = b"\xff\xfe";
    let result: Result<&str, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_nested_vec() {
    let v: Vec<Vec<u64>> = vec![vec![1, 2], vec![3, 4]];
    let json = encode(&v).unwrap();
    let decoded: Vec<Vec<u64>> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_option_in_vec() {
    let v: Vec<Option<u64>> = vec![Some(1), None, Some(3)];
    let json = encode(&v).unwrap();
    let decoded: Vec<Option<u64>> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_of_strings_escaped() {
    let v: Vec<String> = vec!["hello\nworld".to_string(), "tab\there".to_string()];
    let json = encode(&v).unwrap();
    let decoded: Vec<String> = decode(&json).unwrap();
    assert_eq!(v[0], decoded[0]);
    assert_eq!(v[1], decoded[1]);
}

#[test]
fn test_string_long() {
    let s = String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ").repeat(100);
    let json = encode(&s).unwrap();
    let decoded: &str = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_vec_large() {
    let v: Vec<u64> = (0..10000).collect();
    let json = encode(&v).unwrap();
    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}
