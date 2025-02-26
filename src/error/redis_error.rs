use derive_more::{Display, Error};

#[derive(Debug, Error, Display)]
pub enum RedisError {
    PoolError,
    RedisError,
}
