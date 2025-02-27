use std::{future::Future, pin::Pin, rc::Rc, sync::Arc, task::Poll};

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures_util::future::{ok, Ready};
use log::{error, info};

use crate::{service::token_service::TokenService, traits::redis_traits::TokenRedisRepository};

/// `AuthMiddleware` is a struct representing authentication middleware.
/// This middleware is responsible for verifying access and refresh tokens
/// from incoming requests and injecting verified token claims into the request
/// extensions for downstream handlers to use.
#[derive(Clone)]
pub struct AuthMiddleware<T: TokenRedisRepository> {
    // Shared instance of the `TokenService`, responsible for token verification.
    token_service: Arc<TokenService<T>>,
}

impl<T: TokenRedisRepository> AuthMiddleware<T> {
    /// Creates a new `AuthMiddleware` with the given token repository.
    ///
    /// # Arguments
    ///
    /// * `token_redis_repo` - An `Arc` wrapped repository for token storage and retrieval.
    pub fn new(token_redis_repo: Arc<T>) -> Self {
        Self {
            token_service: Arc::new(TokenService::new(token_redis_repo)),
        }
    }
}

/// Actix Web `Transform` implementation for `AuthMiddleware`.
///
/// This allows the middleware to be used in the Actix Web middleware chain.
impl<S, B, T> Transform<S, ServiceRequest> for AuthMiddleware<T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    T: TokenRedisRepository + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S, T>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Initializes the middleware with the downstream service.
    fn new_transform(&self, service: S) -> Self::Future {
        ok(AuthMiddlewareService {
            service: Rc::new(service),
            token_service: self.token_service.clone(),
        })
    }
}

/// `AuthMiddlewareService` is the actual service handling request processing.
/// It wraps the underlying service and performs token validation before
/// delegating the request to the next service in the chain.
pub struct AuthMiddlewareService<S, T: TokenRedisRepository> {
    /// The next service in the chain.
    service: Rc<S>,
    /// Shared `TokenService` for token verification.
    token_service: Arc<TokenService<T>>,
}

/// Actix Web `Service` implementation for `AuthMiddlewareService`.
impl<S, B, T> Service<ServiceRequest> for AuthMiddlewareService<S, T>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
    T: TokenRedisRepository + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    /// Always ready to process requests.
    fn poll_ready(&self, _ctx: &mut core::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Handles the incoming request.
    ///
    /// If the request path is `/api/auth/refresh`, the middleware expects a `refresh_token` cookie.
    /// It will verify the refresh token and insert the new access token into request extensions.
    ///
    /// For other requests, the middleware expects a Bearer token in the `Authorization` header.
    /// It will verify the access token and insert token claims into request extensions.
    ///
    /// If any token is invalid or missing, the middleware responds with `401 Unauthorized`.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let token_service = self.token_service.clone();

        Box::pin(async move {
            let path = req.path();

            // Special handling for refresh token endpoint
            if path.contains("/api/auth/refresh") {
                if let Some(cookie) = req.cookie("refresh_token") {
                    let refresh_token = cookie.value().to_string();
                    match token_service.verify_refresh_token(&refresh_token).await {
                        Ok(new_access_token) => {
                            info!("Refresh token verified");

                            // Insert new access token into request extensions
                            req.extensions_mut().insert(new_access_token);

                            // Forward request to next service
                            let res = srv.call(req).await?;
                            return Ok(res.map_into_boxed_body());
                        }
                        Err(_) => {
                            // Refresh token verification failed, return 401
                        }
                    }
                }
            } else {
                // For all other path, check for access token in Authorization header
                let access_token = req
                    .headers()
                    .get("Authorization")
                    .and_then(|hv| hv.to_str().ok())
                    .map(|s| s.trim_start_matches("Bearer "));

                if let Some(token) = access_token {
                    return match token_service.verify_access_token(token) {
                        Ok(claims) => {
                            info!("Access token verified");

                            // Insert claims into request extensions
                            req.extensions_mut().insert(claims);

                            // Forward request to next service
                            let res = srv.call(req).await?;
                            Ok(res.map_into_boxed_body())
                        }
                        Err(e) => {
                            // Access token verification failed, return 401
                            error!("Access token error: {}", e);
                            Ok(req.into_response(HttpResponse::Unauthorized().finish()))
                        }
                    };
                };
            }
            // No valid token found, return 401
            Ok(req.into_response(HttpResponse::Unauthorized().finish()))
        })
    }
}
