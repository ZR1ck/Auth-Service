use deadpool_redis::{redis::cmd, Pool};
use log::error;

use crate::error::redis_error::RedisError;

pub async fn store_refresh_token(
    pool: &Pool,
    user_id: &str,
    token: &str,
    ttl: u64,
) -> Result<(), RedisError> {
    let mut conn = pool.get().await.map_err(|_| RedisError::PoolError)?;

    let key = format!("refresh_token:{}", token);
    let value = format!(r#"{{"user_id": {}, "exp": {}}}"#, user_id, ttl);

    cmd("SETEX")
        .arg(&[key, ttl.to_string(), value])
        .query_async::<()>(&mut conn)
        .await
        .map_err(|_| RedisError::RedisError)?;

    Ok(())
}

pub async fn is_refresh_token_valid(pool: &Pool, token: &str) -> Result<bool, RedisError> {
    let mut conn = pool.get().await.map_err(|_| RedisError::PoolError)?;
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

pub async fn delete_refresh_token(pool: &Pool, token: &str) -> Result<(), RedisError> {
    let mut conn = pool.get().await.map_err(|_| RedisError::PoolError)?;
    let key = format!("refresh_token:{}", token);

    cmd("DEL")
        .arg(&key)
        .query_async::<()>(&mut conn)
        .await
        .map_err(|_| RedisError::RedisError)?;

    Ok(())
}
