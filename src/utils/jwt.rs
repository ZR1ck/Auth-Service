use std::env;

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub id: String,
    pub role: String,
    pub exp: usize,
}

pub fn generate_access_token(
    user_id: &str,
    role: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let secret_key = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let claims = Claims {
        id: user_id.to_string(),
        role: role.to_string(),
        exp: (Utc::now() + Duration::minutes(1)).timestamp() as usize,
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
        exp: (Utc::now() + Duration::days(1)).timestamp() as usize,
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
