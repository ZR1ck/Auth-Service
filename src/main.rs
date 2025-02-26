use actix_web::{
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use config::{db, redis};
use dotenvy::dotenv;
use env_logger::Env;
use log::info;
use repository::redis_repo;
use sqlx::migrate;

mod config;
mod error;
mod handlers;
mod model;
mod repository;
mod service;
mod utils;

pub async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome!")
}

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

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(posgres_pool.clone()))
            .app_data(web::Data::new(redis_pool.clone()))
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
                            .route("/me", web::get().to(index))
                            .route("/refresh", web::post().to(index))
                            .route("/logout", web::post().to(index))
                            .route("/check-token", web::post().to(index)),
                    )
                    .service(
                        web::scope("/admin/users")
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
