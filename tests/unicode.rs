use fastserial::json::{decode, encode};

#[test]
fn test_unicode_general() {
    let _arena = fastserial::arena::Arena::new();
    let s = "Hello world with Emoji 🚀";
    let mut json = encode(&s).expect("Failed to encode Unicode string");
    let decoded: String = decode(&mut json, &_arena).expect("Failed to decode Unicode string");
    assert_eq!(s, decoded);
}

#[test]
fn test_unicode_escape_sequences() {
    let _arena = fastserial::arena::Arena::new();
    // \u1234 format
    let mut json = br#""\u2764\ufe0f""#.to_vec(); // ❤️
    let decoded: String =
        decode(&mut json, &_arena).expect("Failed to decode Unicode escape sequence");
    assert_eq!(decoded, "❤️");
}

#[test]
fn test_complex_unicode_escaping() {
    let _arena = fastserial::arena::Arena::new();
    let s = "Emoji: 🦀, Japanese: こんにちは, Russian: Привет";
    let mut json = encode(&s).expect("Failed to encode complex Unicode string");
    let decoded: String =
        decode(&mut json, &_arena).expect("Failed to decode complex Unicode string");
    assert_eq!(s, decoded);
}

#[test]
fn test_special_escapes_roundtrip() {
    let _arena = fastserial::arena::Arena::new();
    let s = "Line 1\nLine 2\tTabbed\r\"Quotes\"\\Backslash";
    let mut json = encode(&s).expect("Failed to encode escaped string");
    let decoded: String = decode(&mut json, &_arena).expect("Failed to decode escaped string");
    assert_eq!(s, decoded);
}

#[test]
fn test_null_byte_in_string() {
    let _arena = fastserial::arena::Arena::new();
    let s = "Hello\0World";
    let mut json = encode(&s).expect("Failed to encode null byte string");
    let decoded: String = decode(&mut json, &_arena).expect("Failed to decode null byte string");
    assert_eq!(s, decoded);
}
