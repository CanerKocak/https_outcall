use actix_web::{web, HttpResponse, Responder};
use serde::Deserialize;
use log::{info, error};

use crate::db::pool::DbPool;
use crate::db::models::canister::{Canister, CanisterType};
use crate::api::handlers::ApiResponse;
use crate::db::models::verified_module_hash::VerifiedModuleHash;
use crate::ic::agent::create_agent;
use crate::ic::services::module_hash::get_module_hash;

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
    pub hash: String,
    pub description: String,
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
    
    // If module_hash is provided, use it
    // Otherwise, we'll fetch it in the background tasks
    let module_hash = request.module_hash.clone();
    
    // Create new canister
    let canister = Canister::new(
        request.principal.clone(),
        request.canister_id.clone(),
        canister_type,
        module_hash,
    );
    
    // Save canister
    match canister.save(&conn) {
        Ok(_) => {
            // If module_hash not provided, start a background task to fetch it
            if request.module_hash.is_none() {
                let canister_id = request.canister_id.clone();
                let canister_type_str = request.canister_type.clone();
                let db_pool = db_pool.clone();
                
                // Spawn a task to fetch and verify the module hash
                tokio::spawn(async move {
                    if let Err(e) = update_and_verify_module_hash(db_pool, &canister_id, &canister_type_str).await {
                        error!("Failed to update module hash for canister {}: {}", canister_id, e);
                    }
                });
            } else {
                // Verify the provided module hash
                let module_hash = request.module_hash.as_ref().unwrap();
                let canister_type_str = request.canister_type.clone();
                
                match VerifiedModuleHash::is_hash_verified(&conn, module_hash, &canister_type_str) {
                    Ok(true) => {
                        info!("Verified module hash for canister {}", request.canister_id);
                    },
                    Ok(false) => {
                        info!("Unverified module hash for canister {}", request.canister_id);
                    },
                    Err(e) => {
                        error!("Failed to check if module hash is verified: {}", e);
                    }
                }
            }
            
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

/// Get all module hashes - forwarding to verified module hashes
pub async fn get_all_module_hashes(db_pool: web::Data<DbPool>) -> impl Responder {
    get_all_verified_module_hashes(db_pool).await
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

/// Get all verified module hashes
pub async fn get_all_verified_module_hashes(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all verified module hashes");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<VerifiedModuleHash>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match VerifiedModuleHash::find_all(&conn) {
        Ok(hashes) => {
            HttpResponse::Ok().json(
                ApiResponse::success(hashes, "Verified module hashes retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get verified module hashes: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<VerifiedModuleHash>>::error(&format!("Failed to get verified module hashes: {}", e))
            )
        }
    }
}

#[derive(Deserialize)]
pub struct VerifiedModuleHashRequest {
    pub hash: String,
    pub description: String,
    pub canister_type: String,
}

/// Add a new verified module hash (admin only)
pub async fn add_verified_module_hash(
    db_pool: web::Data<DbPool>,
    request: web::Json<VerifiedModuleHashRequest>,
) -> impl Responder {
    info!("API: Add verified module hash: {}", request.hash);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<VerifiedModuleHash>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    // Validate hash format (hex string with 64 characters)
    if !is_valid_hex_hash(&request.hash) {
        return HttpResponse::BadRequest().json(
            ApiResponse::<VerifiedModuleHash>::error("Invalid hash format. Must be a 64-character hex string.")
        );
    }
    
    // Create new verified module hash
    let verified_hash = VerifiedModuleHash::new(
        request.hash.clone(),
        request.description.clone(),
        request.canister_type.clone(),
    );
    
    // Save verified module hash
    match verified_hash.save(&conn) {
        Ok(_) => {
            HttpResponse::Created().json(
                ApiResponse::success(verified_hash, "Verified module hash added successfully")
            )
        },
        Err(e) => {
            error!("Failed to save verified module hash: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<VerifiedModuleHash>::error(&format!("Failed to save verified module hash: {}", e))
            )
        }
    }
}

/// Remove a verified module hash (admin only)
pub async fn remove_verified_module_hash(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let hash = path.into_inner();
    info!("API: Remove verified module hash: {}", hash);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"));
        }
    };
    
    // First check if the verified module hash exists
    match VerifiedModuleHash::find_by_hash(&conn, &hash) {
        Ok(Some(_)) => {
            // Delete the verified module hash from the database
            match VerifiedModuleHash::delete(&conn, &hash) {
                Ok(true) => {
                    info!("Verified module hash deleted: {}", hash);
                    HttpResponse::Ok().json(ApiResponse::success(true, "Verified module hash deleted"))
                },
                Ok(false) => {
                    // This shouldn't happen as we already checked for existence
                    error!("Verified module hash not found after existence check: {}", hash);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete verified module hash"))
                },
                Err(e) => {
                    error!("Failed to delete verified module hash: {}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete verified module hash"))
                }
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<bool>::error("Verified module hash not found"))
        },
        Err(e) => {
            error!("Failed to find verified module hash: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"))
        }
    }
}

/// Helper function to validate a hex hash
fn is_valid_hex_hash(hash: &str) -> bool {
    if hash.len() != 64 {
        return false;
    }
    
    hash.chars().all(|c| c.is_ascii_hexdigit())
}

/// Helper function to update and verify a canister's module hash
async fn update_and_verify_module_hash(
    db_pool: web::Data<DbPool>,
    canister_id: &str,
    canister_type: &str,
) -> Result<(), anyhow::Error> {
    info!("Updating and verifying module hash for canister: {}", canister_id);
    
    // Create IC agent
    let agent = create_agent("https://ic0.app").await?;
    
    // Get module hash from canister
    let module_hash = get_module_hash(&agent, canister_id).await?;
    
    // Get DB connection
    let conn = db_pool.get()?;
    
    // Check if the hash is verified
    let is_verified = VerifiedModuleHash::is_hash_verified(&conn, &module_hash, canister_type)?;
    
    // Update the canister's module hash
    match Canister::find_by_canister_id(&conn, canister_id)? {
        Some(mut canister) => {
            // Update the module hash
            canister.module_hash = Some(module_hash.clone());
            
            // Save the updated canister
            canister.save(&conn)?;
            
            if is_verified {
                info!("Verified module hash for canister {}: {}", canister_id, module_hash);
            } else {
                info!("Unverified module hash for canister {}: {}", canister_id, module_hash);
            }
            
            Ok(())
        },
        None => {
            Err(anyhow::anyhow!("Canister not found"))
        }
    }
} 