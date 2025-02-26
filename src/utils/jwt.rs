use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub role: String,
    pub exp: usize,
}

lazy_static! {
    pub static ref ACCESS_TOKEN_EXPIRY: Duration = Duration::seconds(30);
    pub static ref REFRESH_TOKEN_EXPIRY: Duration = Duration::minutes(1);
}

pub struct Jwt;

impl Jwt {
    pub fn get_access_exp() -> Duration {
        *ACCESS_TOKEN_EXPIRY
    }

    pub fn get_refresh_exp() -> Duration {
        *REFRESH_TOKEN_EXPIRY
    }

    pub fn generate_access_token(
        user_id: &str,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let secret_key = env::var("JWT_ACCESS_SECRET").expect("JWT_ACCESS_SECRET must be set");
        let claims = Claims {
            id: user_id.to_string(),
            role: role.to_string(),
            exp: (Utc::now() + *ACCESS_TOKEN_EXPIRY).timestamp() as usize,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
    }

    pub fn generate_refresh_token(
        user_id: &str,
        role: &str,
    ) -> Result<String, jsonwebtoken::errors::Error> {
        let secret_key = env::var("JWT_REFRESH_SECRET").expect("JWT_REFRESH_SECRET must be set");
        let claims = Claims {
            id: user_id.to_string(),
            role: role.to_string(),
            exp: (Utc::now() + *REFRESH_TOKEN_EXPIRY).timestamp() as usize,
        };

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
    }

    pub fn verify_refresh_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
        let secret_key = env::var("JWT_REFRESH_SECRET").expect("JWT_SECRET must be set");
        let token_data = jsonwebtoken::decode(
            token,
            &DecodingKey::from_secret(secret_key.as_ref()),
            &Validation::default(),
        )?;

        Ok(token_data.claims)
    }
}
