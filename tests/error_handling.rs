use fastserial::Error;
use fastserial::json::decode;

#[test]
fn test_invalid_json_missing_quotes() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"{id: 42}".to_vec(); // Missing quotes around "id"
    let result: Result<u64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_invalid_json_unclosed_brace() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"{\"id\": 42".to_vec();
    let result: Result<u64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_invalid_json_extra_comma() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"{\"id\": 42,}".to_vec();
    let result: Result<u64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_invalid_number_format() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"42.42.42".to_vec();
    let result: Result<f64, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_invalid_boolean() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"truuu".to_vec();
    let result: Result<bool, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_string_eof() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"\"hello".to_vec();
    let result: Result<&str, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_unexpected_token() {
    let _arena = fastserial::arena::Arena::new();
    let mut json = b"[1, 2, {]".to_vec();
    let result: Result<Vec<u64>, Error> = decode(&mut json, &_arena);
    assert!(result.is_err());
}

#[test]
fn test_binary_invalid_magic() {
    let _arena = fastserial::arena::Arena::new();
    use fastserial::binary::decode as binary_decode;
    let mut data = b"NOT_FBIN_MAGIC".to_vec();
    let result: Result<u64, Error> = binary_decode(&mut data, &_arena);
    assert!(result.is_err());
}
