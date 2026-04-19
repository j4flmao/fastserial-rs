use crate::handlers::post::{
    create_post, create_post_binary, get_stats, list_posts, list_posts_binary, list_posts_detail,
};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::MySqlPool;

pub fn create_post_routes(pool: MySqlPool) -> Router {
    Router::new()
        .route("/", post(create_post))
        .route("/", get(list_posts))
        .route("/detail", get(list_posts_detail))
        .route("/stats", get(get_stats))
        .route("/binary/create", post(create_post_binary))
        .route("/binary/list", get(list_posts_binary))
        .with_state(pool)
}
