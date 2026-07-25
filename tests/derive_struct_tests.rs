use fastserial::json::{decode, encode};
use fastserial::{Decode, Encode};

// ─── Single-field struct ────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct SingleField {
    value: u64,
}

#[test]
fn test_single_field_struct() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = SingleField { value: 42 };
    let mut json = encode(&s).unwrap();
    let decoded: SingleField = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Many fields struct ────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct ManyFields {
    a: u8,
    b: u16,
    c: u32,
    d: u64,
    e: i8,
    f: i16,
    g: i32,
    h: i64,
    i: bool,
    j: String,
}

#[test]
fn test_many_fields_struct() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = ManyFields {
        a: 1,
        b: 2,
        c: 3,
        d: 4,
        e: -1,
        f: -2,
        g: -3,
        h: -4,
        i: true,
        j: "test".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: ManyFields = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with optional fields ────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithOptionals {
    name: String,
    age: Option<u32>,
    email: Option<String>,
}

#[test]
fn test_struct_all_options_present() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = WithOptionals {
        name: "Alice".to_string(),
        age: Some(30),
        email: Some("alice@example.com".to_string()),
    };
    let mut json = encode(&s).unwrap();
    let decoded: WithOptionals = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_all_options_none() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = WithOptionals {
        name: "Bob".to_string(),
        age: None,
        email: None,
    };
    let mut json = encode(&s).unwrap();
    let decoded: WithOptionals = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with Vec fields ─────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct WithVecs {
    tags: Vec<String>,
    scores: Vec<i32>,
}

#[test]
fn test_struct_with_populated_vecs() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = WithVecs {
        tags: vec!["rust".to_string(), "fast".to_string()],
        scores: vec![100, 95, 88],
    };
    let mut json = encode(&s).unwrap();
    let decoded: WithVecs = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_empty_vecs() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = WithVecs {
        tags: vec![],
        scores: vec![],
    };
    let mut json = encode(&s).unwrap();
    let decoded: WithVecs = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Nested structs ────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct Inner {
    x: i32,
    y: i32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Outer {
    name: String,
    position: Inner,
}

#[test]
fn test_nested_struct() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Outer {
        name: "point".to_string(),
        position: Inner { x: 10, y: 20 },
    };
    let mut json = encode(&s).unwrap();
    let decoded: Outer = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Deeply nested structs ─────────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct Level3 {
    value: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Level2 {
    deep: Level3,
    count: u32,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Level1 {
    name: String,
    child: Level2,
}

#[test]
fn test_deeply_nested_structs() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Level1 {
        name: "root".to_string(),
        child: Level2 {
            deep: Level3 {
                value: "leaf".to_string(),
            },
            count: 42,
        },
    };
    let mut json = encode(&s).unwrap();
    let decoded: Level1 = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with Vec of structs ─────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct Item {
    id: u32,
    label: String,
}

#[derive(Debug, PartialEq, Encode, Decode)]
struct Container {
    items: Vec<Item>,
}

#[test]
fn test_struct_with_vec_of_structs() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Container {
        items: vec![
            Item {
                id: 1,
                label: "first".to_string(),
            },
            Item {
                id: 2,
                label: "second".to_string(),
            },
            Item {
                id: 3,
                label: "third".to_string(),
            },
        ],
    };
    let mut json = encode(&s).unwrap();
    let decoded: Container = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_empty_vec_of_structs() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Container { items: vec![] };
    let mut json = encode(&s).unwrap();
    let decoded: Container = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with special string values ──────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct TextContent {
    title: String,
    body: String,
}

#[test]
fn test_struct_with_special_chars_in_strings() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = TextContent {
        title: "Hello \"World\"".to_string(),
        body: "Line1\nLine2\tTabbed\\Backslash".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: TextContent = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_empty_strings() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = TextContent {
        title: "".to_string(),
        body: "".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: TextContent = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_struct_with_unicode_strings() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = TextContent {
        title: "こんにちは".to_string(),
        body: "Emoji: 🦀🚀💡".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: TextContent = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with float fields ───────────────────────────────────────────────

#[derive(Debug, Encode, Decode)]
struct FloatStruct {
    x: f32,
    y: f64,
}

#[test]
fn test_struct_with_floats() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = FloatStruct { x: 1.5, y: 2.75 };
    let mut json = encode(&s).unwrap();
    let decoded: FloatStruct = decode(&mut json, &_arena).unwrap();
    assert!((s.x - decoded.x).abs() < 1e-6);
    assert!((s.y - decoded.y).abs() < 1e-14);
}

#[test]
fn test_struct_with_zero_floats() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = FloatStruct { x: 0.0, y: 0.0 };
    let mut json = encode(&s).unwrap();
    let decoded: FloatStruct = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded.x, 0.0);
    assert_eq!(decoded.y, 0.0);
}

// ─── Struct with mixed Option and Vec ───────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct Complex {
    id: u64,
    tags: Vec<String>,
    metadata: Option<String>,
    scores: Vec<Option<i32>>,
}

#[test]
fn test_complex_struct() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Complex {
        id: 100,
        tags: vec!["a".to_string(), "b".to_string()],
        metadata: Some("info".to_string()),
        scores: vec![Some(10), None, Some(20)],
    };
    let mut json = encode(&s).unwrap();
    let decoded: Complex = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

#[test]
fn test_complex_struct_minimal() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = Complex {
        id: 0,
        tags: vec![],
        metadata: None,
        scores: vec![],
    };
    let mut json = encode(&s).unwrap();
    let decoded: Complex = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── SCHEMA_HASH consistency ────────────────────────────────────────────────

#[test]
fn test_schema_hash_is_consistent() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    // Same type should always produce the same hash
    let hash1 = SingleField::SCHEMA_HASH;
    let hash2 = SingleField::SCHEMA_HASH;
    assert_eq!(hash1, hash2);
}

#[test]
fn test_primitive_schema_hashes_are_nonzero() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    // Primitive types should have non-zero hashes
    assert_ne!(u8::SCHEMA_HASH, 0);
    assert_ne!(u16::SCHEMA_HASH, 0);
    assert_ne!(u32::SCHEMA_HASH, 0);
    assert_ne!(u64::SCHEMA_HASH, 0);
    assert_ne!(i8::SCHEMA_HASH, 0);
    assert_ne!(i16::SCHEMA_HASH, 0);
    assert_ne!(i32::SCHEMA_HASH, 0);
    assert_ne!(i64::SCHEMA_HASH, 0);
    assert_ne!(bool::SCHEMA_HASH, 0);
    assert_ne!(String::SCHEMA_HASH, 0);
}

#[test]
fn test_different_primitives_different_hashes() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    assert_ne!(u8::SCHEMA_HASH, u16::SCHEMA_HASH);
    assert_ne!(u32::SCHEMA_HASH, i32::SCHEMA_HASH);
    assert_ne!(f32::SCHEMA_HASH, f64::SCHEMA_HASH);
    assert_ne!(bool::SCHEMA_HASH, u8::SCHEMA_HASH);
}

