use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse,
};
use futures::future::{ok, Ready};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use log::{info, error};

use crate::db::DbPool;
use crate::db::models::admin::Admin;
use crate::api::handlers::ApiResponse;

// Admin authentication middleware
pub struct AdminAuth;

impl AdminAuth {
    pub fn new() -> Self {
        AdminAuth
    }
}

// Middleware factory for AdminAuth
impl<S, B> Transform<S, ServiceRequest> for AdminAuth
where
    // We need Clone because the service is cloned inside the call method
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AdminAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(AdminAuthMiddleware { service })
    }
}

pub struct AdminAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AdminAuthMiddleware<S>
where
    // We need Clone because we need to clone the service inside the async block
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static + Clone,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Check for API key in headers
        let api_key = if let Some(key) = req.headers().get("X-API-KEY") {
            if let Ok(key_str) = key.to_str() {
                key_str.to_string()
            } else {
                // Create response with error message
                let _resp = HttpResponse::Unauthorized()
                    .json(ApiResponse::<()>::error("Invalid API key format"));
                return Box::pin(async {
                    Err(actix_web::error::ErrorUnauthorized("Invalid API key format"))
                });
            }
        } else {
            // Create response with error message
            let _resp = HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("API key required"));
            return Box::pin(async {
                Err(actix_web::error::ErrorUnauthorized("API key required"))
            });
        };

        // Get the database pool from app data
        let db_pool = match req.app_data::<actix_web::web::Data<DbPool>>() {
            Some(pool) => pool.clone(),
            None => {
                error!("DB pool not found in app data");
                let _resp = HttpResponse::InternalServerError()
                    .json(ApiResponse::<()>::error("Server configuration error"));
                return Box::pin(async {
                    Err(actix_web::error::ErrorInternalServerError("Server configuration error"))
                });
            }
        };

        // Verify the API key
        let service = self.service.clone();
        Box::pin(async move {
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    let _resp = HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                    return Err(actix_web::error::ErrorInternalServerError("Database error"));
                }
            };

            // Check if the API key is valid
            match Admin::find_by_api_key(&conn, &api_key) {
                Ok(Some(admin)) => {
                    info!("Admin authenticated: {}", admin.username);
                    // Store admin in request extensions
                    req.extensions_mut().insert(admin);
                    // Continue with the request
                    let fut = service.call(req);
                    fut.await
                }
                Ok(None) => {
                    let _resp = HttpResponse::Unauthorized()
                        .json(ApiResponse::<()>::error("Invalid API key"));
                    Err(actix_web::error::ErrorUnauthorized("Invalid API key"))
                }
                Err(e) => {
                    error!("DB error while verifying API key: {}", e);
                    let _resp = HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                    Err(actix_web::error::ErrorInternalServerError(format!("Database error: {}", e)))
                }
            }
        })
    }
} 