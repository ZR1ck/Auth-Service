use crate::model::account::Account;

pub trait AccountRepository: Send + Sync {
    async fn insert_account(&self, username: String, password: String) -> Result<u64, sqlx::Error>;
    async fn get_account_by_username(&self, username: &str) -> Result<Account, sqlx::Error>;
    async fn get_account_by_id(&self, id: i32) -> Result<Account, sqlx::Error>;
    async fn is_account_exist(&self, username: &str) -> Result<(), sqlx::Error>;
}
