use fastserial::binary::{decode, encode};
use fastserial::{Decode, Encode, Error};

// ─── Binary header validation ───────────────────────────────────────────────

#[test]
fn test_binary_header_magic() {
    let data = 42u64;
    let encoded = encode(&data).unwrap();
    assert_eq!(&encoded[0..4], b"FBIN");
}

#[test]
fn test_binary_header_version() {
    let data = 42u64;
    let encoded = encode(&data).unwrap();
    let version = u16::from_le_bytes([encoded[4], encoded[5]]);
    assert_eq!(version, 0x0001);
}

#[test]
fn test_binary_header_minimum_size() {
    let data = 0u64;
    let encoded = encode(&data).unwrap();
    assert!(
        encoded.len() >= 16,
        "Binary format must have at least 16-byte header"
    );
}

#[test]
fn test_binary_invalid_magic_error() {
    let data = b"XXXX\x01\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x0042";
    let result: Result<u64, Error> = decode(data);
    assert!(result.is_err());
}

#[test]
fn test_binary_too_short_input() {
    let data = b"FBIN";
    let result: Result<u64, Error> = decode(data);
    assert!(result.is_err());
}

#[test]
fn test_binary_empty_input() {
    let data: &[u8] = b"";
    let result: Result<u64, Error> = decode(data);
    assert!(result.is_err());
}

#[test]
fn test_binary_wrong_version() {
    let mut data = Vec::new();
    data.extend_from_slice(b"FBIN");
    data.extend_from_slice(&0x0099u16.to_le_bytes()); // wrong version
    data.extend_from_slice(&0u64.to_le_bytes()); // schema hash
    data.extend_from_slice(&[0, 0]); // reserved
    data.extend_from_slice(&42u64.to_le_bytes());
    let result: Result<u64, Error> = decode(&data);
    assert!(result.is_err());
}

// ─── Binary primitive roundtrips ────────────────────────────────────────────

#[test]
fn test_binary_u8_roundtrip() {
    let val: u8 = 200;
    let encoded = encode(&val).unwrap();
    let decoded: u8 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_u16_roundtrip() {
    let val: u16 = 50000;
    let encoded = encode(&val).unwrap();
    let decoded: u16 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_u32_roundtrip() {
    let val: u32 = 3_000_000_000;
    let encoded = encode(&val).unwrap();
    let decoded: u32 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_i8_roundtrip() {
    let val: i8 = -100;
    let encoded = encode(&val).unwrap();
    let decoded: i8 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_i16_roundtrip() {
    let val: i16 = -30000;
    let encoded = encode(&val).unwrap();
    let decoded: i16 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_i32_roundtrip() {
    let val: i32 = -2_000_000_000;
    let encoded = encode(&val).unwrap();
    let decoded: i32 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_f32_roundtrip() {
    let val: f32 = std::f32::consts::PI;
    let encoded = encode(&val).unwrap();
    let decoded: f32 = decode(&encoded).unwrap();
    assert!((val - decoded).abs() < 1e-5);
}

#[test]
fn test_binary_bool_false_roundtrip() {
    let val: bool = false;
    let encoded = encode(&val).unwrap();
    let decoded: bool = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_unit_roundtrip() {
    let val: () = ();
    let encoded = encode(&val).unwrap();
    let _: () = decode(&encoded).unwrap();
}

// ─── Binary string tests ───────────────────────────────────────────────────

#[test]
fn test_binary_i64_max() {
    let val: i64 = i64::MAX;
    let encoded = encode(&val).unwrap();
    let decoded: i64 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_long_string() {
    let val = "x".repeat(10000);
    let encoded = encode(&val).unwrap();
    let decoded: String = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_string_with_unicode() {
    let val = "Hello 🦀 Rust";
    let encoded = encode(&val).unwrap();
    let decoded: &str = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

// ─── Binary vec tests ──────────────────────────────────────────────────────

#[test]
fn test_binary_vec_i32() {
    let val: Vec<i32> = vec![-1, 0, 1, 100, -100];
    let encoded = encode(&val).unwrap();
    let decoded: Vec<i32> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_vec_single() {
    let val: Vec<u64> = vec![42];
    let encoded = encode(&val).unwrap();
    let decoded: Vec<u64> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_vec_strings() {
    let val: Vec<&str> = vec!["hello", "world", "test"];
    let encoded = encode(&val).unwrap();
    let decoded: Vec<&str> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_vec_large() {
    let val: Vec<u64> = (0..1000).collect();
    let encoded = encode(&val).unwrap();
    let decoded: Vec<u64> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

// ─── Binary struct tests ───────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct BinaryTestStruct {
    id: u64,
    name: String,
    active: bool,
}

#[test]
fn test_binary_struct_roundtrip() {
    let val = BinaryTestStruct {
        id: 42,
        name: "test".to_string(),
        active: true,
    };
    let encoded = encode(&val).unwrap();
    let decoded: BinaryTestStruct = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct BinaryNestedStruct {
    inner: BinaryTestStruct,
    count: u32,
}

#[test]
fn test_binary_nested_struct_roundtrip() {
    let val = BinaryNestedStruct {
        inner: BinaryTestStruct {
            id: 1,
            name: "nested".to_string(),
            active: false,
        },
        count: 99,
    };
    let encoded = encode(&val).unwrap();
    let decoded: BinaryNestedStruct = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

// ─── Binary option tests ───────────────────────────────────────────────────

#[test]
fn test_binary_option_some_string() {
    let val: Option<String> = Some("optional".to_string());
    let encoded = encode(&val).unwrap();
    let decoded: Option<String> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_option_none_string() {
    let val: Option<String> = None;
    let encoded = encode(&val).unwrap();
    let decoded: Option<String> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_option_some_i64() {
    let val: Option<i64> = Some(-999);
    let encoded = encode(&val).unwrap();
    let decoded: Option<i64> = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

// ─── Binary boundary values ────────────────────────────────────────────────

#[test]
fn test_binary_u64_max() {
    let val: u64 = u64::MAX;
    let encoded = encode(&val).unwrap();
    let decoded: u64 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_f64_zero() {
    let val: f64 = 0.0;
    let encoded = encode(&val).unwrap();
    let decoded: f64 = decode(&encoded).unwrap();
    assert_eq!(val, decoded);
}

#[test]
fn test_binary_f64_negative_zero() {
    let val: f64 = -0.0;
    let encoded = encode(&val).unwrap();
    let decoded: f64 = decode(&encoded).unwrap();
    assert!(decoded.is_sign_negative() || decoded == 0.0);
}
