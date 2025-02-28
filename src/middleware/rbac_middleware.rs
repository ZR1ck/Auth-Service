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

pub struct RbacMiddleware;

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

    fn new_transform(&self, service: S) -> Self::Future {
        ok(RbacMiddlewareServie {
            service: Rc::new(service),
        })
    }
}

pub struct RbacMiddlewareServie<S> {
    service: Rc<S>,
}

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

    fn call(&self, req: ServiceRequest) -> Self::Future {
        info!("RbacMiddleware called");

        let srv = self.service.clone();
        let path = req.path().to_string();
        let user_info = req.extensions().get::<Claims>().cloned();

        Box::pin(async move {
            if let Some(user_info) = user_info {
                if !has_permission(user_info, &path) {
                    return Ok(req.into_response(HttpResponse::Forbidden().finish()));
                }
            } else {
                return Ok(req.into_response(HttpResponse::InternalServerError().finish()));
            }
            Ok(srv.call(req).await?.map_into_boxed_body())
        })
    }
}

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
