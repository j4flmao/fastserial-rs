use crate::handlers::category::{create_category, list_categories};
use axum::{
    Router,
    routing::{get, post},
};
use sqlx::MySqlPool;

pub fn create_category_routes(pool: MySqlPool) -> Router {
    Router::new()
        .route("/", post(create_category))
        .route("/", get(list_categories))
        .with_state(pool)
}
