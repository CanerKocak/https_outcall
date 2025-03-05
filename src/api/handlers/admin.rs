use actix_web::{web, HttpRequest, HttpResponse};
use log::{info, error};

use crate::db::DbPool;
use crate::api::auth::authenticate_admin;
use crate::api::handlers::{canister, ApiResponse};
use crate::db::models::verified_module_hash::VerifiedModuleHash;

/// Add a verified module hash (admin only)
pub async fn add_verified_module_hash(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    request: web::Json<canister::VerifiedModuleHashRequest>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Extract request data
            let hash = &request.0.hash;
            let description = &request.0.description;
            let canister_type = &request.0.canister_type;
            
            // Validate the hash format
            if !hash.chars().all(|c| c.is_ascii_hexdigit()) || hash.len() != 64 {
                return HttpResponse::BadRequest()
                    .json(ApiResponse::<()>::error("Invalid module hash format"));
            }
            
            // Create the verified module hash
            let module_hash = VerifiedModuleHash::new(
                hash.clone(),
                description.clone(),
                canister_type.clone(),
            );
            
            // Save the verified module hash
            match module_hash.save(&conn) {
                Ok(_) => {
                    info!("Added verified module hash: {}", hash);
                    HttpResponse::Created()
                        .json(ApiResponse::success(module_hash, "Verified module hash added successfully"))
                }
                Err(e) => {
                    error!("Failed to save verified module hash: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to save verified module hash: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
}

/// Remove a verified module hash (admin only)
pub async fn remove_verified_module_hash(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            let hash = path.into_inner();
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Delete the verified module hash
            match VerifiedModuleHash::delete(&conn, &hash) {
                Ok(deleted) => {
                    if deleted {
                        info!("Removed verified module hash: {}", hash);
                        HttpResponse::Ok()
                            .json(ApiResponse::<()>::success((), "Verified module hash removed successfully"))
                    } else {
                        HttpResponse::NotFound()
                            .json(ApiResponse::<()>::error("Verified module hash not found"))
                    }
                }
                Err(e) => {
                    error!("Failed to delete verified module hash: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to delete verified module hash: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
} 