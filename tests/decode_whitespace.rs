use fastserial::{json, Decode, arena::Arena};
use std::collections::{HashMap, BTreeMap};

#[derive(Decode, PartialEq, Debug)]
struct TestStruct {
    a: i32,
    b: bool,
    c: String,
}

#[test]
fn test_decode_extreme_whitespace() {
    let arena = Arena::new();
    
    // Arrays
    let mut json = b" \n \r \t [ \n 1 \t , \r 2 \n ] \t ".to_vec();
    let val: Vec<i32> = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, vec![1, 2]);

    // Empty array
    let mut json = b" [ \n\t ] ".to_vec();
    let val: Vec<i32> = json::decode(&mut json, &arena).unwrap();
    assert!(val.is_empty());

    // Tuples
    let mut json = b" \n [ \t 42 \n , \r 43 \t ] \n ".to_vec();
    let val: (i32, i32) = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, (42, 43));

    // Structs
    let mut json = b" { \n \"a\" \t : \r 1 \n , \t \"b\" : true , \"c\" : \"hello\" \n } ".to_vec();
    let val: TestStruct = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, TestStruct { a: 1, b: true, c: "hello".to_string() });

    // Option
    let mut json = b" \n [ \t null \n , \r 42 \t ] \n ".to_vec();
    let val: (Option<i32>, Option<i32>) = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, (None, Some(42)));

    // HashMap
    let mut json = b" { \n \"key\" \t : \r 99 \n } ".to_vec();
    let val: HashMap<String, i32> = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val.get("key"), Some(&99));

    // BTreeMap
    let mut json = b" { \n \"key2\" \t : \r 100 \n } ".to_vec();
    let val: BTreeMap<String, i32> = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val.get("key2"), Some(&100));
}

#[test]
fn test_decode_no_whitespace_at_all() {
    let arena = Arena::new();
    
    // Arrays
    let mut json = b"[1,2]".to_vec();
    let val: Vec<i32> = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, vec![1, 2]);

    // Tuples
    let mut json = b"[42,43]".to_vec();
    let val: (i32, i32) = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, (42, 43));

    // Structs
    let mut json = b"{\"a\":1,\"b\":true,\"c\":\"hello\"}".to_vec();
    let val: TestStruct = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, TestStruct { a: 1, b: true, c: "hello".to_string() });

    // Option
    let mut json = b"[null,42]".to_vec();
    let val: (Option<i32>, Option<i32>) = json::decode(&mut json, &arena).unwrap();
    assert_eq!(val, (None, Some(42)));
}
