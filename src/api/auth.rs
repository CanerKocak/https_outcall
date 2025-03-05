use actix_web::{web, HttpRequest, HttpResponse};
use log::{error, info};

use crate::db::DbPool;
use crate::db::models::admin::Admin;
use crate::api::handlers::ApiResponse;

/// Authenticate an admin user by API key
/// Returns Ok(admin) if authentication succeeds, or Err(HttpResponse) if it fails
pub async fn authenticate_admin(
    req: &HttpRequest,
    db_pool: &web::Data<DbPool>,
) -> Result<Admin, HttpResponse> {
    // Check for API key in headers
    let api_key = match req.headers().get("X-API-KEY") {
        Some(key) => match key.to_str() {
            Ok(key_str) => key_str,
            Err(_) => {
                return Err(HttpResponse::Unauthorized()
                    .json(ApiResponse::<()>::error("Invalid API key format")));
            }
        },
        None => {
            return Err(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("API key required")));
        }
    };

    // Get database connection
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return Err(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error")));
        }
    };

    // Check if the API key is valid
    match Admin::find_by_api_key(&conn, api_key) {
        Ok(Some(admin)) => {
            info!("Admin authenticated: {}", admin.username);
            Ok(admin)
        }
        Ok(None) => {
            Err(HttpResponse::Unauthorized()
                .json(ApiResponse::<()>::error("Invalid API key")))
        }
        Err(e) => {
            error!("DB error while verifying API key: {}", e);
            Err(HttpResponse::InternalServerError()
                .json(ApiResponse::<()>::error("Database error")))
        }
    }
} 