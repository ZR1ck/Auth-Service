use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use dotenvy::dotenv;
use env_logger::Env;
use log::info;

async fn index() -> impl Responder {
    HttpResponse::Ok().body("Welcome!")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init_from_env(Env::default().default_filter_or("info"));

    info!("Starting server...");

    HttpServer::new(|| App::new().route("/", web::get().to(index)))
        .workers(1)
        .bind(("localhost", 8080))?
        .run()
        .await
}
