use fastserial::json::{decode, encode};

#[test]
fn test_encode_decode_u64() {
    let n: u64 = 42;
    let json = encode(&n).unwrap();
    assert_eq!(json, b"42");

    let decoded: u64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u64_large() {
    let n: u64 = u64::MAX;
    let json = encode(&n).unwrap();
    let decoded: u64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u64_zero() {
    let n: u64 = 0;
    let json = encode(&n).unwrap();
    assert_eq!(json, b"0");

    let decoded: u64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64() {
    let n: i64 = -123;
    let json = encode(&n).unwrap();
    assert_eq!(json, b"-123");

    let decoded: i64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64_positive() {
    let n: i64 = 12345;
    let json = encode(&n).unwrap();
    let decoded: i64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]

fn test_encode_decode_i64_min() {
    let n: i64 = i64::MIN;
    let json = encode(&n).unwrap();
    let decoded: i64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64_max() {
    let n: i64 = i64::MAX;
    let json = encode(&n).unwrap();
    let decoded: i64 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_bool_true() {
    let b: bool = true;
    let json = encode(&b).unwrap();
    assert_eq!(json, b"true");

    let decoded: bool = decode(&json).unwrap();
    assert_eq!(b, decoded);
}

#[test]
fn test_encode_decode_bool_false() {
    let b: bool = false;
    let json = encode(&b).unwrap();
    assert_eq!(json, b"false");

    let decoded: bool = decode(&json).unwrap();
    assert_eq!(b, decoded);
}

#[test]
fn test_encode_decode_string() {
    let s: &str = "hello";
    let json = encode(&s).unwrap();
    assert_eq!(json, br#""hello""#);

    let decoded: &str = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_empty() {
    let s: &str = "";
    let json = encode(&s).unwrap();
    assert_eq!(json, br#""""#);

    let decoded: &str = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_quotes() {
    let s: &str = "hello \"world\"";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_newline() {
    let s: &str = "line1\nline2";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_tab() {
    let s: &str = "col1\tcol2";
    let json = encode(&s).unwrap();
    let decoded: String = decode(&json).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_vec_u64() {
    let v: Vec<u64> = vec![1, 2, 3];
    let json = encode(&v).unwrap();
    assert_eq!(json, b"[1,2,3]");

    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_empty() {
    let v: Vec<u64> = vec![];
    let json = encode(&v).unwrap();
    assert_eq!(json, b"[]");

    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_large() {
    let v: Vec<u64> = (0..1000).collect();
    let json = encode(&v).unwrap();
    let decoded: Vec<u64> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_unit() {
    let u: () = ();
    let json = encode(&u).unwrap();
    assert_eq!(json, b"null");

    let _: () = decode(&json).unwrap();
}

#[test]
fn test_encode_decode_option_some() {
    let o: Option<u64> = Some(42);
    let json = encode(&o).unwrap();
    assert_eq!(json, b"42");

    let decoded: Option<u64> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_none() {
    let o: Option<u64> = None;
    let json = encode(&o).unwrap();
    assert_eq!(json, b"null");

    let decoded: Option<u64> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_string_some() {
    let o: Option<&str> = Some("hello");
    let json = encode(&o).unwrap();
    let decoded: Option<&str> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_string_none() {
    let o: Option<&str> = None;
    let json = encode(&o).unwrap();
    let decoded: Option<&str> = decode(&json).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_vec_string() {
    let v: Vec<&str> = vec!["a", "b", "c"];
    let json = encode(&v).unwrap();
    let decoded: Vec<&str> = decode(&json).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_string_with_special_chars() {
    let v: Vec<&str> = vec!["hello", "world", "test\nwith\nnewlines"];
    let json = encode(&v).unwrap();
    let decoded: Vec<String> = decode(&json).unwrap();
    assert_eq!(v.len(), decoded.len());
    for (original, decoded) in v.iter().zip(decoded.iter()) {
        assert_eq!(*original, decoded);
    }
}

#[test]
fn test_encode_decode_i32() {
    let n: i32 = -1000;
    let json = encode(&n).unwrap();
    let decoded: i32 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u32() {
    let n: u32 = 4000000;
    let json = encode(&n).unwrap();
    let decoded: u32 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i16() {
    let n: i16 = -1000;
    let json = encode(&n).unwrap();
    let decoded: i16 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u16() {
    let n: u16 = 60000;
    let json = encode(&n).unwrap();
    let decoded: u16 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i8() {
    let n: i8 = -100;
    let json = encode(&n).unwrap();
    let decoded: i8 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u8() {
    let n: u8 = 255;
    let json = encode(&n).unwrap();
    let decoded: u8 = decode(&json).unwrap();
    assert_eq!(n, decoded);
}
