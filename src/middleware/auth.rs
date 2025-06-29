use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, web
};
use futures_util::future::LocalBoxFuture;
use std::{
    future::{ready, Ready},
    rc::Rc,
};
use crate::{utils::jwt::decode_jwt, config::Config, cache::CacheManager};

pub struct AuthMiddleware;

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            // Skip authentication for certain routes
            let path = req.path();
            if should_skip_auth(path) {
                return service.call(req).await;
            }

            // Extract JWT token from Authorization header
            let auth_header = req.headers().get("Authorization");
            let token = match auth_header {
                Some(header_value) => {
                    match header_value.to_str() {
                        Ok(header_str) => {
                            if header_str.starts_with("Bearer ") {
                                header_str.trim_start_matches("Bearer ").to_string()
                            } else {
                                return Err(actix_web::error::ErrorUnauthorized("Invalid authorization header format"));
                            }
                        }
                        Err(_) => {
                            return Err(actix_web::error::ErrorUnauthorized("Invalid authorization header"));
                        }
                    }
                }
                None => {
                    return Err(actix_web::error::ErrorUnauthorized("Authorization header missing"));
                }
            };

            // Check if token is blacklisted
            let cache = req.app_data::<web::Data<CacheManager>>().unwrap();
            if cache.is_token_blacklisted(&token).await {
                return Err(actix_web::error::ErrorUnauthorized("Token has been invalidated"));
            }

            // Get config from app data
            let config = req.app_data::<web::Data<Config>>().unwrap();

            // Decode and validate JWT token
            match decode_jwt(&token, &config.jwt_secret) {
                Ok(claims) => {
                    // Add claims to request extensions for use in handlers
                    req.extensions_mut().insert(claims);
                    service.call(req).await
                }
                Err(_) => {
                    Err(actix_web::error::ErrorUnauthorized("Invalid or expired token"))
                }
            }
        })
    }
}

fn should_skip_auth(path: &str) -> bool {
    let public_routes = [
        "/",
        "/api/v1/health",
        "/api/v1/auth/login",
        "/api/v1/auth/register",
        "/api/v1/auth/forgot-password",
        "/api/v1/auth/reset-password",
    ];
    
    public_routes.contains(&path)
} 