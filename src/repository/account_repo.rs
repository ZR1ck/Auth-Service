use sqlx::{Pool, Postgres};

use crate::{model::account::Account, traits::account_trait::AuthRepository};

/// `AccountRepo` provides an implementation of `AuthRepository` for PostgreSQL.
/// It handles basic account-related operations such as inserting a new account,
/// fetching account informations by values, and checking if an account exists.
pub struct AccountRepo {
    /// Connection pool for interacting with the PostgreSQL database.
    pool: Pool<Postgres>,
}

impl AccountRepo {
    /// Creates a new `AccountRepo`.
    ///
    /// # Arguments
    ///
    /// * `pool` - The SQLx connection pool for PostgreSQL.
    ///
    /// # Returns
    ///
    /// * A new instance of `AccountRepo`.
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

impl AuthRepository for AccountRepo {
    /// Inserts a new account into the database.
    ///
    /// # Arguments
    ///
    /// * `username` - The username for the new account.
    /// * `password` - The hashed password for the new account.
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Number of rows affected (should be 1 on success).
    /// * `Err(sqlx::Error)` - If an error occurs during database operation.
    async fn insert_account(&self, username: String, password: String) -> Result<u64, sqlx::Error> {
        // Load SQL query from external file.
        let insert_stmt = include_str!("../../sql/insert_account.sql");

        // Execute the query, binding the username, password, and default "user" role.
        let x = sqlx::query(&insert_stmt)
            .bind(username)
            .bind(password)
            .bind(String::from("user")) // Default role assigned to new accounts.
            .execute(&self.pool)
            .await?;

        // Return the number of rows affected.
        Ok(x.rows_affected())
    }

    /// Retrieves account information by username.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to search for.
    ///
    /// # Returns
    ///
    /// * `Ok(Account)` - The account information if found.
    /// * `Err(sqlx::Error)` - Returns `RowNotFound` if no account exists with this username, or other SQLx errors.
    async fn get_auth_info_by_username(&self, username: &str) -> Result<Account, sqlx::Error> {
        // Load SQL query from external file.
        let select_stmt = include_str!("../../sql/get_auth_info.sql");

        // Perform the query and try to fetch the account as `Account` model.
        let result: Option<Account> = sqlx::query_as(&select_stmt)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        // Handle the result: return the account if found, otherwise return RowNotFound.
        match result {
            Some(info) => Ok(info),
            None => Err(sqlx::Error::RowNotFound),
        }
    }

    /// Checks if an account with the given username exists.
    ///
    /// # Arguments
    ///
    /// * `username` - The username to check.
    ///
    /// # Returns
    ///
    /// * `Ok(())` - If the account exists.
    /// * `Err(sqlx::Error)` - Returns `RowNotFound` if no such account exists, or other SQLx errors.
    async fn is_account_exist(&self, username: &str) -> Result<(), sqlx::Error> {
        // Load the same query used to fetch account information.
        let select_stmt = include_str!("../../sql/get_auth_info.sql");

        // Check if the account exists by trying to fetch it.
        let account: Option<Account> = sqlx::query_as(&select_stmt)
            .bind(username)
            .fetch_optional(&self.pool)
            .await?;

        // If found, return Ok, otherwise return RowNotFound.
        match account {
            Some(_) => Ok(()),
            None => Err(sqlx::Error::RowNotFound),
        }
    }
}
