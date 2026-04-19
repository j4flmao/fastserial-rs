use fastserial::json::{decode, encode};
use fastserial::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct AttributeTest {
    #[fastserial(rename = "user_name")]
    username: String,

    #[fastserial(skip)]
    secret: String,

    #[fastserial(rename = "active_status")]
    is_active: bool,
}

#[test]
fn test_rename_and_skip_attributes() {
    let test = AttributeTest {
        username: "j4flmao".to_string(),
        secret: "hidden_value".to_string(),
        is_active: true,
    };

    let json = encode(&test).expect("Failed to encode AttributeTest");
    let json_str = String::from_utf8_lossy(&json);

    // Check if renamed fields are present
    assert!(json_str.contains("\"user_name\":"));
    assert!(json_str.contains("\"active_status\":"));

    // Check if skipped field is absent
    assert!(!json_str.contains("\"secret\":"));
    assert!(!json_str.contains("hidden_value"));

    // Decode back
    let decoded: AttributeTest = decode(&json).expect("Failed to decode AttributeTest");

    assert_eq!(decoded.username, test.username);
    assert_eq!(decoded.is_active, test.is_active);
    // Skipped field should be initialized with Default::default()
    assert_eq!(decoded.secret, String::default());
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct NestedAttributeTest {
    #[fastserial(rename = "meta_data")]
    meta: AttributeTest,
}

#[test]
fn test_nested_rename_attributes() {
    let test = NestedAttributeTest {
        meta: AttributeTest {
            username: "admin".to_string(),
            secret: "top_secret".to_string(),
            is_active: false,
        },
    };

    let json = encode(&test).expect("Failed to encode NestedAttributeTest");
    let json_str = String::from_utf8_lossy(&json);

    assert!(json_str.contains("\"meta_data\":"));
    assert!(json_str.contains("\"user_name\":"));
    assert!(json_str.contains("\"active_status\":"));
    assert!(!json_str.contains("\"secret\":"));

    let decoded: NestedAttributeTest = decode(&json).expect("Failed to decode NestedAttributeTest");

    // We can't compare the whole struct because of the skipped field
    assert_eq!(decoded.meta.username, test.meta.username);
    assert_eq!(decoded.meta.is_active, test.meta.is_active);
    assert_eq!(decoded.meta.secret, String::default());
}
