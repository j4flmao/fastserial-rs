use fastserial::Error;
use fastserial::json::decode;

#[test]
fn test_invalid_json_missing_quotes() {
    let json = b"{id: 42}"; // Missing quotes around "id"
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_json_unclosed_brace() {
    let json = b"{\"id\": 42";
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_json_extra_comma() {
    let json = b"{\"id\": 42,}";
    let result: Result<u64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_number_format() {
    let json = b"42.42.42";
    let result: Result<f64, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_invalid_boolean() {
    let json = b"truuu";
    let result: Result<bool, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_string_eof() {
    let json = b"\"hello";
    let result: Result<&str, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_unexpected_token() {
    let json = b"[1, 2, {]";
    let result: Result<Vec<u64>, Error> = decode(json);
    assert!(result.is_err());
}

#[test]
fn test_binary_invalid_magic() {
    use fastserial::binary::decode as binary_decode;
    let data = b"NOT_FBIN_MAGIC";
    let result: Result<u64, Error> = binary_decode(data);
    assert!(result.is_err());
}
