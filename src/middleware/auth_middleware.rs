use std::{future::Future, pin::Pin, rc::Rc, sync::Arc, task::Poll};

use actix_web::{
    body::{BoxBody, MessageBody},
    cookie::Cookie,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ok, Ready};
use log::info;

use crate::{service::token_service::TokenService, traits::redis_traits::TokenRedisRepository};

#[derive(Clone)]
pub struct AuthMiddleware<T: TokenRedisRepository> {
    token_service: Arc<TokenService<T>>,
}

impl<T: TokenRedisRepository> AuthMiddleware<T> {
    pub fn new(token_redis_repo: Arc<T>) -> Self {
        Self {
            token_service: Arc::new(TokenService::new(token_redis_repo)),
        }
    }
}

impl<S, B, T> Transform<S, ServiceRequest> for AuthMiddleware<T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    T: TokenRedisRepository + 'static,
{
    type Error = Error;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    type InitError = ();
    type Response = ServiceResponse<BoxBody>;
    type Transform = AuthMiddlewareService<S, T>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            token_service: self.token_service.clone(),
        })
    }
}

pub struct AuthMiddlewareService<S, T: TokenRedisRepository> {
    service: Rc<S>,
    token_service: Arc<TokenService<T>>,
}

impl<S, B, T> Service<ServiceRequest> for AuthMiddlewareService<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    T: TokenRedisRepository + 'static,
{
    type Error = Error;
    type Response = ServiceResponse<BoxBody>;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        _ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let token_service = self.token_service.clone();

        Box::pin(async move {
            let access_token = req
                .headers()
                .get("Authorization")
                .and_then(|hv| hv.to_str().ok())
                .map(|s| s.trim_start_matches("Bearer "));

            if let Some(token) = access_token {
                match token_service.verify_access_token(token) {
                    Ok(claims) => {
                        info!("Access token verified");
                        req.extensions_mut().insert(claims.id.clone());

                        let res = srv.call(req).await?;
                        return Ok(res.map_into_boxed_body());
                    }
                    Err(_) => {
                        info!("Access token denied");
                        if let Some(refresh_token) = req
                            .headers()
                            .get("Refresh-token")
                            .and_then(|hv| hv.to_str().ok())
                        {
                            match token_service.verify_refresh_token(refresh_token).await {
                                Ok(new_access_token) => {
                                    info!("Refresh token verified");
                                    let mut res = srv.call(req).await?.map_into_boxed_body();

                                    let access_token_cockie =
                                        Cookie::build("access_token", new_access_token)
                                            .path("/")
                                            .http_only(true)
                                            .secure(true)
                                            .finish();
                                    let res_header = res.headers_mut();
                                    res_header.append(
                                        actix_web::http::header::SET_COOKIE,
                                        access_token_cockie.to_string().parse().unwrap(),
                                    );

                                    return Ok(res);
                                }
                                Err(_) => {
                                    info!("Refresh token denied");
                                    return Ok(
                                        req.into_response(HttpResponse::Unauthorized().finish())
                                    );
                                }
                            }
                        }
                    }
                }
            }

            Ok(req.into_response(HttpResponse::Unauthorized().finish()))
        })
    }
}
