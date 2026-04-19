use fastserial::json::decode;

#[test]
fn test_zero_copy_string() {
    let input = b"\"hello\"";
    let decoded: &str = decode(input).unwrap();
    assert_eq!(decoded, "hello");

    let ptr = decoded.as_ptr() as usize;
    let input_ptr = input.as_ptr() as usize;
    let input_end = input_ptr + input.len();

    assert!(
        ptr >= input_ptr && ptr < input_end,
        "decoded string should borrow from input buffer"
    );
}

#[test]
fn test_zero_copy_vec() {
    let input = br#"["a","b","c"]"#;
    let decoded: Vec<&str> = decode(input).unwrap();
    assert_eq!(decoded, vec!["a", "b", "c"]);
}

#[test]
fn test_zero_copy_numbers() {
    let input = b"42";
    let decoded: u64 = decode(input).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_large_input() {
    let json_str = r#""xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx""#;

    let input = json_str.as_bytes();
    let result: Result<&str, _> = decode(input);

    assert!(result.is_ok());
}

#[test]
fn test_mixed_content() {
    let input = b"123";
    let decoded: u64 = decode(input).unwrap();
    assert_eq!(decoded, 123);
}
