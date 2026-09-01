use fastserial::{arena::Arena, json};

#[test]
fn test_truncated_json() {
    let arena = Arena::new();

    let truncated = vec![
        b"[1, 2, ".to_vec(),
        b"{\"a\": ".to_vec(),
        b"{\"a\": 1".to_vec(),
        b"\"hello".to_vec(),
        b"true ".to_vec(), // wait, true is valid if it's the whole document
    ];

    for mut trunc in truncated.into_iter().take(4) {
        // We decode into a generic structure like tape or just expect it to fail for specific types
        let res: Result<Vec<i32>, _> = json::decode(&mut trunc, &arena);
        assert!(res.is_err());
    }
}

#[test]
fn test_unclosed_string() {
    let arena = Arena::new();
    let mut json = b"\"unclosed string here".to_vec();
    let res: Result<&str, _> = json::decode(&mut json, &arena);
    assert!(res.is_err());
}

#[test]
fn test_invalid_escape() {
    let arena = Arena::new();
    // \x is not valid JSON
    let mut json = b"\"invalid \\x escape\"".to_vec();
    let res: Result<&str, _> = json::decode(&mut json, &arena);
    assert!(res.is_err());
}

#[test]
fn test_invalid_bare_words() {
    let arena = Arena::new();
    let mut json = b"tru".to_vec();
    let res: Result<bool, _> = json::decode(&mut json, &arena);
    assert!(res.is_err());

    let mut json = b"fals".to_vec();
    let res: Result<bool, _> = json::decode(&mut json, &arena);
    assert!(res.is_err());

    let mut json = b"nul".to_vec();
    let res: Result<Option<i32>, _> = json::decode(&mut json, &arena);
    assert!(res.is_err());
}
