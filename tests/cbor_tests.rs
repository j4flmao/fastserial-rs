use fastserial::value::{Value, Number};
use std::collections::BTreeMap;

#[test]
fn test_cbor_encode_decode() {
    let mut map = BTreeMap::new();
    map.insert("name".to_string(), Value::String("fastserial".to_string()));
    map.insert("speed".to_string(), Value::Number(Number::U64(100)));
    map.insert("max_u64".to_string(), Value::Number(Number::U64(u64::MAX)));
    map.insert("min_i64".to_string(), Value::Number(Number::I64(i64::MIN)));
    map.insert("pi".to_string(), Value::Number(Number::F64(3.14159)));
    map.insert("is_fast".to_string(), Value::Bool(true));
    map.insert("none".to_string(), Value::Null);
    
    let mut arr = Vec::new();
    arr.push(Value::Number(Number::U64(1)));
    arr.push(Value::String("two".to_string()));
    map.insert("list".to_string(), Value::Array(arr));
    
    let original = Value::Object(map);
    
    let encoded = fastserial::codec::cbor::encode(&original).unwrap();
    let decoded = fastserial::codec::cbor::decode(&encoded).unwrap();
    
    assert_eq!(original, decoded);
}

#[test]
fn test_cbor_errors() {
    // 1. Unexpected EOF
    let bytes = vec![0b101_11111, 0x61]; // map start, string start but missing bytes
    assert!(fastserial::codec::cbor::decode(&bytes).is_err());

    // 2. Invalid Float bytes
    let bytes = vec![0b111_11011, 0xFF, 0, 0, 0, 0, 0, 0, 0]; // 27 (f64) but invalid maybe? Wait, f64 can be anything.
    // 3. Unsupported Type (e.g. f16)
    let bytes = vec![0b111_11001, 0, 0]; // 25 is unsupported in our decoder
    assert!(fastserial::codec::cbor::decode(&bytes).is_err());
    
    // 4. Invalid integer sizes
    let bytes = vec![28]; // Invalid size
    assert!(fastserial::codec::cbor::decode(&bytes).is_err());
}

#[test]
fn test_cbor_primitive_edge_cases() {
    let cases = vec![
        Value::Number(Number::I64(-1)),
        Value::Number(Number::I64(-24)),
        Value::Number(Number::I64(-25)),
        Value::Number(Number::I64(i64::MIN)),
        Value::Number(Number::U64(0)),
        Value::Number(Number::U64(23)),
        Value::Number(Number::U64(24)),
        Value::Number(Number::U64(u8::MAX as u64)),
        Value::Number(Number::U64(u16::MAX as u64)),
        Value::Number(Number::U64(u32::MAX as u64)),
        Value::Number(Number::U64(u64::MAX)),
        Value::Bool(false),
        Value::Bool(true),
        Value::Null,
        Value::String("a".repeat(30)),
        Value::String("a".repeat(300)),
        Value::String("a".repeat(70000)),
    ];

    for case in cases {
        let encoded = fastserial::codec::cbor::encode(&case).unwrap();
        let decoded = fastserial::codec::cbor::decode(&encoded).unwrap();
        assert_eq!(case, decoded);
    }
}
