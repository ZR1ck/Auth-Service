use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshToken {
    pub refresh_token: String,
}
