use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use log::{info, error};
use std::collections::HashMap;

use crate::db::pool::DbPool;
use crate::db::models::canister::{Canister, CanisterType};
use crate::api::handlers::ApiResponse;

#[derive(Deserialize)]
pub struct RegisterCanisterRequest {
    principal: String,
    canister_id: String,
    canister_type: String,
    module_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateCanisterRequest {
    principal: Option<String>,
    canister_type: Option<String>,
    module_hash: Option<String>,
}

#[derive(Deserialize)]
pub struct ModuleHashRequest {
    hash: String,
    description: String,
}

/// Get all canisters
pub async fn get_all_canisters(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all canisters");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<Canister>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match Canister::find_all(&conn) {
        Ok(canisters) => {
            HttpResponse::Ok().json(
                ApiResponse::success(canisters, "Canisters retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get canisters: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<Canister>>::error(&format!("Failed to get canisters: {}", e))
            )
        }
    }
}

/// Register a new canister
pub async fn register_canister(
    db_pool: web::Data<DbPool>,
    request: web::Json<RegisterCanisterRequest>,
) -> impl Responder {
    info!("API: Register canister: {}", request.canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    // Check if canister already exists
    match Canister::find_by_canister_id(&conn, &request.canister_id) {
        Ok(Some(_)) => {
            return HttpResponse::BadRequest().json(
                ApiResponse::<Canister>::error(&format!("Canister with ID {} already exists", request.canister_id))
            );
        },
        Ok(None) => {},
        Err(e) => {
            error!("Failed to check if canister exists: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Failed to check if canister exists: {}", e))
            );
        }
    }
    
    // Parse canister type
    let canister_type = match request.canister_type.to_lowercase().as_str() {
        "token" => CanisterType::Token,
        "miner" => CanisterType::Miner,
        "wallet" => CanisterType::Wallet,
        _ => {
            return HttpResponse::BadRequest().json(
                ApiResponse::<Canister>::error(&format!("Invalid canister type: {}", request.canister_type))
            );
        }
    };
    
    // Create new canister
    let canister = Canister::new(
        request.principal.clone(),
        request.canister_id.clone(),
        canister_type,
        request.module_hash.clone(),
    );
    
    // Save canister
    match canister.save(&conn) {
        Ok(_) => {
            HttpResponse::Created().json(
                ApiResponse::success(canister, "Canister registered successfully")
            )
        },
        Err(e) => {
            error!("Failed to save canister: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Failed to save canister: {}", e))
            )
        }
    }
}

/// Get a specific canister
pub async fn get_canister(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Get canister: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match Canister::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(canister)) => {
            HttpResponse::Ok().json(
                ApiResponse::success(canister, "Canister retrieved successfully")
            )
        },
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<Canister>::error(&format!("Canister with ID {} not found", canister_id))
            )
        },
        Err(e) => {
            error!("Failed to get canister: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Failed to get canister: {}", e))
            )
        }
    }
}

/// Update a canister
pub async fn update_canister(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
    request: web::Json<UpdateCanisterRequest>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Update canister: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    // Get existing canister
    let mut canister = match Canister::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(canister)) => canister,
        Ok(None) => {
            return HttpResponse::NotFound().json(
                ApiResponse::<Canister>::error(&format!("Canister with ID {} not found", canister_id))
            );
        },
        Err(e) => {
            error!("Failed to get canister: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Failed to get canister: {}", e))
            );
        }
    };
    
    // Update fields if provided
    if let Some(principal) = &request.principal {
        canister.principal = principal.clone();
    }
    
    if let Some(canister_type) = &request.canister_type {
        canister.canister_type = match canister_type.to_lowercase().as_str() {
            "token" => CanisterType::Token,
            "miner" => CanisterType::Miner,
            "wallet" => CanisterType::Wallet,
            _ => {
                return HttpResponse::BadRequest().json(
                    ApiResponse::<Canister>::error(&format!("Invalid canister type: {}", canister_type))
                );
            }
        };
    }
    
    canister.module_hash = request.module_hash.clone();
    
    // Save updated canister
    match canister.save(&conn) {
        Ok(_) => {
            HttpResponse::Ok().json(
                ApiResponse::success(canister, "Canister updated successfully")
            )
        },
        Err(e) => {
            error!("Failed to update canister: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Canister>::error(&format!("Failed to update canister: {}", e))
            )
        }
    }
}

/// Delete a canister
pub async fn delete_canister(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Delete canister: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"));
        }
    };
    
    // First check if the canister exists
    match Canister::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(_)) => {
            // Delete the canister from the database
            match Canister::delete(&conn, &canister_id) {
                Ok(true) => {
                    info!("Canister deleted: {}", canister_id);
                    HttpResponse::Ok().json(ApiResponse::success(true, "Canister deleted"))
                },
                Ok(false) => {
                    // This shouldn't happen as we already checked for existence
                    error!("Canister not found after existence check: {}", canister_id);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete canister"))
                },
                Err(e) => {
                    error!("Failed to delete canister: {}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete canister"))
                }
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<bool>::error("Canister not found"))
        },
        Err(e) => {
            error!("Failed to find canister: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"))
        }
    }
}

/// Get all module hashes
pub async fn get_all_module_hashes(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all module hashes");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<HashMap<String, Vec<Canister>>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match Canister::find_all(&conn) {
        Ok(canisters) => {
            // Group canisters by module hash
            let mut hash_map: HashMap<String, Vec<Canister>> = HashMap::new();
            
            for canister in canisters {
                if let Some(hash) = &canister.module_hash {
                    hash_map.entry(hash.clone())
                        .or_insert_with(Vec::new)
                        .push(canister);
                }
            }
            
            HttpResponse::Ok().json(
                ApiResponse::success(hash_map, "Module hashes retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get canisters: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<HashMap<String, Vec<Canister>>>::error(&format!("Failed to get canisters: {}", e))
            )
        }
    }
}

/// Set module hash for a canister
pub async fn set_module_hash(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
    req: web::Json<ModuleHashRequest>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Set module hash for canister: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"));
        }
    };
    
    match Canister::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(mut canister)) => {
            // Update the module hash
            canister.module_hash = Some(req.hash.clone());
            
            // Add description to the log
            info!("Setting module hash with description: {}", req.description);
            
            // Save the updated canister
            if let Err(e) = canister.save(&conn) {
                error!("Failed to save canister: {}", e);
                return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to save canister"));
            }
            
            HttpResponse::Ok().json(ApiResponse::success(true, "Module hash updated"))
        },
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<bool>::error("Canister not found"))
        },
        Err(e) => {
            error!("Failed to find canister: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"))
        }
    }
} 