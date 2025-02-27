use deadpool_redis::{redis::cmd, Pool};
use log::error;

use crate::{error::redis_error::RedisError, traits::redis_traits::TokenRedisRepository};

pub struct TokenRedisRepo {
    pool: Pool,
}

impl TokenRedisRepo {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl TokenRedisRepository for TokenRedisRepo {
    async fn store_refresh_token(
        &self,
        user_id: &str,
        token: &str,
        ttl: i64,
    ) -> Result<(), RedisError> {
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;

        let key = format!("refresh_token:{}", token);
        let value = format!(r#"{{"user_id": {}, "exp": {}}}"#, user_id, ttl);

        cmd("SETEX")
            .arg(&[key, ttl.to_string(), value])
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| RedisError::RedisError)?;

        Ok(())
    }

    async fn is_refresh_token_valid(&self, token: &str) -> Result<bool, RedisError> {
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;
        let key = format!("refresh_token:{}", token);

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

    async fn delete_refresh_token(&self, token: &str) -> Result<(), RedisError> {
        let mut conn = self.pool.get().await.map_err(|_| RedisError::PoolError)?;
        let key = format!("refresh_token:{}", token);

        cmd("DEL")
            .arg(&key)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|_| RedisError::RedisError)?;

        Ok(())
    }
}
