use fastserial::binary::{decode, encode};

#[test]
fn test_binary_roundtrip_u64() {
    let _arena = fastserial::arena::Arena::new();
    let original: u64 = 42;
    let mut encoded = encode(&original).unwrap();

    assert_eq!(&encoded[0..4], b"FBIN");

    let decoded: u64 = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_i64() {
    let _arena = fastserial::arena::Arena::new();
    let original: i64 = -12345;
    let mut encoded = encode(&original).unwrap();
    let decoded: i64 = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_f64() {
    let _arena = fastserial::arena::Arena::new();
    let original: f64 = std::f64::consts::PI;
    let mut encoded = encode(&original).unwrap();
    let decoded: f64 = decode(&mut encoded, &_arena).unwrap();
    assert!((original - decoded).abs() < 0.001);
}

#[test]
fn test_binary_roundtrip_bool() {
    let _arena = fastserial::arena::Arena::new();
    let original: bool = true;
    let mut encoded = encode(&original).unwrap();
    let decoded: bool = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_string() {
    let _arena = fastserial::arena::Arena::new();
    let original: &str = "hello";
    let mut encoded = encode(&original).unwrap();
    let decoded: &str = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_vec() {
    let _arena = fastserial::arena::Arena::new();
    let original: Vec<u64> = vec![1, 2, 3, 4, 5];
    let mut encoded = encode(&original).unwrap();
    let decoded: Vec<u64> = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_empty_vec() {
    let _arena = fastserial::arena::Arena::new();
    let original: Vec<u64> = vec![];
    let mut encoded = encode(&original).unwrap();
    let decoded: Vec<u64> = decode(&mut encoded, &_arena).unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_binary_roundtrip_option_some() {
    let _arena = fastserial::arena::Arena::new();
    let original: Option<u64> = Some(42);
    let mut encoded = encode(&original).unwrap();
    let decoded: Option<u64> = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_option_none() {
    let _arena = fastserial::arena::Arena::new();
    let original: Option<u64> = None;
    let mut encoded = encode(&original).unwrap();
    let decoded: Option<u64> = decode(&mut encoded, &_arena).unwrap();
    assert_eq!(original, decoded);
}
