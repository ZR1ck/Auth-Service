use log::{error, info};
use sqlx::{Pool, Postgres};

use crate::{
    model::{
        account::{Account, LoginInfo},
        token::Token,
    },
    repository::{account_repo, redis_repo},
    utils,
};

pub async fn add_account(
    pool: &Pool<Postgres>,
    login_info: LoginInfo,
) -> Result<u64, actix_web::error::Error> {
    if account_repo::get_account_by_username(pool, &login_info.username)
        .await
        .is_ok()
    {
        error!("Username existed");
        return Err(actix_web::error::ErrorConflict("Username existed"));
    }

    let password_hash = utils::password::hash_password(&login_info.password)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

    account_repo::insert_account(pool, login_info.username, password_hash)
        .await
        .map_err(|e| {
            error!("Create account error: {}", e);
            actix_web::error::ErrorInternalServerError("Can not create account")
        })
}

pub async fn verify_account(
    postgres_pool: &Pool<Postgres>,
    redis_pool: &deadpool_redis::Pool,
    login_info: LoginInfo,
) -> Result<Token, actix_web::error::Error> {
    let auth_info: Account = match account_repo::get_auth_info(postgres_pool, &login_info.username)
        .await
    {
        Ok(auth_info) => auth_info,
        Err(sqlx::Error::RowNotFound) => return Err(actix_web::error::ErrorNotFound("Not found")),
        Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
    };

    if utils::password::verify_password(&login_info.password, &auth_info.password.unwrap())
        .map_err(|e| {
            error!("Password verifying error: {}", e);
            actix_web::error::ErrorUnauthorized(e)
        })
        .is_ok()
    {
        let access_token =
            utils::jwt::generate_access_token(&auth_info.id.to_string(), &auth_info.role).unwrap();
        let refresh_token =
            utils::jwt::generate_refresh_token(&auth_info.id.to_string(), &auth_info.role).unwrap();

        redis_repo::store_refresh_token(redis_pool, &auth_info.id.to_string(), &refresh_token, 30)
            .await
            .unwrap();

        return Ok(Token {
            access_token,
            refresh_token,
        });
    }
    Err(actix_web::error::ErrorInternalServerError(
        "Cannot verify account",
    ))
}
