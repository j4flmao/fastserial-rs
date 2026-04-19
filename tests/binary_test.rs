use fastserial::binary::{decode, encode};

#[test]
fn test_binary_roundtrip_u64() {
    let original: u64 = 42;
    let encoded = encode(&original).unwrap();

    assert_eq!(&encoded[0..4], b"FBIN");

    let decoded: u64 = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_i64() {
    let original: i64 = -12345;
    let encoded = encode(&original).unwrap();
    let decoded: i64 = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_f64() {
    let original: f64 = std::f64::consts::PI;
    let encoded = encode(&original).unwrap();
    let decoded: f64 = decode(&encoded).unwrap();
    assert!((original - decoded).abs() < 0.001);
}

#[test]
fn test_binary_roundtrip_bool() {
    let original: bool = true;
    let encoded = encode(&original).unwrap();
    let decoded: bool = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_string() {
    let original: &str = "hello";
    let encoded = encode(&original).unwrap();
    let decoded: &str = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_vec() {
    let original: Vec<u64> = vec![1, 2, 3, 4, 5];
    let encoded = encode(&original).unwrap();
    let decoded: Vec<u64> = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_empty_vec() {
    let original: Vec<u64> = vec![];
    let encoded = encode(&original).unwrap();
    let decoded: Vec<u64> = decode(&encoded).unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_binary_roundtrip_option_some() {
    let original: Option<u64> = Some(42);
    let encoded = encode(&original).unwrap();
    let decoded: Option<u64> = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}

#[test]
fn test_binary_roundtrip_option_none() {
    let original: Option<u64> = None;
    let encoded = encode(&original).unwrap();
    let decoded: Option<u64> = decode(&encoded).unwrap();
    assert_eq!(original, decoded);
}
