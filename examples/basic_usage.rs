use fastserial::{Decode, Encode, json};

#[derive(Encode, Decode, Debug)]
struct User<'de> {
    id: u64,
    name: &'de str,
    email: &'de str,
    active: bool,
}

#[derive(Encode, Decode, Debug)]
struct Post<'de> {
    id: u64,
    title: &'de str,
    content: &'de str,
    author_id: u64,
    tags: Vec<&'de str>,
}

fn main() {
    let user = User {
        id: 1,
        name: "Alice",
        email: "alice@example.com",
        active: true,
    };

    let json_bytes = json::encode(&user).expect("Failed to encode");
    println!("Encoded user: {}", String::from_utf8_lossy(&json_bytes));

    let decoded: User = json::decode(&json_bytes).expect("Failed to decode");
    println!("Decoded user: {:?}", decoded);

    let post = Post {
        id: 42,
        title: "Hello World",
        content: "This is my first post!",
        author_id: 1,
        tags: vec!["rust", "serialization", "fast"],
    };

    let json_bytes = json::encode(&post).expect("Failed to encode post");
    println!("\nEncoded post: {}", String::from_utf8_lossy(&json_bytes));

    let decoded: Post = json::decode(&json_bytes).expect("Failed to decode post");
    println!("Decoded post: {:?}", decoded);

    println!("\nSchema hash for User: {:#x}", User::SCHEMA_HASH);
    println!("Schema hash for Post: {:#x}", Post::SCHEMA_HASH);
}
