use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{EncodingKey, Header};
use lazy_static::lazy_static;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub id: String,
    pub role: String,
    pub exp: usize,
}

lazy_static! {
    pub static ref ACCESS_TOKEN_EXPIRY: Duration = Duration::seconds(20);
    pub static ref REFRESH_TOKEN_EXPIRY: Duration = Duration::minutes(1);
}

pub struct JwtUtils;

impl JwtUtils {
    pub fn get_refresh_exp() -> i64 {
        REFRESH_TOKEN_EXPIRY.num_seconds()
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

        info!("Access token generated");

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

        info!("Refresh token generated");

        jsonwebtoken::encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(secret_key.as_ref()),
        )
    }
}
