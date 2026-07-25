use fastserial::Error;
use fastserial::json::{decode, encode};

#[test]
fn test_whitespace_handling() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"  42  ".to_vec();
    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_whitespace_handling_with_tabs() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"\t\n42\r\n".to_vec();
    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_whitespace_handling_leading() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"   123".to_vec();
    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded, 123);
}

#[test]
fn test_whitespace_handling_trailing() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"456   ".to_vec();
    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded, 456);
}

#[test]
fn test_number_variants() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"42".to_vec();
    let n: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, 42);

    let mut json = b"-123".to_vec();
    let n: i64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, -123);

    let mut json = b"0".to_vec();
    let n: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, 0);

    let mut json = b"18446744073709551615".to_vec();
    let n: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, u64::MAX);
}

#[test]
fn test_float_handling() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"3.125".to_vec();
    let n: f64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, 3.125);

    let mut json = b"0.5".to_vec();
    let n: f64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, 0.5);
}

#[test]
fn test_boolean_variants() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"true".to_vec();
    let b: bool = decode(&mut json, &_arena).unwrap();
    assert!(b);

    let mut json = b"false".to_vec();
    let b: bool = decode(&mut json, &_arena).unwrap();
    assert!(!b);
}

#[test]
fn test_null_handling() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"null".to_vec();
    let result: Result<(), Error> = decode(&mut json, &_arena);
    assert!(result.is_ok());
}

#[test]
fn test_empty_vec() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<u64> = vec![];
    let mut json = encode(&v).unwrap();
    assert_eq!(json, b"[]");

    let decoded: Vec<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_strings() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<&str> = vec!["a", "b", "c"];
    let mut json = encode(&v).unwrap();
    assert_eq!(json, br#"["a","b","c"]"#);

    let decoded: Vec<&str> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_numbers() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<i32> = vec![1, -2, 3, -4, 5];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<i32> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_with_integers() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<i32> = vec![1, 2, 3];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<i32> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_string_with_unicode() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "hello world with accents";
    let mut json = encode(&s).unwrap();
    let decoded: &str = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_emoji() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "hello 👋 world";
    let mut json = encode(&s).unwrap();
    let decoded: &str = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_control_chars() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "test\x00null";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_escaped_backslash() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "path\\to\\file";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_escaped_quotes() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "say \"hello\"";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_error_unexpected_byte() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"abc".to_vec();
    let result: Result<u64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_error_eof() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"".to_vec();
    let result: Result<u64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_error_invalid_utf8() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"\xff\xfe".to_vec();
    let result: Result<&str, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_nested_vec() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<Vec<u64>> = vec![vec![1, 2], vec![3, 4]];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<Vec<u64>> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_option_in_vec() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<Option<u64>> = vec![Some(1), None, Some(3)];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<Option<u64>> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_of_strings_escaped() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<String> = vec!["hello\nworld".to_string(), "tab\there".to_string()];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<String> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v[0], decoded[0]);
    assert_eq!(v[1], decoded[1]);
}

#[test]
fn test_string_long() {
    let _arena = fastserial::arena::Arena::new();
    let mut s =
        String::from("Lorem ipsum dolor sit amet, consectetur adipiscing elit. ").repeat(100);
    let mut json = encode(&s).unwrap();
    let decoded: &str = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_vec_large() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<u64> = (0..10000).collect();
    let mut json = encode(&v).unwrap();
    let decoded: Vec<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}
