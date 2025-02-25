use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};
use sqlx::PgPool;

use crate::{model::account::LoginInfo, service::auth_service};

pub async fn register(pool: web::Data<PgPool>, login_info: Json<LoginInfo>) -> impl Responder {
    match auth_service::add_account(&pool, login_info.0).await {
        Ok(result) => HttpResponse::Ok().body(format!("Success: rows: {}", result)),
        Err(e) => HttpResponse::from_error(e),
    }
}
