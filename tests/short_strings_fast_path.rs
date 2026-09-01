use fastserial::{json, Decode, arena::Arena};
use std::borrow::Cow;

#[derive(Decode, PartialEq, Debug)]
struct StringTest<'a> {
    #[fastserial(rename = "s1")]
    s1: Cow<'a, str>,
    #[fastserial(rename = "s2")]
    s2: Cow<'a, str>,
    #[fastserial(rename = "s3")]
    s3: Cow<'a, str>,
    #[fastserial(rename = "s4")]
    s4: Cow<'a, str>,
}

#[test]
fn test_strings_around_simd_boundaries() {
    let arena = Arena::new();
    
    // Construct a JSON with various string lengths to test the 16-byte fast-path loop
    // and the 32-byte SIMD chunk transitions.
    let s1 = "a"; // 1 byte
    let s2 = "123456789012345"; // 15 bytes
    let s3 = "1234567890123456"; // 16 bytes
    let s4 = "12345678901234567890123456789012"; // 32 bytes

    let json_str = format!(
        r#"{{"s1":"{}","s2":"{}","s3":"{}","s4":"{}"}}"#,
        s1, s2, s3, s4
    );
    
    let mut json_data = json_str.into_bytes();
    let val: StringTest = json::decode(&mut json_data, &arena).unwrap();
    
    assert_eq!(val.s1, s1);
    assert_eq!(val.s2, s2);
    assert_eq!(val.s3, s3);
    assert_eq!(val.s4, s4);
}

#[test]
fn test_strings_with_escapes_around_simd_boundaries() {
    let arena = Arena::new();
    
    // s1: 15 bytes, escape at the end
    let s1 = "12345678901234\n";
    // s2: 16 bytes, escape at the end
    let s2 = "123456789012345\n";
    // s3: 31 bytes, escape at the end
    let s3 = "123456789012345678901234567890\n";
    // s4: 32 bytes, escape at the end
    let s4 = "1234567890123456789012345678901\n";

    let json_str = r#"{"s1":"12345678901234\n","s2":"123456789012345\n","s3":"123456789012345678901234567890\n","s4":"1234567890123456789012345678901\n"}"#;
    
    let mut json_data = json_str.as_bytes().to_vec();
    let val: StringTest = json::decode(&mut json_data, &arena).unwrap();
    
    assert_eq!(val.s1, s1);
    assert_eq!(val.s2, s2);
    assert_eq!(val.s3, s3);
    assert_eq!(val.s4, s4);
}

#[test]
fn test_decode_escaped_keys() {
    let arena = Arena::new();
    
    // Keys cannot have backslashes in read_key_fast. Wait, if read_key_fast encounters
    // a backslash, it returns Error::UnexpectedByte. We fall back to skip_value if it's unknown.
    // However, if a key contains an escape, we should reject it if it's the known key,
    // but right now it just errors out completely.
    // Let's test that we get the expected error.
    let json_str = r#"{"s\n1":"a","s2":"b","s3":"c","s4":"d"}"#;
    let mut json_data = json_str.as_bytes().to_vec();
    
    let res: Result<StringTest, fastserial::Error> = json::decode(&mut json_data, &arena);
    assert!(matches!(res, Err(fastserial::Error::UnexpectedByte)));
}
