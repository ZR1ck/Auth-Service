use actix_web::{
    http::header,
    web::{self, Json},
    HttpMessage, HttpRequest, HttpResponse, Responder,
};
use log::info;
use serde_json::json;

use crate::{model::account::LoginInfo, AppAuthService};

pub async fn register(
    auth_service: web::Data<AppAuthService>,
    login_info: Json<LoginInfo>,
) -> impl Responder {
    match auth_service.add_account(login_info.0).await {
        Ok(result) => {
            info!("{} rows inserted", result);
            HttpResponse::Ok().body(format!("Success"))
        }
        Err(e) => HttpResponse::from_error(e),
    }
}

pub async fn login(
    auth_service: web::Data<AppAuthService>,
    login_info: Json<LoginInfo>,
) -> impl Responder {
    match auth_service.verify_account(login_info.0).await {
        Ok(result) => HttpResponse::Ok()
            .insert_header((
                header::SET_COOKIE,
                format!(
                    "refresh_token={};Path=/; HttpOnly; Secure; SameSite=Strict",
                    result.refresh_token
                ),
            ))
            .json(result),
        Err(e) => HttpResponse::from_error(e),
    }
}

pub async fn refresh(req: HttpRequest) -> impl Responder {
    let access_token = match req.extensions().get::<String>() {
        Some(token) => token.clone(),
        None => return HttpResponse::Unauthorized().body("Some thing wrong"),
    };
    HttpResponse::Ok().json(json!({"access_token": access_token}))
}
