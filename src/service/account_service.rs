use std::sync::Arc;

use log::error;

use crate::{model::account::Account, traits::account_trait::AccountRepository};

/// Service responsible for handling account-related operations.
/// This service interacts with an `AccountRepository`
pub struct AccountService<T: AccountRepository> {
    /// The repository responsible for interacting with the underlying data source.
    account_repo: Arc<T>,
}

impl<T: AccountRepository> AccountService<T> {
    /// Creates a new `AccountService` instance.
    ///
    /// # Arguments
    ///
    /// * `account_repo` - An `Arc` wrapped repository implementing `AccountRepository`.
    ///
    /// # Returns
    ///
    /// A new instance of `AccountService`.
    pub fn new(account_repo: Arc<T>) -> Self {
        Self { account_repo }
    }

    /// Retrieves account information based on the provided account ID.
    ///
    /// # Arguments
    ///
    /// * `id` - A string slice representing the account ID.
    ///
    /// # Returns
    ///
    /// * `Ok(Account)` - The account information if found.
    /// * `Err(actix_web::error::Error)` - An Actix Web error if the ID conversion fails,
    ///   the account is not found, or a database error occurs.
    pub async fn get_account_info(&self, id: &str) -> Result<Account, actix_web::error::Error> {
        // Parse the provided string ID into an i32.
        let id = id
            .parse::<i32>()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        // Query the repository for the account information.
        let account = self.account_repo.get_account_by_id(id).await;

        // Handle the result from the repository.
        match account {
            Ok(account) => Ok(account), // Successfully retrieved account.
            Err(sqlx::Error::RowNotFound) => {
                // Log and return a 404 Not Found error if account does not exist.
                error!("Account not found");
                Err(actix_web::error::ErrorNotFound("Account not found"))
            }
            Err(e) => {
                // Log and return a 500 Internal Server Error for other unexpected errors.
                error!("{}", e);
                Err(actix_web::error::ErrorInternalServerError(e))
            }
        }
    }
}
