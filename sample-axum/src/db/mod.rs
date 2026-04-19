pub mod schema;
pub mod seeds;

use sqlx::MySqlPool;
use sqlx::mysql::MySqlPoolOptions;
use std::env;

pub async fn init_db() -> MySqlPool {
    let database_url = env::var("DATABASE_URL").expect(
        "DATABASE_URL must be set in .env file (e.g., DATABASE_URL=mysql://root:password@localhost:3306/fastserial_db)",
    );
    MySqlPoolOptions::new()
        .max_connections(50)
        .connect(&database_url)
        .await
        .expect("Failed to connect to MySQL. Please ensure the database is running and the URL is correct.")
}
