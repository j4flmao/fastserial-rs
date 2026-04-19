use bcrypt::{DEFAULT_COST, hash};
use sqlx::MySqlPool;

/// Seed initial data for the database
pub async fn seed_data(pool: &MySqlPool) -> anyhow::Result<()> {
    // Seed Categories
    let categories = vec!["Technology", "Rust", "Web Development", "Microservices"];
    for cat in categories {
        sqlx::query("INSERT IGNORE INTO categories (name) VALUES (?)")
            .bind(cat)
            .execute(pool)
            .await?;
    }

    // Seed Admin User if not exists
    let admin_email = "admin@fastserial.rs";
    let user_exists = sqlx::query("SELECT id FROM users WHERE email = ?")
        .bind(admin_email)
        .fetch_optional(pool)
        .await?;

    if user_exists.is_none() {
        let hashed_password = hash("admin123", DEFAULT_COST)?;
        let result =
            sqlx::query("INSERT INTO users (username, email, password_hash) VALUES (?, ?, ?)")
                .bind("admin")
                .bind(admin_email)
                .bind(hashed_password)
                .execute(pool)
                .await?;

        let admin_id = result.last_insert_id();

        // Seed a welcome post
        let tech_cat: (i64,) =
            sqlx::query_as("SELECT id FROM categories WHERE name = 'Technology'")
                .fetch_one(pool)
                .await?;

        sqlx::query("INSERT INTO posts (title, content, user_id, category_id) VALUES (?, ?, ?, ?)")
            .bind("Welcome to FastSerial API")
            .bind(
                "This is a seed post to demonstrate the API capabilities with Axum and FastSerial.",
            )
            .bind(admin_id)
            .bind(tech_cat.0)
            .execute(pool)
            .await?;

        // Seed 100 more posts for performance testing
        for i in 1..=100 {
            sqlx::query(
                "INSERT INTO posts (title, content, user_id, category_id) VALUES (?, ?, ?, ?)",
            )
            .bind(format!("Performance Test Post #{}", i))
            .bind(format!(
                "This is a dummy post for testing performance. Item number: {}",
                i
            ))
            .bind(admin_id)
            .bind(tech_cat.0)
            .execute(pool)
            .await?;
        }
    }

    Ok(())
}
