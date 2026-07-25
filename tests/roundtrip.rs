use fastserial::json::{decode, encode};

#[test]
fn test_encode_decode_u64() {
    let _arena = fastserial::arena::Arena::new();
    let n: u64 = 42;
    let mut json = encode(&n).unwrap();
    assert_eq!(json, b"42");

    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u64_large() {
    let _arena = fastserial::arena::Arena::new();
    let n: u64 = u64::MAX;
    let mut json = encode(&n).unwrap();
    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u64_zero() {
    let _arena = fastserial::arena::Arena::new();
    let n: u64 = 0;
    let mut json = encode(&n).unwrap();
    assert_eq!(json, b"0");

    let decoded: u64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64() {
    let _arena = fastserial::arena::Arena::new();
    let n: i64 = -123;
    let mut json = encode(&n).unwrap();
    assert_eq!(json, b"-123");

    let decoded: i64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64_positive() {
    let _arena = fastserial::arena::Arena::new();
    let n: i64 = 12345;
    let mut json = encode(&n).unwrap();
    let decoded: i64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]

fn test_encode_decode_i64_min() {
    let _arena = fastserial::arena::Arena::new();
    let n: i64 = i64::MIN;
    let mut json = encode(&n).unwrap();
    let decoded: i64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i64_max() {
    let _arena = fastserial::arena::Arena::new();
    let n: i64 = i64::MAX;
    let mut json = encode(&n).unwrap();
    let decoded: i64 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_bool_true() {
    let _arena = fastserial::arena::Arena::new();
    let b: bool = true;
    let mut json = encode(&b).unwrap();
    assert_eq!(json, b"true");

    let decoded: bool = decode(&mut json, &_arena).unwrap();
    assert_eq!(b, decoded);
}

#[test]
fn test_encode_decode_bool_false() {
    let _arena = fastserial::arena::Arena::new();
    let b: bool = false;
    let mut json = encode(&b).unwrap();
    assert_eq!(json, b"false");

    let decoded: bool = decode(&mut json, &_arena).unwrap();
    assert_eq!(b, decoded);
}

#[test]
fn test_encode_decode_string() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "hello";
    let mut json = encode(&s).unwrap();
    assert_eq!(json, br#""hello""#);

    let decoded: &str = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_empty() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "";
    let mut json = encode(&s).unwrap();
    assert_eq!(json, br#""""#);

    let decoded: &str = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_quotes() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "hello \"world\"";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_newline() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "line1\nline2";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_string_with_tab() {
    let _arena = fastserial::arena::Arena::new();
    let s: &str = "col1\tcol2";
    let mut json = encode(&s).unwrap();
    let decoded: String = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_encode_decode_vec_u64() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<u64> = vec![1, 2, 3];
    let mut json = encode(&v).unwrap();
    assert_eq!(json, b"[1,2,3]");

    let decoded: Vec<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_empty() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<u64> = vec![];
    let mut json = encode(&v).unwrap();
    assert_eq!(json, b"[]");

    let decoded: Vec<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_large() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<u64> = (0..1000).collect();
    let mut json = encode(&v).unwrap();
    let decoded: Vec<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_unit() {
    let _arena = fastserial::arena::Arena::new();
    let u: () = ();
    let mut json = encode(&u).unwrap();
    assert_eq!(json, b"null");

    let _: () = decode(&mut json, &_arena).unwrap();
}

#[test]
fn test_encode_decode_option_some() {
    let _arena = fastserial::arena::Arena::new();
    let o: Option<u64> = Some(42);
    let mut json = encode(&o).unwrap();
    assert_eq!(json, b"42");

    let decoded: Option<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_none() {
    let _arena = fastserial::arena::Arena::new();
    let o: Option<u64> = None;
    let mut json = encode(&o).unwrap();
    assert_eq!(json, b"null");

    let decoded: Option<u64> = decode(&mut json, &_arena).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_string_some() {
    let _arena = fastserial::arena::Arena::new();
    let o: Option<&str> = Some("hello");
    let mut json = encode(&o).unwrap();
    let decoded: Option<&str> = decode(&mut json, &_arena).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_option_string_none() {
    let _arena = fastserial::arena::Arena::new();
    let o: Option<&str> = None;
    let mut json = encode(&o).unwrap();
    let decoded: Option<&str> = decode(&mut json, &_arena).unwrap();
    assert_eq!(o, decoded);
}

#[test]
fn test_encode_decode_vec_string() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<&str> = vec!["a", "b", "c"];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<&str> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v, decoded);
}

#[test]
fn test_encode_decode_vec_string_with_special_chars() {
    let _arena = fastserial::arena::Arena::new();
    let v: Vec<&str> = vec!["hello", "world", "test\nwith\nnewlines"];
    let mut json = encode(&v).unwrap();
    let decoded: Vec<String> = decode(&mut json, &_arena).unwrap();
    assert_eq!(v.len(), decoded.len());
    for (original, decoded) in v.iter().zip(decoded.iter()) {
        assert_eq!(*original, decoded);
    }
}

#[test]
fn test_encode_decode_i32() {
    let _arena = fastserial::arena::Arena::new();
    let n: i32 = -1000;
    let mut json = encode(&n).unwrap();
    let decoded: i32 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u32() {
    let _arena = fastserial::arena::Arena::new();
    let n: u32 = 4000000;
    let mut json = encode(&n).unwrap();
    let decoded: u32 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i16() {
    let _arena = fastserial::arena::Arena::new();
    let n: i16 = -1000;
    let mut json = encode(&n).unwrap();
    let decoded: i16 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u16() {
    let _arena = fastserial::arena::Arena::new();
    let n: u16 = 60000;
    let mut json = encode(&n).unwrap();
    let decoded: u16 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_i8() {
    let _arena = fastserial::arena::Arena::new();
    let n: i8 = -100;
    let mut json = encode(&n).unwrap();
    let decoded: i8 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}

#[test]
fn test_encode_decode_u8() {
    let _arena = fastserial::arena::Arena::new();
    let n: u8 = 255;
    let mut json = encode(&n).unwrap();
    let decoded: u8 = decode(&mut json, &_arena).unwrap();
    assert_eq!(n, decoded);
}
