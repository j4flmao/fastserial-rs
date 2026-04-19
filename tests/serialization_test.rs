//! FastSerial unit tests

use fastserial::{Decode, Encode, binary, json};

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
struct TestUser {
    id: u64,
    username: String,
    email: String,
}

#[derive(Encode, Decode, PartialEq, Debug, Clone)]
struct TestPost {
    id: u64,
    title: String,
    content: String,
    user_id: u64,
    tags: Vec<String>,
}

#[test]
fn test_user_json_serialize() {
    let user = TestUser {
        id: 1,
        username: "john".into(),
        email: "john@example.com".into(),
    };

    let encoded = json::encode(&user).unwrap();
    let decoded: TestUser = json::decode(&encoded).unwrap();

    assert_eq!(user.username, decoded.username);
}

#[test]
fn test_user_binary_serialize() {
    let user = TestUser {
        id: 1,
        username: "john".into(),
        email: "john@example.com".into(),
    };

    let encoded = binary::encode_raw(&user).unwrap();
    let decoded: TestUser = binary::decode_raw(&encoded).unwrap();

    assert_eq!(user.email, decoded.email);
}

#[test]
fn test_post_with_vec_json() {
    let post = TestPost {
        id: 1,
        title: "Hello".into(),
        content: "World".into(),
        user_id: 1,
        tags: vec!["rust".into(), "fast".into()],
    };

    let encoded = json::encode(&post).unwrap();
    let decoded: TestPost = json::decode(&encoded).unwrap();

    assert_eq!(post.tags.len(), decoded.tags.len());
}

#[test]
fn test_post_with_vec_binary() {
    let post = TestPost {
        id: 1,
        title: "Hello".into(),
        content: "World".into(),
        user_id: 1,
        tags: vec!["rust".into(), "fast".into()],
    };

    let encoded = binary::encode_raw(&post).unwrap();
    let decoded: TestPost = binary::decode_raw(&encoded).unwrap();

    assert_eq!(post.tags.len(), decoded.tags.len());
}

#[test]
fn test_vec_users_json() {
    let users: Vec<TestUser> = (0..100)
        .map(|i| TestUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();

    let encoded = json::encode(&users).unwrap();
    let decoded: Vec<TestUser> = json::decode(&encoded).unwrap();

    assert_eq!(users.len(), decoded.len());
}

#[test]
fn test_vec_users_binary() {
    let users: Vec<TestUser> = (0..100)
        .map(|i| TestUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();

    let encoded = binary::encode_raw(&users).unwrap();
    let decoded: Vec<TestUser> = binary::decode_raw(&encoded).unwrap();

    assert_eq!(users.len(), decoded.len());
}

#[test]
fn test_size_comparison_single() {
    let user = TestUser {
        id: 1,
        username: "johndoe".into(),
        email: "john@example.com".into(),
    };

    let json_size = json::encode(&user).unwrap().len();
    let binary_size = binary::encode_raw(&user).unwrap().len();

    println!(
        "Single user - JSON: {} bytes, Binary: {} bytes",
        json_size, binary_size
    );
    // Binary is smaller for structs with many fields/strings
}

#[test]
fn test_size_comparison_vector() {
    let users: Vec<TestUser> = (0..100)
        .map(|i| TestUser {
            id: i,
            username: format!("user{}", i),
            email: format!("user{}@example.com", i),
        })
        .collect();

    let json_size = json::encode(&users).unwrap().len();
    let binary_size = binary::encode_raw(&users).unwrap().len();

    println!(
        "100 users - JSON: {} bytes, Binary: {} bytes",
        json_size, binary_size
    );
    println!(
        "Binary is {:.1}% of JSON",
        (binary_size as f64 / json_size as f64) * 100.0
    );
    // Binary should be smaller or equal for large vectors
}

#[test]
fn test_option_field() {
    #[derive(Encode, Decode, PartialEq, Debug)]
    struct WithOption {
        id: u64,
        name: String,
        description: Option<String>,
    }

    let with_some = WithOption {
        id: 1,
        name: "Test".into(),
        description: Some("desc".into()),
    };

    let with_none = WithOption {
        id: 2,
        name: "Test2".into(),
        description: None,
    };

    let encoded = json::encode(&with_some).unwrap();
    let decoded: WithOption = json::decode(&encoded).unwrap();
    assert!(decoded.description.is_some());

    let encoded = json::encode(&with_none).unwrap();
    let decoded: WithOption = json::decode(&encoded).unwrap();
    assert!(decoded.description.is_none());
}

#[test]
fn test_binary_vs_json_performance() {
    let user = TestUser {
        id: 1,
        username: "john".into(),
        email: "john@example.com".into(),
    };

    use std::time::Instant;
    let start = Instant::now();
    for _ in 0..10000 {
        let _ = json::encode(&user).unwrap();
    }
    let json_time = start.elapsed().as_millis();

    let start = Instant::now();
    for _ in 0..10000 {
        let _ = binary::encode_raw(&user).unwrap();
    }
    let binary_time = start.elapsed().as_millis();

    println!(
        "10000 iterations - JSON: {}ms, Binary: {}ms",
        json_time, binary_time
    );
}
