use fastserial::{arena::Arena, json};
use std::borrow::Cow;

#[test]
fn test_all_control_escapes() {
    let arena = Arena::new();

    // JSON defines \" \\ \/ \b \f \n \r \t
    let json_str = r#"["\"","\\","\/","\b","\f","\n","\r","\t"]"#;
    let mut json_data = json_str.as_bytes().to_vec();

    let strings: Vec<Cow<'_, str>> = json::decode(&mut json_data, &arena).unwrap();
    assert_eq!(strings[0], "\"");
    assert_eq!(strings[1], "\\");
    assert_eq!(strings[2], "/");
    assert_eq!(strings[3], "\x08"); // \b
    assert_eq!(strings[4], "\x0c"); // \f
    assert_eq!(strings[5], "\n");
    assert_eq!(strings[6], "\r");
    assert_eq!(strings[7], "\t");
}

#[test]
fn test_complex_unicode_surrogates() {
    let arena = Arena::new();

    // Test a surrogate pair: U+1D11E MUSICAL SYMBOL G CLEF
    // In UTF-16: D834 DD1E
    let json_str = r#"["\uD834\uDD1E", "𝄞"]"#;
    let mut json_data = json_str.as_bytes().to_vec();

    let strings: Vec<Cow<'_, str>> = json::decode(&mut json_data, &arena).unwrap();
    assert_eq!(strings[0], "𝄞");
    assert_eq!(strings[1], "𝄞");
}

#[test]
fn test_invalid_unicode_surrogates() {
    let arena = Arena::new();

    // High surrogate not followed by low surrogate
    let json_str = r#"["\uD834"]"#;
    let mut json_data = json_str.as_bytes().to_vec();

    let res: Result<Vec<Cow<'_, str>>, _> = json::decode(&mut json_data, &arena);
    assert!(res.is_err());

    // Low surrogate without high surrogate
    let json_str = r#"["\uDD1E"]"#;
    let mut json_data = json_str.as_bytes().to_vec();
    let res: Result<Vec<Cow<'_, str>>, _> = json::decode(&mut json_data, &arena);
    assert!(res.is_err());
}
