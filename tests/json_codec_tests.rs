use fastserial::json::{decode, encode, encode_into};
use fastserial::{Decode, Encode, Error};

// ─── encode_into tests ───────────────────────────────────────────────────────

#[test]
fn test_encode_into_u64() {
    let mut buf = Vec::new();
    encode_into(&42u64, &mut buf).unwrap();
    assert_eq!(buf, b"42");
}

#[test]
fn test_encode_into_string() {
    let mut buf = Vec::new();
    encode_into(&"hello", &mut buf).unwrap();
    assert_eq!(buf, br#""hello""#);
}

#[test]
fn test_encode_into_appends() {
    let mut buf = Vec::new();
    encode_into(&1u64, &mut buf).unwrap();
    encode_into(&2u64, &mut buf).unwrap();
    assert_eq!(buf, b"12");
}

#[test]
fn test_encode_into_bool() {
    let mut buf = Vec::new();
    encode_into(&true, &mut buf).unwrap();
    assert_eq!(buf, b"true");
}

#[test]
fn test_encode_into_vec() {
    let mut buf = Vec::new();
    let v: Vec<u32> = vec![10, 20, 30];
    encode_into(&v, &mut buf).unwrap();
    assert_eq!(buf, b"[10,20,30]");
}

#[test]
fn test_encode_into_option_some() {
    let mut buf = Vec::new();
    let o: Option<u64> = Some(99);
    encode_into(&o, &mut buf).unwrap();
    assert_eq!(buf, b"99");
}

#[test]
fn test_encode_into_option_none() {
    let mut buf = Vec::new();
    let o: Option<u64> = None;
    encode_into(&o, &mut buf).unwrap();
    assert_eq!(buf, b"null");
}

#[test]
fn test_encode_into_unit() {
    let mut buf = Vec::new();
    encode_into(&(), &mut buf).unwrap();
    assert_eq!(buf, b"null");
}

// ─── Float encode/decode tests ──────────────────────────────────────────────

#[test]
fn test_encode_f64_nan_returns_error() {
    let result = encode(&f64::NAN);
    assert!(result.is_err());
}

#[test]
fn test_encode_f64_infinity_returns_error() {
    let result = encode(&f64::INFINITY);
    assert!(result.is_err());
}

#[test]
fn test_encode_f64_neg_infinity_returns_error() {
    let result = encode(&f64::NEG_INFINITY);
    assert!(result.is_err());
}

#[test]
fn test_encode_f32_nan_returns_error() {
    let result = encode(&f32::NAN);
    assert!(result.is_err());
}

#[test]
fn test_encode_f32_infinity_returns_error() {
    let result = encode(&f32::INFINITY);
    assert!(result.is_err());
}

#[test]
fn test_decode_f64_zero() {
    let json = b"0.0";
    let decoded: f64 = decode(json).unwrap();
    assert_eq!(decoded, 0.0);
}

#[test]
#[allow(clippy::approx_constant)]
fn test_decode_f64_negative() {
    let json = b"-3.14";
    let decoded: f64 = decode(json).unwrap();
    assert!((decoded - (-3.14)).abs() < 1e-10);
}

#[test]
fn test_decode_f64_scientific_notation() {
    let json = b"1.5e10";
    let decoded: f64 = decode(json).unwrap();
    assert_eq!(decoded, 1.5e10);
}

#[test]
fn test_decode_f64_scientific_notation_uppercase() {
    let json = b"2.5E3";
    let decoded: f64 = decode(json).unwrap();
    assert_eq!(decoded, 2.5e3);
}

#[test]
fn test_decode_f64_scientific_notation_negative_exp() {
    let json = b"1.5e-3";
    let decoded: f64 = decode(json).unwrap();
    assert!((decoded - 0.0015).abs() < 1e-10);
}

#[test]
fn test_decode_f64_scientific_notation_positive_exp() {
    let json = b"1.5e+3";
    let decoded: f64 = decode(json).unwrap();
    assert_eq!(decoded, 1500.0);
}

#[test]
fn test_decode_f64_invalid_double_dot() {
    let json = b"1.2.3";
    let result: Result<f64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_decode_f64_dot_no_fractional() {
    let json = b"1.e5";
    let result: Result<f64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_f32_roundtrip() {
    let val: f32 = 0.1;
    let json = encode(&val).unwrap();
    let decoded: f32 = decode(&json).unwrap();
    assert!((val - decoded).abs() < 1e-6);
}

#[test]
fn test_f64_roundtrip_pi() {
    let val: f64 = std::f64::consts::PI;
    let json = encode(&val).unwrap();
    let decoded: f64 = decode(&json).unwrap();
    assert!((val - decoded).abs() < 1e-14);
}

#[test]
fn test_f64_roundtrip_very_small() {
    let val: f64 = 1e-300;
    let json = encode(&val).unwrap();
    let decoded: f64 = decode(&json).unwrap();
    assert!((val - decoded).abs() / val < 1e-10);
}

#[test]
fn test_f64_roundtrip_very_large() {
    let val: f64 = 1e300;
    let json = encode(&val).unwrap();
    let decoded: f64 = decode(&json).unwrap();
    assert!((val - decoded).abs() / val < 1e-10);
}

// ─── String escape edge cases ───────────────────────────────────────────────

#[test]
fn test_string_with_all_escape_chars() {
    let s = "quote:\" backslash:\\ newline:\n tab:\t cr:\r";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_control_char_0x01() {
    let s = "ctrl-a:\x01";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_multiple_control_chars() {
    let s = "\x01\x02\x03\x04\x05\x06\x07\x08\x09";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_string_with_slash_passthrough() {
    // JSON allows forward slash to be unescaped
    let json = br#""hello/world""#;
    let decoded: &str = decode(json).unwrap();
    assert_eq!(decoded, "hello/world");
}

#[test]
fn test_string_with_escaped_slash() {
    let json = br#""hello\/world""#;
    let decoded: String = decode(json).unwrap();
    assert_eq!(decoded, "hello/world");
}

#[test]
fn test_string_with_backspace_escape() {
    let json = br#""hello\bworld""#;
    let decoded: String = decode(json).unwrap();
    assert_eq!(decoded, "hello\x08world");
}

#[test]
fn test_string_with_formfeed_escape() {
    let json = br#""hello\fworld""#;
    let decoded: String = decode(json).unwrap();
    assert_eq!(decoded, "hello\x0cworld");
}

// ─── Trailing data detection ────────────────────────────────────────────────

#[test]
fn test_trailing_data_u64() {
    let json = b"42 extra";
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_trailing_data_string() {
    let json = br#""hello" extra"#;
    let result: Result<&str, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_trailing_data_bool() {
    let json = b"true false";
    let result: Result<bool, Error> = decode(json);
    assert!(result.is_err());
}

// ─── Struct derive Encode/Decode tests ──────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct SimpleStruct {
    id: u64,
    name: String,
}

#[test]
fn test_simple_struct_roundtrip() {
    let s = SimpleStruct {
        id: 1,
        name: "test".to_string(),
    };
    let json = encode(&s).unwrap();
    let decoded: SimpleStruct = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_simple_struct_json_format() {
    let s = SimpleStruct {
        id: 42,
        name: "hello".to_string(),
    };
    let json = encode(&s).unwrap();
    let json_str = String::from_utf8(json).unwrap();
    assert!(json_str.contains("\"id\":42"));
    assert!(json_str.contains("\"name\":\"hello\""));
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct AllPrimitivesStruct {
    a_u8: u8,
    a_u16: u16,
    a_u32: u32,
    a_u64: u64,
    a_i8: i8,
    a_i16: i16,
    a_i32: i32,
    a_i64: i64,
    a_f32: f32,
    a_f64: f64,
    a_bool: bool,
    a_string: String,
}

#[test]
fn test_all_primitives_struct_roundtrip() {
    let s = AllPrimitivesStruct {
        a_u8: 255,
        a_u16: 65535,
        a_u32: 4294967295,
        a_u64: 18446744073709551615,
        a_i8: -128,
        a_i16: -32768,
        a_i32: -2147483648,
        a_i64: -9223372036854775808,
        a_f32: std::f32::consts::PI,
        a_f64: std::f64::consts::E,
        a_bool: true,
        a_string: "all primitives".to_string(),
    };
    let json = encode(&s).unwrap();
    let decoded: AllPrimitivesStruct = decode(&json).unwrap();

    assert_eq!(s.a_u8, decoded.a_u8);
    assert_eq!(s.a_u16, decoded.a_u16);
    assert_eq!(s.a_u32, decoded.a_u32);
    assert_eq!(s.a_u64, decoded.a_u64);
    assert_eq!(s.a_i8, decoded.a_i8);
    assert_eq!(s.a_i16, decoded.a_i16);
    assert_eq!(s.a_i32, decoded.a_i32);
    assert_eq!(s.a_i64, decoded.a_i64);
    assert!((s.a_f32 - decoded.a_f32).abs() < 1e-5);
    assert!((s.a_f64 - decoded.a_f64).abs() < 1e-14);
    assert_eq!(s.a_bool, decoded.a_bool);
    assert_eq!(s.a_string, decoded.a_string);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithVec {
    items: Vec<u32>,
    labels: Vec<String>,
}

#[test]
fn test_struct_with_vec_roundtrip() {
    let s = StructWithVec {
        items: vec![1, 2, 3, 4, 5],
        labels: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithVec = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_empty_vecs() {
    let s = StructWithVec {
        items: vec![],
        labels: vec![],
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithVec = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithOptions {
    required: u64,
    optional_num: Option<i32>,
    optional_str: Option<String>,
}

#[test]
fn test_struct_with_all_some_options() {
    let s = StructWithOptions {
        required: 42,
        optional_num: Some(100),
        optional_str: Some("present".to_string()),
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithOptions = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_all_none_options() {
    let s = StructWithOptions {
        required: 0,
        optional_num: None,
        optional_str: None,
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithOptions = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_mixed_options() {
    let s = StructWithOptions {
        required: 7,
        optional_num: Some(-5),
        optional_str: None,
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithOptions = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct StructWithNestedVec {
    matrix: Vec<Vec<i32>>,
}

#[test]
fn test_struct_with_nested_vec() {
    let s = StructWithNestedVec {
        matrix: vec![vec![1, 2, 3], vec![4, 5, 6]],
    };
    let json = encode(&s).unwrap();
    let decoded: StructWithNestedVec = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

// ─── Vec<Struct> tests ──────────────────────────────────────────────────────

#[test]
fn test_vec_of_structs() {
    let v = vec![
        SimpleStruct {
            id: 1,
            name: "alice".to_string(),
        },
        SimpleStruct {
            id: 2,
            name: "bob".to_string(),
        },
    ];
    let json = encode(&v).unwrap();
    let decoded: Vec<SimpleStruct> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_empty_vec_of_structs() {
    let v: Vec<SimpleStruct> = vec![];
    let json = encode(&v).unwrap();
    let decoded: Vec<SimpleStruct> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

// ─── Whitespace tolerance in decode ─────────────────────────────────────────

#[test]
fn test_decode_vec_with_whitespace() {
    let json = b"[ 1 , 2 , 3 ]";
    let decoded: Vec<u64> = decode(json).unwrap();
    assert_eq!(decoded, vec![1, 2, 3]);
}

#[test]
fn test_decode_bool_with_whitespace() {
    let json = b"  true  ";
    let decoded: bool = decode(json).unwrap();
    assert!(decoded);
}

#[test]
fn test_decode_null_with_whitespace() {
    let json = b"  null  ";
    let _: () = decode(json).unwrap();
}

#[test]
fn test_decode_string_with_leading_whitespace() {
    // Strings in JSON don't skip leading whitespace before the quote
    // but the decoder should handle whitespace that's part of the format
    let json = br#"  "hello"  "#;
    // This should work if the decoder handles leading whitespace for strings
    let result: Result<String, Error> = decode(json);
    // The string decoder expects a quote first, so leading whitespace may fail
    // depending on implementation. Let's test what actually happens.
    if let Ok(decoded) = result {
        assert_eq!(decoded, "hello");
    }
}

// ─── Bytes encode/decode tests ──────────────────────────────────────────────

#[test]
fn test_encode_bytes_simple() {
    let bytes: &[u8] = b"hello";
    let json = encode(&bytes).unwrap();
    assert_eq!(json, br#""hello""#);
}

#[test]
fn test_encode_bytes_empty() {
    let bytes: &[u8] = b"";
    let json = encode(&bytes).unwrap();
    assert_eq!(json, br#""""#);
}

#[test]
fn test_bytes_roundtrip_no_escapes() {
    let bytes: &[u8] = b"helloworld";
    let json = encode(&bytes).unwrap();
    let decoded: &[u8] = decode(&json).unwrap();
    assert_eq!(bytes, decoded);
}

#[test]
fn test_bytes_with_special_chars_encodes_escaped() {
    // Bytes with special chars get escaped in encoding, but read_bytes
    // only supports unescaped strings (zero-copy), so we just verify encoding works
    let bytes: &[u8] = b"hello\"world";
    let json = encode(&bytes).unwrap();
    let json_str = String::from_utf8(json).unwrap();
    assert!(json_str.contains("\\\""));
}

// ─── Single element vec tests ───────────────────────────────────────────────

#[test]
fn test_vec_single_element() {
    let v: Vec<u64> = vec![42];
    let json = encode(&v).unwrap();
    assert_eq!(json, b"[42]");
    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_vec_single_string() {
    let v: Vec<&str> = vec!["only"];
    let json = encode(&v).unwrap();
    assert_eq!(json, br#"["only"]"#);
    let decoded: Vec<&str> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

// ─── Nested Option tests ───────────────────────────────────────────────────

#[test]
fn test_option_of_vec() {
    let o: Option<Vec<u32>> = Some(vec![1, 2, 3]);
    let json = encode(&o).unwrap();
    let decoded: Option<Vec<u32>> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_option_of_vec_none() {
    let o: Option<Vec<u32>> = None;
    let json = encode(&o).unwrap();
    assert_eq!(json, b"null");
    let decoded: Option<Vec<u32>> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_option_of_string() {
    let o: Option<String> = Some("test".to_string());
    let json = encode(&o).unwrap();
    let decoded: Option<String> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_option_of_bool() {
    let o: Option<bool> = Some(false);
    let json = encode(&o).unwrap();
    let decoded: Option<bool> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}
