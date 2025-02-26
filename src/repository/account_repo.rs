use sqlx::{Pool, Postgres};

use crate::{model::account::Account, traits::account_trait::AuthRepository};

pub struct AccountRepo {
    pool: Pool<Postgres>,
}

impl AccountRepo {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl AuthRepository for AccountRepo {
    async fn insert_account(&self, username: String, password: String) -> Result<u64, sqlx::Error> {
        let insert_stmt = include_str!("../../sql/insert_account.sql");
        let x = sqlx::query(&insert_stmt)
            .bind(username)
            .bind(password)
            .bind(String::from("user"))
            .execute(&self.pool)
            .await?;

        Ok(x.rows_affected())
    }

    async fn get_auth_info_by_username(&self, username: &str) -> Result<Account, sqlx::Error> {
        let select_stmt = include_str!("../../sql/get_auth_info.sql");

        let result: Option<Account> = sqlx::query_as(&select_stmt)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        match result {
            Some(info) => Ok(info),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    async fn is_account_exist(&self, username: &str) -> Result<(), sqlx::Error> {
        let select_stmt = include_str!("../../sql/get_auth_info.sql");

        let account: Option<Account> = sqlx::query_as(&select_stmt)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        match account {
            Some(_) => Ok(()),
            None => Err(sqlx::Error::RowNotFound),
        }
    }
}
