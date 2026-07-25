use fastserial::json::decode;

#[test]
fn test_zero_copy_string() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut input = b"\"hello\"".to_vec();
    let decoded: &str = decode(&mut input, &_arena).unwrap();
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
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut input = br#"["a","b","c"]"#.to_vec();
    let decoded: Vec<&str> = decode(&mut input, &_arena).unwrap();
    assert_eq!(decoded, vec!["a", "b", "c"]);
}

#[test]
fn test_zero_copy_numbers() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut input = b"42".to_vec();
    let decoded: u64 = decode(&mut input, &_arena).unwrap();
    assert_eq!(decoded, 42);
}

#[test]
fn test_large_input() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let json_str = r#""xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx""#;

    let mut input = json_str.as_bytes();
    let result: Result<&str, _> = decode(&mut input, &_arena);

    assert!(result.is_ok());
}

#[test]
fn test_mixed_content() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut input = b"123".to_vec();
    let decoded: u64 = decode(&mut input, &_arena).unwrap();
    assert_eq!(decoded, 123);
}
