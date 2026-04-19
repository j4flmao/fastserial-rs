use fastserial::{Decode, Encode, json};

#[derive(Encode, Decode, Debug)]
struct Event<'de> {
    #[fastserial(rename = "type")]
    event_type: &'de str,
    timestamp: u64,
    data: &'de str,
}

fn main() {
    let event = Event {
        event_type: "login",
        timestamp: 1699999999,
        data: r#"{"user_id": 123}"#,
    };

    let json_bytes = json::encode(&event).expect("Failed to encode");
    println!("Encoded event: {}", String::from_utf8_lossy(&json_bytes));

    let decoded: Event = json::decode(&json_bytes).expect("Failed to decode");
    println!("Decoded event: {:?}", decoded);

    assert_eq!(event.event_type, decoded.event_type);
    assert_eq!(event.timestamp, decoded.timestamp);
    assert_eq!(event.data, decoded.data);

    println!("\nSerialization successful!");
}
