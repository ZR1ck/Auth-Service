use std::sync::Arc;

use log::error;

use crate::{
    model::{
        account::{Account, LoginInfo},
        token::Token,
    },
    traits::{account_trait::AuthRepository, redis_traits::TokenRedisRepository},
    utils,
};

pub struct AuthService<R: AuthRepository, T: TokenRedisRepository> {
    pg_repo: Arc<R>,
    redis_repo: Arc<T>,
}

impl<R: AuthRepository, T: TokenRedisRepository> AuthService<R, T> {
    pub fn new(pg_repo: Arc<R>, redis_repo: Arc<T>) -> Self {
        Self {
            pg_repo,
            redis_repo,
        }
    }

    pub async fn add_account(&self, login_info: LoginInfo) -> Result<u64, actix_web::error::Error> {
        if self
            .pg_repo
            .is_account_exist(&login_info.username)
            .await
            .is_ok()
        {
            error!("Username existed");
            return Err(actix_web::error::ErrorConflict("Username existed"));
        }

        let password_hash =
            utils::password::Hasher::hash_password(&login_info.password).map_err(|e| {
                error!("Hashing error: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

        self.pg_repo
            .insert_account(login_info.username, password_hash)
            .await
            .map_err(|e| {
                error!("Create account error: {}", e);
                actix_web::error::ErrorInternalServerError("Can not create account")
            })
    }

    pub async fn verify_account(
        &self,
        login_info: LoginInfo,
    ) -> Result<Token, actix_web::error::Error> {
        let auth_info: Account = match self
            .pg_repo
            .get_auth_info_by_username(&login_info.username)
            .await
        {
            Ok(auth_info) => auth_info,
            Err(sqlx::Error::RowNotFound) => {
                error!("Row not found");
                return Err(actix_web::error::ErrorNotFound("Not found"));
            }
            Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
        };

        if utils::password::Hasher::verify_password(
            &login_info.password,
            &auth_info.password.unwrap(),
        )
        .map_err(|e| {
            error!("Password verifying error: {}", e);
            actix_web::error::ErrorUnauthorized(e)
        })
        .is_ok()
        {
            let access_token = utils::jwt::JwtUtils::generate_access_token(
                &auth_info.id.to_string(),
                &auth_info.role,
            )
            .unwrap();
            let refresh_token = utils::jwt::JwtUtils::generate_refresh_token(
                &auth_info.id.to_string(),
                &auth_info.role,
            )
            .unwrap();

            self.redis_repo
                .store_refresh_token(
                    &auth_info.id.to_string(),
                    &refresh_token,
                    utils::jwt::JwtUtils::get_refresh_exp(),
                )
                .await
                .map_err(|e| {
                    error!("Redis error: {}", e);
                    actix_web::error::ErrorInternalServerError(e)
                })?;

            return Ok(Token {
                access_token,
                refresh_token,
            });
        }
        Err(actix_web::error::ErrorInternalServerError(
            "Cannot verify account",
        ))
    }
}
