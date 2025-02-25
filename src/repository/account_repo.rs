use sqlx::{Pool, Postgres};

use crate::model::account::Account;

pub async fn get_account_by_username(
    pool: &Pool<Postgres>,
    username: &String,
) -> Result<Account, sqlx::Error> {
    let select_stmt = include_str!("../../sql/find_account.sql");

    let account: Option<Account> = sqlx::query_as(&select_stmt)
        .bind(username)
        .fetch_optional(pool)
        .await?;

    match account {
        Some(data) => Ok(data),
        None => Err(sqlx::Error::RowNotFound),
    }
}

pub async fn insert_account(
    pool: &Pool<Postgres>,
    username: String,
    password: String,
) -> Result<u64, sqlx::Error> {
    let insert_stmt = include_str!("../../sql/insert_account.sql");
    let x = sqlx::query(&insert_stmt)
        .bind(username)
        .bind(password)
        .bind(String::from("user"))
        .execute(pool)
        .await?;

    Ok(x.rows_affected())
}
