use actix_web::{HttpResponse, ResponseError};
use derive_more::{Display, Error};

#[derive(Debug, Display, Error)]
pub enum ServiceError {
    #[display("Invalid account id format")]
    InvalidIdFormat(std::num::ParseIntError),

    #[display("Account not found")]
    NotFound,

    #[display("Database error: {_0}")]
    DatabaseError(sqlx::Error),

    #[display("Jwt error")]
    JwtError(jsonwebtoken::errors::Error),

    #[display("Redis error")]
    RedisError,

    #[display("Unauthorized")]
    UnAuthorizedError,
}

impl ResponseError for ServiceError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            ServiceError::NotFound => HttpResponse::NotFound().finish(),
            ServiceError::RedisError => HttpResponse::InternalServerError().finish(),
            ServiceError::UnAuthorizedError => HttpResponse::Unauthorized().finish(),
            _ => HttpResponse::InternalServerError().finish(),
        }
    }
}
