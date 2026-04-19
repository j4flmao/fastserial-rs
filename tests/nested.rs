use fastserial::json::{decode, encode};
use fastserial::{Decode, Encode};

#[derive(Debug, PartialEq, Encode, Decode)]
struct Address {
    street: String,
    city: String,
    zip_code: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct User {
    id: u64,
    name: String,
    emails: Vec<String>,
    address: Address,
    active: bool,
    score: Option<f64>,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Company {
    name: String,
    employees: Vec<User>,
    founded_year: i32,
}

#[test]
fn test_nested_struct_roundtrip() {
    let company = Company {
        name: "Tech Corp".to_string(),
        employees: vec![
            User {
                id: 1,
                name: "Alice".to_string(),
                emails: vec![
                    "alice@example.com".to_string(),
                    "work@alice.com".to_string(),
                ],
                address: Address {
                    street: "123 Main St".to_string(),
                    city: "Tech City".to_string(),
                    zip_code: 12345,
                },
                active: true,
                score: Some(95.5),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
                emails: vec!["bob@gmail.com".to_string()],
                address: Address {
                    street: "456 Oak Ave".to_string(),
                    city: "Dev Town".to_string(),
                    zip_code: 67890,
                },
                active: false,
                score: None,
            },
        ],
        founded_year: 2010,
    };

    let json = encode(&company).expect("Failed to encode Company");
    let decoded: Company = decode(&json).expect("Failed to decode Company");

    assert_eq!(company, decoded);
}

#[test]
fn test_deeply_nested_vecs() {
    #[derive(Debug, PartialEq, Encode, Decode)]
    struct Matrix {
        data: Vec<Vec<Vec<i32>>>,
    }

    let matrix = Matrix {
        data: vec![vec![vec![1, 2], vec![3, 4]], vec![vec![5, 6], vec![7, 8]]],
    };

    let json = encode(&matrix).expect("Failed to encode Matrix");
    let decoded: Matrix = decode(&json).expect("Failed to decode Matrix");

    assert_eq!(matrix, decoded);
}

#[test]
fn test_empty_nested_structs() {
    let company = Company {
        name: "".to_string(),
        employees: vec![],
        founded_year: 0,
    };

    let json = encode(&company).expect("Failed to encode empty Company");
    let decoded: Company = decode(&json).expect("Failed to decode empty Company");

    assert_eq!(company, decoded);
}
