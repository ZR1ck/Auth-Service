use crate::error::redis_error::RedisError;

pub trait TokenRedisRepository: Send + Sync {
    async fn store_refresh_token(
        &self,
        user_id: &str,
        token: &str,
        ttl: i64,
    ) -> Result<(), RedisError>;
    async fn is_refresh_token_valid(&self, token: &str) -> Result<bool, RedisError>;
    async fn delete_refresh_token(&self, token: &str) -> Result<(), RedisError>;
}
