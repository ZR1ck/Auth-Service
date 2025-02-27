use std::sync::Arc;

use log::error;

use crate::{
    model::{
        account::{Account, LoginInfo},
        token::Token,
    },
    traits::{account_trait::AccountRepository, redis_traits::TokenRedisRepository},
    utils,
};

/// Service responsible for handling user authentication and account management.
/// It interacts with both the PostgreSQL repository (for account data) and the Redis repository (for refresh token storage).
pub struct AuthService<R: AccountRepository, T: TokenRedisRepository> {
    /// Repository for PostgreSQL operations related to user accounts.
    pg_repo: Arc<R>,
    /// Repository for Redis operations related to refresh token storage.
    redis_repo: Arc<T>,
}

impl<R: AccountRepository, T: TokenRedisRepository> AuthService<R, T> {
    /// Creates a new instance of `AuthService`.
    ///
    /// # Arguments
    ///
    /// * `pg_repo` - A shared Arc reference to the PostgreSQL repository.
    /// * `redis_repo` - A shared Arc reference to the Redis repository.
    ///
    /// # Returns
    ///
    /// * New instance of `AuthService`.
    pub fn new(pg_repo: Arc<R>, redis_repo: Arc<T>) -> Self {
        Self {
            pg_repo,
            redis_repo,
        }
    }

    /// Adds a new account to the system after checking that the username does not already exist.
    ///
    /// # Arguments
    ///
    /// * `login_info` - The username and password provided by the user during registration.
    ///
    /// # Returns
    ///
    /// * `Ok(u64)` - Number of rows affected (should be 1 if successful).
    /// * `Err(actix_web::error::Error)` - Actix error if username already exists or other issues occur.
    pub async fn add_account(&self, login_info: LoginInfo) -> Result<u64, actix_web::error::Error> {
        // Check if the username already exists in the database.
        if self
            .pg_repo
            .is_account_exist(&login_info.username)
            .await
            .is_ok()
        {
            error!("Username existed");
            return Err(actix_web::error::ErrorConflict("Username existed"));
        }

        // Hash the provided password.
        let password_hash =
            utils::password::Hasher::hash_password(&login_info.password).map_err(|e| {
                error!("Hashing error: {}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

        // Insert the new account into the database.
        self.pg_repo
            .insert_account(login_info.username, password_hash)
            .await
            .map_err(|e| {
                error!("Create account error: {}", e);
                actix_web::error::ErrorInternalServerError("Cannot create account")
            })
    }

    /// Verifies the provided username and password, and if successful, generates access and refresh tokens.
    /// The refresh token is stored in Redis.
    ///
    /// # Arguments
    ///
    /// * `login_info` - The username and password provided by the user during login.
    ///
    /// # Returns
    ///
    /// * `Ok(Token)` - Struct containing the generated access and refresh tokens.
    /// * `Err(actix_web::error::Error)` - Actix error if the account is not found or the credentials are invalid.
    pub async fn verify_account(
        &self,
        login_info: LoginInfo,
    ) -> Result<Token, actix_web::error::Error> {
        // Fetch authentication information from the database.
        let auth_info: Account = match self
            .pg_repo
            .get_account_by_username(&login_info.username)
            .await
        {
            Ok(auth_info) => auth_info,
            Err(sqlx::Error::RowNotFound) => {
                error!("Row not found");
                return Err(actix_web::error::ErrorNotFound("Not found"));
            }
            Err(e) => return Err(actix_web::error::ErrorInternalServerError(e)),
        };

        // Verify the provided password against the stored hash.
        if utils::password::Hasher::verify_password(
            &login_info.password,
            &auth_info.password.unwrap(), // Assume password is always Some.
        )
        .map_err(|e| {
            error!("Password verifying error: {}", e);
            actix_web::error::ErrorUnauthorized(e)
        })
        .is_ok()
        {
            // Generate new access and refresh tokens upon successful verification.
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

            // Store the refresh token in Redis with an appropriate expiration time.
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

            // Return the generated tokens.
            return Ok(Token {
                access_token,
                refresh_token,
            });
        }

        // Return an error if password verification fails.
        Err(actix_web::error::ErrorInternalServerError(
            "Cannot verify account",
        ))
    }

    /// Remove refresh token in Redis
    ///
    /// # Arguments
    ///
    /// * `refresh_token` - The token to remove
    ///
    /// # Returns
    ///
    /// *`Ok(())` - If remove success
    /// *`Err(actix_web::error::Error)` - Actix error if the account is not found or the credentials are invalid.
    pub async fn logout(&self, refresh_token: &str) -> Result<(), actix_web::error::Error> {
        self.redis_repo
            .delete_refresh_token(refresh_token)
            .await
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))
    }
}
