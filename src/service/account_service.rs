use std::sync::Arc;

use log::error;

use crate::{model::account::Account, traits::account_trait::AccountRepository};

pub struct AccountService<T: AccountRepository> {
    account_repo: Arc<T>,
}

impl<T: AccountRepository> AccountService<T> {
    pub fn new(account_repo: Arc<T>) -> Self {
        Self { account_repo }
    }

    pub async fn get_account_info(&self, id: &str) -> Result<Account, actix_web::error::Error> {
        let id = id
            .parse::<i32>()
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let account = self.account_repo.get_account_by_id(id).await;
        match account {
            Ok(account) => Ok(account),
            Err(sqlx::Error::RowNotFound) => {
                error!("Account not found");
                return Err(actix_web::error::ErrorNotFound("Account not found"));
            }
            Err(e) => {
                error!("{}", e);
                Err(actix_web::error::ErrorInternalServerError(e))
            }
        }
    }
}