// ─── Struct with rename attribute ──────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct RenameTest {
    #[fastserial(rename = "user_id")]
    id: u64,
    #[fastserial(rename = "full_name")]
    name: String,
}

#[test]
fn test_rename_attribute_encode() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = RenameTest {
        id: 1,
        name: "test".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let json_str = String::from_utf8(json.clone()).unwrap();
    assert!(json_str.contains("\"user_id\":"));
    assert!(json_str.contains("\"full_name\":"));
    assert!(!json_str.contains("\"id\":"));
    assert!(!json_str.contains("\"name\":"));
}

#[test]
fn test_rename_attribute_roundtrip() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = RenameTest {
        id: 42,
        name: "Alice".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: RenameTest = decode(&mut json, &_arena).unwrap();
    assert_eq!(s, decoded);
}

// ─── Struct with skip attribute ────────────────────────────────────────────

#[derive(Debug, PartialEq, Encode, Decode)]
struct SkipTest {
    visible: String,
    #[fastserial(skip)]
    hidden: String,
}

#[test]
fn test_skip_attribute_not_in_output() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = SkipTest {
        visible: "shown".to_string(),
        hidden: "secret".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let json_str = String::from_utf8(json).unwrap();
    assert!(json_str.contains("\"visible\":"));
    assert!(!json_str.contains("\"hidden\":"));
    assert!(!json_str.contains("secret"));
}

#[test]
fn test_skip_attribute_default_on_decode() {
    let _arena = fastserial::arena::Arena::new();
    let _arena = fastserial::arena::Arena::new();
    let mut s = SkipTest {
        visible: "hello".to_string(),
        hidden: "should_be_lost".to_string(),
    };
    let mut json = encode(&s).unwrap();
    let decoded: SkipTest = decode(&mut json, &_arena).unwrap();
    assert_eq!(decoded.visible, "hello");
    assert_eq!(decoded.hidden, String::default());
}
