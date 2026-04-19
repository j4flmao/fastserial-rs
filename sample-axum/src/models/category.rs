use chrono::{DateTime, Utc};
use fastserial::{Decode, Encode};
use sqlx::FromRow;

#[derive(Debug, Encode, Decode, FromRow, Clone)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Encode, Decode)]
pub struct CreateCategoryRequest {
    pub name: String,
}
