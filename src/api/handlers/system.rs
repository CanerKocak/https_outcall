use actix_web::{web, HttpResponse, Responder};
use log::{info, error};
use serde::Serialize;
use serde_json::json;
use std::collections::HashMap;

use crate::db::DbPool;
use crate::api::handlers::ApiResponse;
use crate::ic::utils::interface_util::generate_interface_files;
use crate::websocket;

#[derive(Serialize)]
pub struct SystemStatus {
    pub version: String,
    pub uptime: u64,
    pub database_connected: bool,
    pub canisters_count: usize,
    pub tokens_count: usize,
    pub miners_count: usize,
}

/// Get system status
pub async fn get_system_status(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get system status");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<SystemStatus>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    // Get counts from database
    let canisters_count = match conn.query_row(
        "SELECT COUNT(*) FROM canisters",
        [],
        |row| row.get::<_, usize>(0),
    ) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get canisters count: {}", e);
            0
        }
    };
    
    let tokens_count = match conn.query_row(
        "SELECT COUNT(*) FROM token_info",
        [],
        |row| row.get::<_, usize>(0),
    ) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get tokens count: {}", e);
            0
        }
    };
    
    let miners_count = match conn.query_row(
        "SELECT COUNT(*) FROM miner_info",
        [],
        |row| row.get::<_, usize>(0),
    ) {
        Ok(count) => count,
        Err(e) => {
            error!("Failed to get miners count: {}", e);
            0
        }
    };
    
    // Create status response
    let status = SystemStatus {
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: 0, // TODO: Track uptime
        database_connected: true,
        canisters_count,
        tokens_count,
        miners_count,
    };
    
    HttpResponse::Ok().json(
        ApiResponse::success(status, "System status retrieved successfully")
    )
}

/// Trigger a manual refresh notification via WebSockets
pub async fn trigger_refresh(_db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Trigger refresh notification");
    
    // Instead of running background tasks, we'll just broadcast a notification
    // that clients can use to refresh their data
    websocket::broadcast_notification(
        "refresh_requested", 
        json!({
            "timestamp": chrono::Utc::now().timestamp_millis(),
            "message": "Manual refresh requested"
        })
    );
    
    HttpResponse::Ok().json(
        ApiResponse::success(true, "Refresh notification sent successfully")
    )
}

/// Generate interface files for clients
pub async fn generate_interfaces(
    query: web::Query<HashMap<String, String>>,
) -> impl Responder {
    info!("API: Generate interface files");
    
    // Get output directory from query params, default to "interfaces"
    let output_dir = query.get("output_dir").map_or("interfaces", |s| s.as_str());
    
    match generate_interface_files(output_dir) {
        Ok(_) => {
            HttpResponse::Ok().json(
                ApiResponse::success(
                    output_dir, 
                    &format!("Interface files generated successfully to {}", output_dir)
                )
            )
        },
        Err(e) => {
            error!("Failed to generate interface files: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<String>::error(&format!("Failed to generate interface files: {}", e))
            )
        }
    }
}

/// Get aggregate statistics (token and miner counts, totals)
pub async fn get_statistics(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get aggregate statistics");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<HashMap<String, i64>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    // Create stats object
    let mut stats = HashMap::new();
    
    // Get total token count
    match conn.query_row(
        "SELECT COUNT(*) FROM token_info",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(count) => {
            stats.insert("token_count".to_string(), count);
        },
        Err(e) => {
            error!("Failed to get token count: {}", e);
            stats.insert("token_count".to_string(), 0);
        }
    }
    
    // Get total miner count
    match conn.query_row(
        "SELECT COUNT(*) FROM miner_info",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(count) => {
            stats.insert("miner_count".to_string(), count);
        },
        Err(e) => {
            error!("Failed to get miner count: {}", e);
            stats.insert("miner_count".to_string(), 0);
        }
    }
    
    // Get total blocks mined
    match conn.query_row(
        "SELECT SUM(blocks_mined) FROM mining_stats",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(count) => {
            stats.insert("blocks_mined".to_string(), count);
        },
        Err(e) => {
            error!("Failed to get blocks mined: {}", e);
            stats.insert("blocks_mined".to_string(), 0);
        }
    }
    
    // Get total rewards
    match conn.query_row(
        "SELECT SUM(total_rewards) FROM mining_stats",
        [],
        |row| row.get::<_, i64>(0),
    ) {
        Ok(count) => {
            stats.insert("total_rewards".to_string(), count);
        },
        Err(e) => {
            error!("Failed to get total rewards: {}", e);
            stats.insert("total_rewards".to_string(), 0);
        }
    }
    
    // Return stats
    HttpResponse::Ok().json(
        ApiResponse::success(stats, "Aggregate statistics retrieved successfully")
    )
} 