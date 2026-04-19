use axum::{
    Router,
    routing::{get, post},
};
use sqlx::MySqlPool;

use crate::handlers::user::{get_profile, get_profile_binary, login, register};

pub fn create_user_routes(pool: MySqlPool) -> Router {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/profile", get(get_profile))
        .route("/profile-binary", get(get_profile_binary))
        .with_state(pool)
}
