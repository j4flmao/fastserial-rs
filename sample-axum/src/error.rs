use crate::models::response::ApiResponse;
use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum AppError {
    BadRequest(String),
    Unauthorized(String),
    NotFound(String),
    InternalServerError(String),
    SqlxError(sqlx::Error),
    BcryptError(bcrypt::BcryptError),
    JwtError(jsonwebtoken::errors::Error),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        Self::SqlxError(err)
    }
}

impl From<bcrypt::BcryptError> for AppError {
    fn from(err: bcrypt::BcryptError) -> Self {
        Self::BcryptError(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        Self::JwtError(err)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            AppError::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg),
            AppError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
            AppError::SqlxError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::BcryptError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
            AppError::JwtError(err) => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        let response: ApiResponse<String> = ApiResponse::error(message);
        (status, response).into_response()
    }
}
