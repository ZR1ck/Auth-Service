use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};
use sqlx::PgPool;

use crate::{db, model::NewAccount};

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome!")
}

pub async fn register(pool: web::Data<PgPool>, new_account: Json<NewAccount>) -> impl Responder {
    match db::add_account(&pool, new_account.0).await {
        Ok(result) => HttpResponse::Ok().body(format!("Success: rows: {}", result)),
        Err(e) => HttpResponse::from_error(e),
    }
}
