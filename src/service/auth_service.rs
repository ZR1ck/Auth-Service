use log::error;
use sqlx::{Pool, Postgres};

use crate::{error::db_error::DbError, model::account::LoginInfo, repository::account_repo, utils};

pub async fn add_account(pool: &Pool<Postgres>, login_info: LoginInfo) -> Result<u64, DbError> {
    if account_repo::get_account_by_username(pool, &login_info.username)
        .await
        .is_ok()
    {
        error!("Username existed");
        return Err(DbError::Existed);
    }

    let password_hash =
        utils::password::hash_password(login_info.password).map_err(|_| DbError::InternalError)?;

    account_repo::insert_account(pool, login_info.username, password_hash)
        .await
        .map_err(|e| {
            error!("Create account error: {}", e);
            DbError::InsertError
        })
}
