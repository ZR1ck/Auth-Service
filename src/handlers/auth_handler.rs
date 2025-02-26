use std::result;

use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};
use log::{error, info};
use sqlx::PgPool;

use crate::{
    model::account::LoginInfo,
    service::auth_service::{self},
};

pub async fn register(pool: web::Data<PgPool>, login_info: Json<LoginInfo>) -> impl Responder {
    match auth_service::add_account(&pool, login_info.0).await {
        Ok(result) => {
            info!("{} rows inserted", result);
            HttpResponse::Ok().body(format!("Success"))
        }
        Err(e) => HttpResponse::from_error(e),
    }
}

pub async fn login(
    postgres_pool: web::Data<PgPool>,
    redis_pool: web::Data<deadpool_redis::Pool>,
    login_info: Json<LoginInfo>,
) -> impl Responder {
    match auth_service::verify_account(&postgres_pool, &redis_pool, login_info.0).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::from_error(e),
    }
}
