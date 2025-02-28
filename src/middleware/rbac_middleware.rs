use std::{collections::HashMap, future::Future, pin::Pin, rc::Rc, task::Poll};

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    error::Error,
    HttpMessage, HttpResponse,
};
use futures_util::future::{ok, Ready};
use log::info;

use crate::utils::jwt::Claims;

/// `RbacMiddleware` is a struct representing a Role-Based Access Control middleware.
/// It checks the user's role (from JWT claims) against the required roles for specific API paths.
/// Unauthorized requests will receive a `403 Forbidden` response
pub struct RbacMiddleware;

/// Actix Web `Transform` implementation for `RbacMiddleware`.
///
/// This allows the middleware to be used in the Actix Web middleware chain.
impl<S, B> Transform<S, ServiceRequest> for RbacMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RbacMiddlewareServie<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Initializes the middleware with the downstream service.
    fn new_transform(&self, service: S) -> Self::Future {
        ok(RbacMiddlewareServie {
            service: Rc::new(service),
        })
    }
}

/// `RbacMiddlewareService` is the actual service that performs role-based checks.
pub struct RbacMiddlewareServie<S> {
    service: Rc<S>,
}

/// Actix Web `Service` implementation for `AuthMiddlewareService`.
impl<S, B> Service<ServiceRequest> for RbacMiddlewareServie<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        _ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    /// Processes incoming requests, performs RBAC checks, and either forwards the request or rejects it.
    fn call(&self, req: ServiceRequest) -> Self::Future {
        info!("RbacMiddleware called");

        let srv = self.service.clone();
        let path = req.path().to_string();

        // Retrieve user claims from request extensions
        let user_info = req.extensions().get::<Claims>().cloned();

        Box::pin(async move {
            if let Some(user_info) = user_info {
                // Check if the user has permission to access the requested path.
                if !has_permission(user_info, &path) {
                    return Ok(req.into_response(HttpResponse::Forbidden().finish()));
                }
            } else {
                // If claims are missing, return 500 Internal Server Error.
                return Ok(req.into_response(HttpResponse::InternalServerError().finish()));
            }

            // Forward the request to the inner service if authorization succeeds.
            Ok(srv.call(req).await?.map_into_boxed_body())
        })
    }
}

/// Checks whether the given user has permission to access a specific path.
/// Permissions are defined using path prefixes and allowed roles.
fn has_permission(user_info: Claims, path: &str) -> bool {
    let mut permissions = HashMap::new();
    permissions.insert("/api/admin", vec!["admin"]);
    permissions.insert("/api/user", vec!["user", "admin"]);

    for (prefix, roles) in &permissions {
        if path.starts_with(prefix) {
            return roles.contains(&user_info.role.as_str());
        }
    }

    false
}
