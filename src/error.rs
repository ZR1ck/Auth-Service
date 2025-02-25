use actix_web::{http::StatusCode, ResponseError};

#[derive(Debug, thiserror::Error)]
pub enum DbError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Password hashing error")]
    Hashing,
    #[error("Item existed")]
    Exists,
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            DbError::Exists => StatusCode::CONFLICT,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
