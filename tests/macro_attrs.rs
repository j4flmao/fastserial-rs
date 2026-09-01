use fastserial::{Decode, Encode, json};

fn is_false(v: &bool) -> bool {
    !*v
}

#[derive(Encode, Decode, Debug, PartialEq)]
#[fastserial(rename_all = "camelCase")]
struct Config {
    pub server_url: String,

    #[fastserial(default)]
    pub max_connections: u32,

    #[fastserial(default, skip_serializing_if = "is_false")]
    pub enable_logging: bool,
}

#[test]
fn test_rename_all_and_default() {
    let mut json = b"{\"serverUrl\":\"http://localhost:8080\"}".to_vec();
    let arena = fastserial::arena::Arena::new();
    let mut config: Config = json::decode(&mut json, &arena).unwrap();

    assert_eq!(config.server_url, "http://localhost:8080");
    assert_eq!(config.max_connections, 0); // Default applied
    assert_eq!(config.enable_logging, false);

    config.max_connections = 100;

    let encoded = json::encode(&config).unwrap();
    let encoded_str = std::str::from_utf8(&encoded).unwrap();

    // enable_logging is false, so it should be skipped
    assert_eq!(
        encoded_str,
        r#"{"serverUrl":"http://localhost:8080","maxConnections":100}"#
    );

    config.enable_logging = true;
    let encoded2 = json::encode(&config).unwrap();
    let encoded_str2 = std::str::from_utf8(&encoded2).unwrap();

    // enable_logging is true, so it should be serialized
    assert_eq!(
        encoded_str2,
        r#"{"serverUrl":"http://localhost:8080","maxConnections":100,"enableLogging":true}"#
    );
}
