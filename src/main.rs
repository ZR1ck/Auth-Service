use std::sync::Arc;

use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use config::{db, redis};
use dotenvy::dotenv;
use env_logger::Env;
use log::info;
use middleware::auth_middleware::{self};
use repository::{account_repo::AccountRepo, token_redis_repo::TokenRedisRepo};
use service::auth_service::AuthService;
use sqlx::migrate;

mod config;
mod error;
mod handlers;
mod middleware;
mod model;
mod repository;
mod service;
mod traits;
mod utils;

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome!")
}

type AppAuthService = AuthService<AccountRepo, TokenRedisRepo>;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    info!("Creating redis pool");
    let redis_pool = redis::create_redis_pool();

    info!("Creating database pool");
    let posgres_pool = db::create_postgres_pool().await;

    let migrator = migrate::Migrator::new(std::path::Path::new("./migrations"))
        .await
        .expect("Failed to init migrator");

    migrator.run(&posgres_pool).await.expect("Migration failed");

    info!("Starting server...");

    let account_repo = Arc::new(repository::account_repo::AccountRepo::new(posgres_pool));
    let token_redis_repo = Arc::new(repository::token_redis_repo::TokenRedisRepo::new(
        redis_pool,
    ));

    let auth_service = Arc::new(service::auth_service::AuthService::new(
        account_repo.clone(),
        token_redis_repo.clone(),
    ));

    let auth_middleware = Arc::new(auth_middleware::AuthMiddleware::new(
        token_redis_repo.clone(),
    ));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::from(auth_service.clone()))
            .route("/", web::get().to(index))
            .service(
                web::scope("/api")
                    .service(
                        web::scope("/auth")
                            .route(
                                "/register",
                                web::post().to(handlers::auth_handler::register),
                            )
                            .route("/login", web::post().to(handlers::auth_handler::login))
                            .route("/refresh", web::post().to(index))
                            .route("/check-token", web::post().to(index))
                            .route("/me", web::get().to(index))
                            .route("/logout", web::post().to(index)),
                    )
                    .service(
                        web::scope("/admin/users")
                            .wrap(auth_middleware.clone())
                            .route("/", web::get().to(index))
                            .route("/{user_id}/role", web::put().to(index))
                            .route("/{users_id}/", web::delete().to(index)),
                    ),
            )
    })
    .workers(1)
    .bind(("localhost", 8080))?
    .run()
    .await
}
