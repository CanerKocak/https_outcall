use actix_web::{web, HttpRequest, HttpResponse};
use log::{info, error};

use crate::db::DbPool;
use crate::api::auth::authenticate_admin;
use crate::api::handlers::{canister, ApiResponse};
use crate::db::models::verified_module_hash::VerifiedModuleHash;
use crate::db::models::canister::Canister;
use crate::db::models::token_info::TokenInfo;
use crate::db::models::miner_info::MinerInfo;

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

/// Delete a canister (admin only)
pub async fn delete_canister(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            let canister_id = path.into_inner();
            info!("Admin deleting canister: {}", canister_id);
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Delete the canister
            match Canister::delete(&conn, &canister_id) {
                Ok(deleted) => {
                    if deleted {
                        info!("Canister deleted: {}", canister_id);
                        HttpResponse::Ok()
                            .json(ApiResponse::<()>::success((), &format!("Canister {} deleted successfully", canister_id)))
                    } else {
                        HttpResponse::NotFound()
                            .json(ApiResponse::<()>::error(&format!("Canister {} not found", canister_id)))
                    }
                }
                Err(e) => {
                    error!("Failed to delete canister: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to delete canister: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
}

/// Delete a token (admin only)
pub async fn delete_token(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            let canister_id = path.into_inner();
            info!("Admin deleting token: {}", canister_id);
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Delete the token
            match TokenInfo::delete(&conn, &canister_id) {
                Ok(deleted) => {
                    if deleted {
                        info!("Token deleted: {}", canister_id);
                        HttpResponse::Ok()
                            .json(ApiResponse::<()>::success((), &format!("Token {} deleted successfully", canister_id)))
                    } else {
                        HttpResponse::NotFound()
                            .json(ApiResponse::<()>::error(&format!("Token {} not found", canister_id)))
                    }
                }
                Err(e) => {
                    error!("Failed to delete token: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to delete token: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
}

/// Delete a miner (admin only)
pub async fn delete_miner(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            let canister_id = path.into_inner();
            info!("Admin deleting miner: {}", canister_id);
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Delete the miner
            match MinerInfo::delete(&conn, &canister_id) {
                Ok(deleted) => {
                    if deleted {
                        info!("Miner deleted: {}", canister_id);
                        HttpResponse::Ok()
                            .json(ApiResponse::<()>::success((), &format!("Miner {} deleted successfully", canister_id)))
                    } else {
                        HttpResponse::NotFound()
                            .json(ApiResponse::<()>::error(&format!("Miner {} not found", canister_id)))
                    }
                }
                Err(e) => {
                    error!("Failed to delete miner: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to delete miner: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
}

/// Get all module hashes (admin only)
pub async fn get_all_module_hashes(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
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
            
            // Get all verified module hashes
            match VerifiedModuleHash::find_all(&conn) {
                Ok(hashes) => {
                    HttpResponse::Ok()
                        .json(ApiResponse::success(hashes, "Retrieved all verified module hashes"))
                }
                Err(e) => {
                    error!("Failed to get verified module hashes: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to get verified module hashes: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
}

/// Set a module hash for a canister (admin only)
pub async fn set_module_hash(
    req: HttpRequest,
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
    request: web::Json<canister::ModuleHashRequest>,
) -> HttpResponse {
    // Authenticate the admin
    match authenticate_admin(&req, &db_pool).await {
        Ok(admin) => {
            info!("Admin authenticated: {}", admin.username);
            
            // Get the canister ID from the path
            let canister_id = path.into_inner();
            
            // Get database connection
            let conn = match db_pool.get() {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get database connection: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error("Database error"));
                }
            };
            
            // Get the canister
            let canister = match Canister::find_by_id(&conn, &canister_id) {
                Ok(Some(canister)) => canister,
                Ok(_) => {
                    return HttpResponse::NotFound()
                        .json(ApiResponse::<()>::error(&format!("Canister {} not found", canister_id)));
                }
                Err(e) => {
                    error!("Failed to get canister: {}", e);
                    return HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to get canister: {}", e)));
                }
            };
            
            // Update the module hash
            let mut updated_canister = canister.clone();
            updated_canister.module_hash = Some(request.0.hash.clone());
            
            // Save the updated canister
            match updated_canister.save(&conn) {
                Ok(_) => {
                    info!("Updated module hash for canister {}: {}", canister_id, request.0.hash);
                    HttpResponse::Ok()
                        .json(ApiResponse::success(updated_canister, &format!("Module hash updated for canister {}", canister_id)))
                }
                Err(e) => {
                    error!("Failed to update module hash: {}", e);
                    HttpResponse::InternalServerError()
                        .json(ApiResponse::<()>::error(&format!("Failed to update module hash: {}", e)))
                }
            }
        }
        Err(response) => response,
    }
} 