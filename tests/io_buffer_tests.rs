use fastserial::io::{ReadBuffer, WriteBuffer};

// ─── WriteBuffer for Vec<u8> ────────────────────────────────────────────────

#[test]
fn test_vec_write_byte() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut buf: Vec<u8> = Vec::new();
    buf.write_byte(b'a').unwrap();
    buf.write_byte(b'b').unwrap();
    buf.write_byte(b'c').unwrap();
    assert_eq!(buf, b"abc");
}

#[test]
fn test_vec_write_bytes() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut buf: Vec<u8> = Vec::new();
    buf.write_bytes(b"hello").unwrap();
    buf.write_bytes(b" world").unwrap();
    assert_eq!(buf, b"hello world");
}

#[test]
fn test_vec_write_empty_bytes() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut buf: Vec<u8> = Vec::new();
    buf.write_bytes(b"").unwrap();
    assert!(buf.is_empty());
}

#[test]
fn test_vec_reserve() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let buf: Vec<u8> = Vec::with_capacity(1024);
    assert!(buf.capacity() >= 1024);
}

#[test]
fn test_vec_write_mixed() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut buf: Vec<u8> = Vec::new();
    buf.write_byte(b'{').unwrap();
    buf.write_bytes(b"\"key\":").unwrap();
    buf.write_bytes(b"42").unwrap();
    buf.write_byte(b'}').unwrap();
    assert_eq!(buf, b"{\"key\":42}");
}

// ─── WriteBuffer for &mut [u8] ─────────────────────────────────────────────

#[test]
fn test_slice_write_byte() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut backing = [0u8; 10];
    let buf: &mut [u8] = &mut backing;
    let mut writer = buf;
    writer.write_byte(b'X').unwrap();
    assert_eq!(backing[0], b'X');
}

#[test]
fn test_slice_write_bytes() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut backing = [0u8; 10];
    {
        let mut writer: &mut [u8] = &mut backing;
        writer.write_bytes(b"hello").unwrap();
    }
    assert_eq!(&backing[..5], b"hello");
}

#[test]
fn test_slice_write_byte_overflow() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut backing = [0u8; 0];
    let mut writer: &mut [u8] = &mut backing;
    let result = writer.write_byte(b'X');
    assert!(result.is_err());
}

#[test]
fn test_slice_write_bytes_overflow() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut backing = [0u8; 3];
    let mut writer: &mut [u8] = &mut backing;
    let result = writer.write_bytes(b"hello");
    assert!(result.is_err());
}

#[test]
fn test_slice_write_exact_capacity() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut backing = [0u8; 5];
    {
        let mut writer: &mut [u8] = &mut backing;
        writer.write_bytes(b"exact").unwrap();
    }
    assert_eq!(&backing, b"exact");
}

// ─── ReadBuffer ─────────────────────────────────────────────────────────────

#[test]
fn test_readbuffer_new() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let buf = ReadBuffer::new(&data);
    assert_eq!(buf.get_pos(), 0);
    assert!(!buf.is_eof());
}

#[test]
fn test_readbuffer_empty() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"".to_vec();
    let buf = ReadBuffer::new(&data);
    assert!(buf.is_eof());
    assert_eq!(buf.remaining(), 0);
}

#[test]
fn test_readbuffer_peek() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"abc".to_vec();
    let buf = ReadBuffer::new(&data);
    assert_eq!(buf.peek(), b'a');
    // peek should not advance
    assert_eq!(buf.peek(), b'a');
}

#[test]
fn test_readbuffer_peek_eof() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"".to_vec();
    let buf = ReadBuffer::new(&data);
    assert_eq!(buf.peek(), 0);
}

#[test]
fn test_readbuffer_next_byte() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"abc".to_vec();
    let mut buf = ReadBuffer::new(&data);
    assert_eq!(buf.next_byte().unwrap(), b'a');
    assert_eq!(buf.next_byte().unwrap(), b'b');
    assert_eq!(buf.next_byte().unwrap(), b'c');
    assert!(buf.is_eof());
}

