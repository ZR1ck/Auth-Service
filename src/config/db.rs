use std::env;

use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

pub async fn create_pool() -> Pool<Postgres> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool")
}
