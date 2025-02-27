use std::{
    env,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use jsonwebtoken::{DecodingKey, Validation};
use log::{error, info};

use crate::{
    traits::redis_traits::TokenRedisRepository,
    utils::jwt::{self, Claims},
};

pub struct TokenService<T: TokenRedisRepository> {
    token_redis_repo: Arc<T>,
}

impl<T: TokenRedisRepository> TokenService<T> {
    pub fn new(token_redis_repo: Arc<T>) -> Self {
        Self { token_redis_repo }
    }

    pub async fn verify_refresh_token(
        &self,
        token: &str,
    ) -> Result<String, actix_web::error::Error> {
        let secret_key = env::var("JWT_REFRESH_SECRET").expect("JWT_SECRET must be set");

        let is_valid = self
            .token_redis_repo
            .is_refresh_token_valid(token)
            .await
            .map_err(|e| {
                error!("{}", e);
                actix_web::error::ErrorInternalServerError(e)
            })?;

        if !is_valid {
            info!("Invalid refresh token");
            return Err(actix_web::error::ErrorUnauthorized("Invalid refresh token"));
        }

        let token_data = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;

        let claims: Claims = token_data.claims;

        jwt::JwtUtils::generate_access_token(&claims.id, &claims.role)
            .map_err(|e| actix_web::error::ErrorInternalServerError(e))
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims, actix_web::error::Error> {
        let secret_key = env::var("JWT_ACCESS_SECRET").expect("JWT_SECRET must be set");

        let token_data: Claims = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?
        .claims;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as usize;
        if token_data.exp < now {
            info!("Expire access token");
            return Err(actix_web::error::ErrorUnauthorized("Invalid access token"));
        }

        Ok(token_data)
    }
}
