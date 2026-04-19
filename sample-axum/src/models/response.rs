use crate::fastjson::FastJson;
use axum::response::{IntoResponse, Response};
use fastserial::Encode;

#[derive(Debug, Encode)]
pub struct ApiResponse<T: Encode> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

#[derive(Debug, Encode)]
pub struct StatsResponse {
    pub total_users: i64,
    pub total_posts: i64,
    pub total_categories: i64,
    pub top_categories: Vec<CategoryStats>,
}

#[derive(Debug, Encode, sqlx::FromRow)]
pub struct CategoryStats {
    pub name: String,
    pub count: i64,
}

impl<T: Encode> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            message: "Success".to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            message,
            data: None,
        }
    }
}

impl<T: Encode> IntoResponse for ApiResponse<T> {
    fn into_response(self) -> Response {
        FastJson(self).into_response()
    }
}
