use fastserial::schema::types::FieldType;
use fastserial::schema::validate::validate_field_type;

// ─── FieldType::is_scalar ───────────────────────────────────────────────────

#[test]
fn test_scalar_types() {
    assert!(FieldType::Bool.is_scalar());
    assert!(FieldType::U8.is_scalar());
    assert!(FieldType::U16.is_scalar());
    assert!(FieldType::U32.is_scalar());
    assert!(FieldType::U64.is_scalar());
    assert!(FieldType::I8.is_scalar());
    assert!(FieldType::I16.is_scalar());
    assert!(FieldType::I32.is_scalar());
    assert!(FieldType::I64.is_scalar());
    assert!(FieldType::F32.is_scalar());
    assert!(FieldType::F64.is_scalar());
}

#[test]
fn test_non_scalar_types() {
    assert!(!FieldType::String.is_scalar());
    assert!(!FieldType::Bytes.is_scalar());
    assert!(!FieldType::Option(Box::new(FieldType::U32)).is_scalar());
    assert!(!FieldType::Vec(Box::new(FieldType::U32)).is_scalar());
    assert!(!FieldType::Struct("Test").is_scalar());
}

// ─── FieldType::alignment ───────────────────────────────────────────────────

#[test]
fn test_alignment_1_byte() {
    assert_eq!(FieldType::Bool.alignment(), 1);
    assert_eq!(FieldType::U8.alignment(), 1);
    assert_eq!(FieldType::I8.alignment(), 1);
}

#[test]
fn test_alignment_2_bytes() {
    assert_eq!(FieldType::U16.alignment(), 2);
    assert_eq!(FieldType::I16.alignment(), 2);
}

#[test]
fn test_alignment_4_bytes() {
    assert_eq!(FieldType::U32.alignment(), 4);
    assert_eq!(FieldType::I32.alignment(), 4);
    assert_eq!(FieldType::F32.alignment(), 4);
}

#[test]
fn test_alignment_8_bytes() {
    assert_eq!(FieldType::U64.alignment(), 8);
    assert_eq!(FieldType::I64.alignment(), 8);
    assert_eq!(FieldType::F64.alignment(), 8);
    assert_eq!(FieldType::String.alignment(), 8);
    assert_eq!(FieldType::Bytes.alignment(), 8);
    assert_eq!(FieldType::Option(Box::new(FieldType::U32)).alignment(), 8);
    assert_eq!(FieldType::Vec(Box::new(FieldType::U32)).alignment(), 8);
    assert_eq!(FieldType::Struct("Foo").alignment(), 8);
}

// ─── FieldType::size ────────────────────────────────────────────────────────

#[test]
fn test_size_fixed() {
    assert_eq!(FieldType::Bool.size(), Some(1));
    assert_eq!(FieldType::U8.size(), Some(1));
    assert_eq!(FieldType::I8.size(), Some(1));
    assert_eq!(FieldType::U16.size(), Some(2));
    assert_eq!(FieldType::I16.size(), Some(2));
    assert_eq!(FieldType::U32.size(), Some(4));
    assert_eq!(FieldType::I32.size(), Some(4));
    assert_eq!(FieldType::F32.size(), Some(4));
    assert_eq!(FieldType::U64.size(), Some(8));
    assert_eq!(FieldType::I64.size(), Some(8));
    assert_eq!(FieldType::F64.size(), Some(8));
}

#[test]
fn test_size_variable() {
    assert_eq!(FieldType::String.size(), None);
    assert_eq!(FieldType::Bytes.size(), None);
    assert_eq!(FieldType::Option(Box::new(FieldType::U32)).size(), None);
    assert_eq!(FieldType::Vec(Box::new(FieldType::U32)).size(), None);
    assert_eq!(FieldType::Struct("Foo").size(), None);
}

// ─── FieldType clone and eq ─────────────────────────────────────────────────

#[test]
fn test_field_type_clone() {
    let ft = FieldType::Vec(Box::new(FieldType::Option(Box::new(FieldType::String))));
    let cloned = ft.clone();
    assert_eq!(ft, cloned);
}

#[test]
fn test_field_type_ne() {
    assert_ne!(FieldType::U32, FieldType::I32);
    assert_ne!(FieldType::String, FieldType::Bytes);
    assert_ne!(
        FieldType::Vec(Box::new(FieldType::U8)),
        FieldType::Vec(Box::new(FieldType::U16))
    );
}

// ─── validate_field_type ────────────────────────────────────────────────────

#[test]
fn test_validate_scalar_types() {
    assert!(validate_field_type(&FieldType::Bool).is_ok());
    assert!(validate_field_type(&FieldType::U8).is_ok());
    assert!(validate_field_type(&FieldType::U16).is_ok());
    assert!(validate_field_type(&FieldType::U32).is_ok());
    assert!(validate_field_type(&FieldType::U64).is_ok());
    assert!(validate_field_type(&FieldType::I8).is_ok());
    assert!(validate_field_type(&FieldType::I16).is_ok());
    assert!(validate_field_type(&FieldType::I32).is_ok());
    assert!(validate_field_type(&FieldType::I64).is_ok());
    assert!(validate_field_type(&FieldType::F32).is_ok());
    assert!(validate_field_type(&FieldType::F64).is_ok());
}

#[test]
fn test_validate_string_and_bytes() {
    assert!(validate_field_type(&FieldType::String).is_ok());
    assert!(validate_field_type(&FieldType::Bytes).is_ok());
}

#[test]
fn test_validate_option_valid() {
    let ft = FieldType::Option(Box::new(FieldType::U32));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_nested_option_invalid() {
    let ft = FieldType::Option(Box::new(FieldType::Option(Box::new(FieldType::U32))));
    assert!(validate_field_type(&ft).is_err());
}

#[test]
fn test_validate_vec_valid() {
    let ft = FieldType::Vec(Box::new(FieldType::String));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_vec_of_option_valid() {
    let ft = FieldType::Vec(Box::new(FieldType::Option(Box::new(FieldType::U64))));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_vec_of_vec_valid() {
    let ft = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::I32))));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_struct_valid() {
    let ft = FieldType::Struct("MyStruct");
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_struct_empty_name_invalid() {
    let ft = FieldType::Struct("");
    assert!(validate_field_type(&ft).is_err());
}

#[test]
fn test_validate_option_of_string() {
    let ft = FieldType::Option(Box::new(FieldType::String));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_option_of_vec() {
    let ft = FieldType::Option(Box::new(FieldType::Vec(Box::new(FieldType::Bool))));
    assert!(validate_field_type(&ft).is_ok());
}

#[test]
fn test_validate_deeply_nested() {
    // Vec<Vec<Option<Struct>>>
    let ft = FieldType::Vec(Box::new(FieldType::Vec(Box::new(FieldType::Option(
        Box::new(FieldType::Struct("Deep")),
    )))));
    assert!(validate_field_type(&ft).is_ok());
}
