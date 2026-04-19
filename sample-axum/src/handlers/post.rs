use crate::auth::AuthUser;
use crate::error::AppError;
use crate::fastjson::{FastBinary, FastJson};
use crate::models::post::{CreatePostRequest, Post, PostDetail};
use crate::models::response::{ApiResponse, CategoryStats, StatsResponse};
use axum::extract::State;
use sqlx::MySqlPool;

pub async fn get_stats(
    State(pool): State<MySqlPool>,
) -> Result<ApiResponse<StatsResponse>, AppError> {
    let (total_users, total_posts, total_categories, top_categories) = tokio::try_join!(
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM users").fetch_one(&pool),
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM posts").fetch_one(&pool),
        sqlx::query_as::<_, (i64,)>("SELECT COUNT(*) FROM categories").fetch_one(&pool),
        sqlx::query_as::<_, CategoryStats>(
            r#"
            SELECT c.name, COUNT(p.id) as count
            FROM categories c
            LEFT JOIN posts p ON c.id = p.category_id
            GROUP BY c.id
            ORDER BY count DESC
            LIMIT 5
            "#,
        )
        .fetch_all(&pool)
    )?;

    Ok(ApiResponse::success(StatsResponse {
        total_users: total_users.0,
        total_posts: total_posts.0,
        total_categories: total_categories.0,
        top_categories,
    }))
}

pub async fn create_post(
    State(pool): State<MySqlPool>,
    auth: AuthUser,
    FastJson(req): FastJson<CreatePostRequest>,
) -> Result<ApiResponse<Post>, AppError> {
    let result =
        sqlx::query("INSERT INTO posts (title, content, user_id, category_id) VALUES (?, ?, ?, ?)")
            .bind(&req.title)
            .bind(&req.content)
            .bind(auth.user_id)
            .bind(req.category_id)
            .execute(&pool)
            .await?;

    let id = result.last_insert_id();

    let post = sqlx::query_as::<_, Post>(
        "SELECT id, title, content, user_id, category_id, created_at FROM posts WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(ApiResponse::success(post))
}

pub async fn list_posts(State(pool): State<MySqlPool>) -> Result<ApiResponse<Vec<Post>>, AppError> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT id, title, content, user_id, category_id, created_at FROM posts",
    )
    .fetch_all(&pool)
    .await?;

    Ok(ApiResponse::success(posts))
}

pub async fn list_posts_detail(
    State(pool): State<MySqlPool>,
) -> Result<ApiResponse<Vec<PostDetail>>, AppError> {
    let posts = sqlx::query_as::<_, PostDetail>(
        r#"
        SELECT 
            p.id, p.title, p.content, p.user_id, u.username as author_name, 
            p.category_id, c.name as category_name, p.created_at 
        FROM posts p
        JOIN users u ON p.user_id = u.id
        LEFT JOIN categories c ON p.category_id = c.id
        ORDER BY p.created_at DESC
        "#,
    )
    .fetch_all(&pool)
    .await?;

    Ok(ApiResponse::success(posts))
}

pub async fn list_posts_binary(
    State(pool): State<MySqlPool>,
) -> Result<FastBinary<Vec<Post>>, AppError> {
    let posts = sqlx::query_as::<_, Post>(
        "SELECT id, title, content, user_id, category_id, created_at FROM posts",
    )
    .fetch_all(&pool)
    .await?;

    Ok(FastBinary(posts))
}

pub async fn create_post_binary(
    State(pool): State<MySqlPool>,
    auth: AuthUser,
    FastBinary(req): FastBinary<CreatePostRequest>,
) -> Result<FastBinary<Post>, AppError> {
    let result =
        sqlx::query("INSERT INTO posts (title, content, user_id, category_id) VALUES (?, ?, ?, ?)")
            .bind(&req.title)
            .bind(&req.content)
            .bind(auth.user_id)
            .bind(req.category_id)
            .execute(&pool)
            .await?;

    let id = result.last_insert_id();

    let post = sqlx::query_as::<_, Post>(
        "SELECT id, title, content, user_id, category_id, created_at FROM posts WHERE id = ?",
    )
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(FastBinary(post))
}
