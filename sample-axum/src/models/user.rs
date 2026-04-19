use chrono::{DateTime, Utc};
use fastserial::{Decode, Encode};
use sqlx::FromRow;

#[derive(Debug, Encode, Decode, FromRow, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[fastserial(skip)]
    pub password_hash: String,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Encode, Decode)]
pub struct AuthResponse {
    pub token: String,
    pub user: User,
}

#[derive(Debug, Encode, Decode)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Encode, Decode)]
pub struct RegisterRequest {
    pub username: String,
    pub email: String,
    pub password: String,
}
