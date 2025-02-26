use crate::model::account::Account;

pub trait AuthRepository: Send + Sync {
    async fn insert_account(&self, username: String, password: String) -> Result<u64, sqlx::Error>;
    async fn get_auth_info_by_username(&self, username: &str) -> Result<Account, sqlx::Error>;
    async fn is_account_exist(&self, username: &str) -> Result<(), sqlx::Error>;
}
