use std::{
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{DecodingKey, Validation};
use log::{error, info};

use crate::{
    error::service_error::ServiceError,
    traits::redis_traits::TokenRedisRepository,
    utils::jwt::{self, Claims},
};

/// Service responsible for handling token verification and generation.
/// This service supports both access tokens and refresh tokens,
/// leveraging Redis to validate the existence of refresh tokens.
pub struct TokenService<T: TokenRedisRepository> {
    /// Repository for interacting with Redis, specifically for storing and validating refresh tokens.
    token_redis_repo: Arc<T>,
}

impl<T: TokenRedisRepository> TokenService<T> {
    /// Creates a new instance of `TokenService`.
    ///
    /// # Arguments
    ///
    /// * `token_redis_repo` - A shared reference to the Redis repository used for token storage.
    ///
    /// # Returns
    ///
    /// * New instance of `TokenService`.
    pub fn new(token_redis_repo: Arc<T>) -> Self {
        Self { token_redis_repo }
    }

    /// Verifies a refresh token by:
    /// - Checking if it exists in Redis.
    /// - Decoding and validating the token using the `JWT_REFRESH_SECRET`.
    /// - If the token is valid, a new access token is generated and returned.
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to verify.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` - The newly generated access token.
    /// * `Err(ServiceError)` - An Actix web error if validation fails.
    pub async fn verify_refresh_token(&self, token: &str) -> Result<String, ServiceError> {
        // Load the refresh token secret key from environment variables.
        let secret_key = env::var("JWT_REFRESH_SECRET").expect("JWT_REFRESH_SECRET must be set");

        // Check if the refresh token exists in Redis.
        let is_valid = self
            .token_redis_repo
            .is_refresh_token_valid(token)
            .await
            .map_err(|e| {
                // Log any error that occurs during Redis operation.
                error!("{}", e);
                ServiceError::RedisError
            })?;

        // If the token does not exist in Redis, reject it.
        if !is_valid {
            info!("Invalid refresh token");
            return Err(ServiceError::UnAuthorizedError);
        }

        // Decode and validate the token using the secret key.
        let token_data = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| ServiceError::JwtError(e))?;

        let claims: Claims = token_data.claims;

        // Generate a new access token using the claims from the refresh token.
        jwt::JwtUtils::generate_access_token(&claims.id, &claims.role)
            .map_err(|e| ServiceError::JwtError(e))
    }

    /// Verifies an access token by:
    /// - Decoding and validating the token using the `JWT_ACCESS_SECRET`.
    /// - Checking that the token's expiration (`exp`) is still valid.
    ///
    /// # Arguments
    ///
    /// * `token` - The access token to verify.
    ///
    /// # Returns
    ///
    /// * `Ok(Claims)` - Decoded claims if the token is valid.
    /// * `Err(ServiceError)` - An Actix web error if validation fails.
    pub fn verify_access_token(&self, token: &str) -> Result<Claims, ServiceError> {
        // Load the access token secret key from environment variables.
        let secret_key = env::var("JWT_ACCESS_SECRET").expect("JWT_ACCESS_SECRET must be set");

        // Decode and validate the access token.
        let token_data: Claims = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| ServiceError::JwtError(e))?
        .claims;

        // Check the token's expiration time against the current time.
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;

        if token_data.exp < now {
            info!("Expired access token");
            return Err(ServiceError::UnAuthorizedError);
        }

        // Return the valid token claims.
        Ok(token_data)
    }
}
