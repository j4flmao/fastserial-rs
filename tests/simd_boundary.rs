use fastserial::json::decode;

#[test]
fn test_simd_whitespace_boundary_32() {
    // 31 whitespaces + 1 char = 32 bytes (boundary for AVX2)
    let mut input = vec![b' '; 31];
    input.push(b'1');
    let decoded: u64 = decode(&input).unwrap();
    assert_eq!(decoded, 1);
}

#[test]
fn test_simd_whitespace_boundary_33() {
    // 32 whitespaces + 1 char = 33 bytes
    let mut input = vec![b' '; 32];
    input.push(b'2');
    let decoded: u64 = decode(&input).unwrap();
    assert_eq!(decoded, 2);
}

#[test]
fn test_simd_string_boundary_32() {
    // String content of exactly 32 bytes (excluding quotes)
    let content = "a".repeat(32);
    let input = format!("\"{}\"", content);
    let decoded: &str = decode(input.as_bytes()).unwrap();
    assert_eq!(decoded, content);
}

#[test]
fn test_simd_string_boundary_31() {
    // String content of exactly 31 bytes (excluding quotes)
    let content = "b".repeat(31);
    let input = format!("\"{}\"", content);
    let decoded: &str = decode(input.as_bytes()).unwrap();
    assert_eq!(decoded, content);
}

#[test]
fn test_simd_string_with_escapes_near_boundary() {
    // Escape character near 32-byte boundary
    let mut content = "a".repeat(30);
    content.push_str("\\n"); // This makes it 32 characters in JSON
    let input = format!("\"{}\"", content);
    let decoded: String = decode(input.as_bytes()).unwrap();
    assert_eq!(decoded, "a".repeat(30) + "\n");
}

#[test]
fn test_multiple_simd_chunks_whitespace() {
    // Multiple 32-byte chunks of whitespace
    let mut input = vec![b' '; 100];
    input.push(b'5');
    let decoded: u64 = decode(&input).unwrap();
    assert_eq!(decoded, 5);
}