#[test]
fn test_readbuffer_next_byte_eof() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"".to_vec();
    let mut buf = ReadBuffer::new(&data);
    let result = buf.next_byte();
    assert!(result.is_err());
}

#[test]
fn test_readbuffer_advance() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello world".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.advance(5);
    assert_eq!(buf.get_pos(), 5);
    assert_eq!(buf.peek(), b' ');
}

#[test]
fn test_readbuffer_expect_byte_success() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b'h').unwrap();
    assert_eq!(buf.get_pos(), 1);
}

#[test]
fn test_readbuffer_expect_byte_failure() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let mut buf = ReadBuffer::new(&data);
    let result = buf.expect_byte(b'x');
    assert!(result.is_err());
}

#[test]
fn test_readbuffer_expect_bytes_success() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello world".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_bytes(b"hello").unwrap();
    assert_eq!(buf.get_pos(), 5);
}

#[test]
fn test_readbuffer_expect_bytes_failure() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let mut buf = ReadBuffer::new(&data);
    let result = buf.expect_bytes(b"world");
    assert!(result.is_err());
}

#[test]
fn test_readbuffer_expect_at_success() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let buf = ReadBuffer::new(&data);
    buf.expect_at(0, b'h').unwrap();
    buf.expect_at(4, b'o').unwrap();
}

#[test]
fn test_readbuffer_expect_at_failure() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let buf = ReadBuffer::new(&data);
    let result = buf.expect_at(0, b'x');
    assert!(result.is_err());
}

#[test]
fn test_readbuffer_expect_at_out_of_bounds() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hi".to_vec();
    let buf = ReadBuffer::new(&data);
    let result = buf.expect_at(10, b'x');
    assert!(result.is_err());
}

#[test]
fn test_readbuffer_remaining() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let mut buf = ReadBuffer::new(&data);
    assert_eq!(buf.remaining(), 5);
    buf.advance(3);
    assert_eq!(buf.remaining(), 2);
    buf.advance(2);
    assert_eq!(buf.remaining(), 0);
}

#[test]
fn test_readbuffer_slice_from() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello world".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.advance(5);
    let slice = buf.slice_from(0);
    assert_eq!(slice, b"hello");
}

#[test]
fn test_readbuffer_peek_slice() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello world".to_vec();
    let buf = ReadBuffer::new(&data);
    let peeked = buf.peek_slice(5);
    assert_eq!(peeked, b"hello");
    assert_eq!(buf.get_pos(), 0); // should not advance
}

#[test]
fn test_readbuffer_peek_slice_beyond_end() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hi".to_vec();
    let buf = ReadBuffer::new(&data);
    let peeked = buf.peek_slice(100);
    assert_eq!(peeked, b"hi"); // clamped to available
}

#[test]
fn test_readbuffer_skip() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hello".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.skip(3);
    assert_eq!(buf.get_pos(), 3);
    assert_eq!(buf.peek(), b'l');
}

#[test]
fn test_readbuffer_skip_beyond_end() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"hi".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.skip(100);
    assert!(buf.is_eof());
}

#[test]
fn test_readbuffer_is_eof_after_read_all() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"ab".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.next_byte().unwrap();
    assert!(!buf.is_eof());
    buf.next_byte().unwrap();
    assert!(buf.is_eof());
}

// ─── ReadBuffer with expect_byte special chars ──────────────────────────────

#[test]
fn test_readbuffer_expect_open_brace() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"{".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b'{').unwrap();
}

#[test]
fn test_readbuffer_expect_close_brace() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"}".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b'}').unwrap();
}

#[test]
fn test_readbuffer_expect_open_bracket() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"[".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b'[').unwrap();
}

#[test]
fn test_readbuffer_expect_close_bracket() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"]".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b']').unwrap();
}

#[test]
fn test_readbuffer_expect_colon() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b":".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b':').unwrap();
}

#[test]
fn test_readbuffer_expect_comma() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b",".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b',').unwrap();
}

#[test]
fn test_readbuffer_expect_quote() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let data = b"\"".to_vec();
    let mut buf = ReadBuffer::new(&data);
    buf.expect_byte(b'"').unwrap();
}
