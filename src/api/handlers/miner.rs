use actix_web::{web, HttpResponse, Responder};
use log::{info, error};
use serde::Serialize;

use crate::db::DbPool;
use crate::db::models::miner_info::MinerInfo;
use crate::db::models::mining_stats::MiningStats;
use crate::api::handlers::ApiResponse;

#[derive(Serialize)]
pub struct MinerWithStats {
    pub miner_info: MinerInfo,
    pub mining_stats: Option<MiningStats>,
}

/// Get all miners
pub async fn get_all_miners(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all miners");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MinerInfo>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match MinerInfo::find_all(&conn) {
        Ok(miners) => {
            HttpResponse::Ok().json(
                ApiResponse::success(miners, "Miners retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get miners: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MinerInfo>>::error(&format!("Failed to get miners: {}", e))
            )
        }
    }
}

/// Get a specific miner
pub async fn get_miner(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Get miner: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<MinerInfo>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match MinerInfo::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(miner)) => {
            HttpResponse::Ok().json(
                ApiResponse::success(miner, "Miner retrieved successfully")
            )
        },
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<MinerInfo>::error(&format!("Miner with canister ID {} not found", canister_id))
            )
        },
        Err(e) => {
            error!("Failed to get miner: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<MinerInfo>::error(&format!("Failed to get miner: {}", e))
            )
        }
    }
}

/// Get mining stats for a specific miner
pub async fn get_miner_stats(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Get mining stats for miner: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<MiningStats>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match MiningStats::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(stats)) => {
            HttpResponse::Ok().json(
                ApiResponse::success(stats, "Mining stats retrieved successfully")
            )
        },
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<MiningStats>::error(&format!("Mining stats for canister ID {} not found", canister_id))
            )
        },
        Err(e) => {
            error!("Failed to get mining stats: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<MiningStats>::error(&format!("Failed to get mining stats: {}", e))
            )
        }
    }
}

/// Get miners by token
pub async fn get_miners_by_token(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let token_canister_id = path.into_inner();
    info!("API: Get miners by token: {}", token_canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MinerInfo>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match MinerInfo::find_by_token(&conn, &token_canister_id) {
        Ok(miners) => {
            HttpResponse::Ok().json(
                ApiResponse::success(miners, "Miners retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get miners by token: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MinerInfo>>::error(&format!("Failed to get miners by token: {}", e))
            )
        }
    }
}

/// Delete a miner
pub async fn delete_miner(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Delete miner: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"));
        }
    };
    
    // First check if the miner exists
    match MinerInfo::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(_)) => {
            // Delete the miner from the database
            match MinerInfo::delete(&conn, &canister_id) {
                Ok(true) => {
                    info!("Miner deleted: {}", canister_id);
                    
                    // Also delete any associated mining stats
                    if let Err(e) = MiningStats::delete(&conn, &canister_id) {
                        error!("Failed to delete associated mining stats: {}", e);
                        // Continue anyway, as this is not critical
                    }
                    
                    HttpResponse::Ok().json(ApiResponse::success(true, "Miner deleted"))
                },
                Ok(false) => {
                    // This shouldn't happen as we already checked for existence
                    error!("Miner not found after existence check: {}", canister_id);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete miner"))
                },
                Err(e) => {
                    error!("Failed to delete miner: {}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete miner"))
                }
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<bool>::error(&format!("Miner with ID {} not found", canister_id)))
        },
        Err(e) => {
            error!("Failed to check miner existence: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete miner"))
        }
    }
}

/// Get all mining stats
pub async fn get_all_mining_stats(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all mining stats");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MiningStats>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match MiningStats::find_all(&conn) {
        Ok(stats) => {
            HttpResponse::Ok().json(
                ApiResponse::success(stats, "Mining stats retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get mining stats: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<MiningStats>>::error(&format!("Failed to get mining stats: {}", e))
            )
        }
    }
} 