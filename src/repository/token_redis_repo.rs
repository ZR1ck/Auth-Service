use deadpool_redis::{redis::cmd, Pool};
use log::error;

use crate::{error::redis_error::RedisError, traits::redis_traits::TokenRedisRepository};

/// `TokenRedisRepo` is an implementation of `TokenRedisRepository`.
/// This repository handles storing, validating, and deleting refresh tokens in Redis.
///
/// Each refresh token is stored with a key prefix `refresh_token:` followed by the token itself.
/// The value is a JSON-like string containing the `user_id` and expiration time (`exp`).
pub struct TokenRedisRepo {
    /// Redis connection pool.
    pool: Pool,
}

impl TokenRedisRepo {
    /// Creates a new `TokenRedisRepo`.
    ///
    /// # Arguments
    ///
    /// * `pool` - The `deadpool_redis::Pool` instance used for obtaining Redis connections.
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

/// Implements the `TokenRedisRepository` trait for `TokenRedisRepo`.
/// This trait defines methods for managing refresh tokens in Redis.
impl TokenRedisRepository for TokenRedisRepo {
    /// Stores a refresh token in Redis with a specified TTL (time-to-live).
    ///
    /// # Arguments
    ///
    /// * `user_id` - The user identifier associated with the token.
    /// * `token` - The refresh token string.
    /// * `ttl` - Time-to-live in seconds.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success.
    /// * `Err(RedisError)` if there's an error with the connection pool or Redis command.
    async fn store_refresh_token(
        &self,
        user_id: &str,
        token: &str,
        ttl: i64,
    ) -> Result<(), RedisError> {
        // Get a connection from the pool.
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;
        // Create the key with prefix "refresh_token:".
        let key = format!("refresh_token:{}", token);
        // Create the value as a JSON-like string.
        let value = format!(r#"{{"user_id": {}, "exp": {}}}"#, user_id, ttl);

        // Use the SETEX command to store the key with expiration.
        cmd("SETEX")
            .arg(&[key, ttl.to_string(), value])
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| RedisError::RedisError)?;

        Ok(())
    }

    /// Checks if a refresh token exists (is still valid) in Redis.
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to check.
    ///
    /// # Returns
    ///
    /// * `Ok(true)` if the token exists.
    /// * `Ok(false)` if the token does not exist.
    /// * `Err(RedisError)` if there's an error with the connection pool or Redis command.
    async fn is_refresh_token_valid(&self, token: &str) -> Result<bool, RedisError> {
        // Get a connection from the pool.
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;
        // Create the key with prefix "refresh_token:".
        let key = format!("refresh_token:{}", token);

        // Check if the key exists using the EXISTS command.
        let existed: bool = cmd("EXISTS")
            .arg(&[key])
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                error!("{}", e);
                RedisError::RedisError
            })?;

        Ok(existed)
    }

    /// Deletes a refresh token from Redis.
    ///
    /// # Arguments
    ///
    /// * `token` - The refresh token to delete.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the token was deleted successfully.
    /// * `Err(RedisError)` if there's an error with the connection pool or Redis command.
    async fn delete_refresh_token(&self, token: &str) -> Result<(), RedisError> {
        // Get a connection from the pool.
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;
        // Create the key with prefix "refresh_token:".
        let key = format!("refresh_token:{}", token);

        // Use DEL command to remove the key from Redis.
        cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| RedisError::RedisError)?;

        Ok(())
    }
}
