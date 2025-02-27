use actix_web::{
    web::{self, Json},
    HttpResponse, Responder,
};
use log::{error, info};

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
        Err(e) => {
            error!("{}", e);
            HttpResponse::from_error(e)
        }
    }
}

pub async fn login(
    auth_service: web::Data<AppAuthService>,
    login_info: Json<LoginInfo>,
) -> impl Responder {
    match auth_service.verify_account(login_info.0).await {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => HttpResponse::from_error(e),
    }
}
