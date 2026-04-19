use crate::auth::AuthUser;
use crate::error::AppError;
use crate::fastjson::FastJson;
use crate::models::category::{Category, CreateCategoryRequest};
use crate::models::response::ApiResponse;
use axum::extract::State;
use sqlx::MySqlPool;

pub async fn create_category(
    State(pool): State<MySqlPool>,
    _auth: AuthUser,
    FastJson(req): FastJson<CreateCategoryRequest>,
) -> Result<ApiResponse<Category>, AppError> {
    let result = sqlx::query("INSERT INTO categories (name) VALUES (?)")
        .bind(&req.name)
        .execute(&pool)
        .await?;

    let id = result.last_insert_id();

    let category =
        sqlx::query_as::<_, Category>("SELECT id, name, created_at FROM categories WHERE id = ?")
            .bind(id)
            .fetch_one(&pool)
            .await?;

    Ok(ApiResponse::success(category))
}

pub async fn list_categories(
    State(pool): State<MySqlPool>,
) -> Result<ApiResponse<Vec<Category>>, AppError> {
    let categories = sqlx::query_as::<_, Category>("SELECT id, name, created_at FROM categories")
        .fetch_all(&pool)
        .await?;

    Ok(ApiResponse::success(categories))
}
