use chrono::{DateTime, Utc};
use fastserial::{Decode, Encode};
use sqlx::FromRow;

#[derive(Debug, Encode, Decode, FromRow, Clone)]
pub struct Post {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub user_id: i64,
    pub category_id: Option<i64>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Encode, Decode, FromRow, Clone)]
pub struct PostDetail {
    pub id: i64,
    pub title: String,
    pub content: String,
    pub user_id: i64,
    pub author_name: String,
    pub category_id: Option<i64>,
    pub category_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Encode, Decode)]
pub struct CreatePostRequest {
    pub title: String,
    pub content: String,
    pub category_id: Option<i64>,
}
