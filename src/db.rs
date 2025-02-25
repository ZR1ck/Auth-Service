use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, SaltString},
    Argon2,
};
use log::{error, info};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
use std::env;

use crate::{
    error::DbError,
    model::{Account, NewAccount},
};

pub async fn create_pool() -> Pool<Postgres> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to create database pool")
}

pub async fn add_account(pool: &Pool<Postgres>, new_account: NewAccount) -> Result<u64, DbError> {
    let select_stmt = include_str!("../sql/find_account.sql");

    let acc: Option<Account> = sqlx::query_as(&select_stmt)
        .bind(&new_account.username)
        .fetch_optional(pool)
        .await
        .map_err(|e| {
            error!("{}", e);
            e
        })?;

    if acc.is_some() {
        info!("Item found: {:?}", acc.unwrap());
        return Err(DbError::Exists);
    }

    let insert_stmt = include_str!("../sql/insert_account.sql");

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();

    let password_hash = argon2
        .hash_password(new_account.password.as_bytes(), &salt)
        .map_err(|_| {
            error!("Password hashing error");
            DbError::Hashing
        })?
        .to_string();

    let x = sqlx::query(&insert_stmt)
        .bind(new_account.username)
        .bind(password_hash)
        .bind(String::from("user"))
        .execute(pool)
        .await
        .map_err(|e| {
            error!("Database insert error: {}", e);
            e
        })?;

    Ok(x.rows_affected())
}
