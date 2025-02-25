use actix_web::{http::StatusCode, ResponseError};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum DbError {
    InternalError,
    InsertError,
    Existed,
    NotFound,
}

impl ResponseError for DbError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match *self {
            DbError::Existed => StatusCode::CONFLICT,
            DbError::NotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
