use actix_web::{web, HttpMessage, HttpRequest, HttpResponse, Responder};

use crate::{utils::jwt::Claims, AppAccountService};

pub async fn me(account_service: web::Data<AppAccountService>, req: HttpRequest) -> impl Responder {
    let extensions = req.extensions();
    let access_token_data: &Claims = match extensions.get::<Claims>() {
        Some(token_data) => token_data,
        None => return HttpResponse::Unauthorized().body("Some thing wrong"),
    };

    let account = match account_service
        .get_account_info(&access_token_data.id)
        .await
    {
        Ok(account) => account,
        Err(e) => {
            return HttpResponse::from_error(e);
        }
    };

    HttpResponse::Ok().json(account)
}
