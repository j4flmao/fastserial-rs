use axum::extract::State;
use bcrypt::{DEFAULT_COST, hash, verify};
use sqlx::MySqlPool;

use crate::auth::create_jwt;
use crate::error::AppError;
use crate::fastjson::{FastBinary, FastJson};
use crate::models::response::ApiResponse;
use crate::models::user::{AuthResponse, LoginRequest, RegisterRequest, User};

pub async fn register(
    State(pool): State<MySqlPool>,
    FastJson(req): FastJson<RegisterRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let password_hash = hash(req.password, DEFAULT_COST)?;

    let result = sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
        .bind(&req.username)
        .bind(&req.email)
        .bind(&password_hash)
        .execute(&pool)
        .await?;

    let user_id = result.last_insert_id() as i64;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?",
    )
    .bind(user_id)
    .fetch_one(&pool)
    .await?;

    let token = create_jwt(user.id)?;

    Ok(ApiResponse::success(AuthResponse { token, user }))
}

pub async fn login(
    State(pool): State<MySqlPool>,
    FastJson(req): FastJson<LoginRequest>,
) -> Result<ApiResponse<AuthResponse>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE email = ?",
    )
    .bind(&req.email)
    .fetch_optional(&pool)
    .await?
    .ok_or(AppError::Unauthorized("Invalid credentials".to_string()))?;

    let valid = verify(req.password, &user.password_hash)?;

    if !valid {
        return Err(AppError::Unauthorized("Invalid credentials".to_string()));
    }

    let token = create_jwt(user.id)?;

    Ok(ApiResponse::success(AuthResponse { token, user }))
}

pub async fn get_profile(
    State(pool): State<MySqlPool>,
    auth_user: crate::auth::AuthUser,
) -> Result<ApiResponse<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?",
    )
    .bind(auth_user.user_id)
    .fetch_one(&pool)
    .await?;

    Ok(ApiResponse::success(user))
}

pub async fn get_profile_binary(
    State(pool): State<MySqlPool>,
    auth_user: crate::auth::AuthUser,
) -> Result<FastBinary<User>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, created_at FROM users WHERE id = ?",
    )
    .bind(auth_user.user_id)
    .fetch_one(&pool)
    .await?;

    Ok(FastBinary(user))
}
