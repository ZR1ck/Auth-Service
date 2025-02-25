use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: i32,
    pub username: String,
    pub password: Option<String>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct NewAccount {
    pub username: String,
    pub password: String,
}
