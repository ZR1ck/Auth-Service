use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Token {
    pub access_token: String,
    pub refresh_token: String,
}
