//! # Schema Validation
//!
//! This module provides validation for schema types.

use super::types::FieldType;

/// Validates a field type for correctness.
pub fn validate_field_type(ty: &FieldType) -> Result<(), ValidationError> {
    match ty {
        FieldType::Option(inner) | FieldType::Vec(inner) => {
            validate_field_type(inner)?;
            if matches!(ty, FieldType::Option(_)) && matches!(&**inner, FieldType::Option(_)) {
                return Err(ValidationError::NestedOption);
            }
            Ok(())
        }
        FieldType::Struct(name) => {
            if name.is_empty() {
                Err(ValidationError::EmptyStructName)
            } else {
                Ok(())
            }
        }
        _ => Ok(()),
    }
}

/// Validation error types.
#[derive(Debug)]
pub enum ValidationError {
    /// Nested Option types are not allowed.
    NestedOption,
    /// Struct name cannot be empty.
    EmptyStructName,
}
