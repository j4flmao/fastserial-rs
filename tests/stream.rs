use fastserial::stream::NdjsonStream;
use fastserial::{Decode, arena::Arena};

#[derive(Decode, Debug, PartialEq)]
struct LogEntry<'a> {
    level: &'a str,
    message: &'a str,
}

#[test]
fn test_ndjson_stream() {
    let ndjson = b"{\"level\":\"INFO\",\"message\":\"Started\"}\n\n{\"level\":\"ERROR\",\"message\":\"Failed\"}\n";
    let arena = Arena::new();
    let mut stream = NdjsonStream::new(&ndjson[..], &arena);

    let entry1: LogEntry = stream.next_obj().unwrap().unwrap();
    assert_eq!(entry1.level, "INFO");
    assert_eq!(entry1.message, "Started");

    let entry2: LogEntry = stream.next_obj().unwrap().unwrap();
    assert_eq!(entry2.level, "ERROR");
    assert_eq!(entry2.message, "Failed");

    assert!(stream.next_obj::<LogEntry>().is_none());
}

#[cfg(feature = "tokio")]
#[tokio::test]
async fn test_async_ndjson_stream() {
    use fastserial::stream::AsyncNdjsonStream;
    let ndjson = b"{\"level\":\"INFO\",\"message\":\"Started\"}\n{\"level\":\"ERROR\",\"message\":\"Failed\"}\n";
    let arena = Arena::new();
    let mut stream = AsyncNdjsonStream::new(&ndjson[..], &arena);

    let entry1: LogEntry = stream.next_obj().await.unwrap().unwrap();
    assert_eq!(entry1.level, "INFO");

    let entry2: LogEntry = stream.next_obj().await.unwrap().unwrap();
    assert_eq!(entry2.level, "ERROR");

    assert!(stream.next_obj::<LogEntry>().await.is_none());
}
