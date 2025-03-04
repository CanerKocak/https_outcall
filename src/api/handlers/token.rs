use actix_web::{web, HttpResponse, Responder};
use log::{info, error};

use crate::db::DbPool;
use crate::db::models::token_info::TokenInfo;
use crate::api::handlers::ApiResponse;

/// Get all tokens
pub async fn get_all_tokens(db_pool: web::Data<DbPool>) -> impl Responder {
    info!("API: Get all tokens");
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<TokenInfo>>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match TokenInfo::find_all(&conn) {
        Ok(tokens) => {
            HttpResponse::Ok().json(
                ApiResponse::success(tokens, "Tokens retrieved successfully")
            )
        },
        Err(e) => {
            error!("Failed to get tokens: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<Vec<TokenInfo>>::error(&format!("Failed to get tokens: {}", e))
            )
        }
    }
}

/// Get a specific token
pub async fn get_token(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Get token: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(
                ApiResponse::<TokenInfo>::error(&format!("Database error: {}", e))
            );
        }
    };
    
    match TokenInfo::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(token)) => {
            HttpResponse::Ok().json(
                ApiResponse::success(token, "Token retrieved successfully")
            )
        },
        Ok(None) => {
            HttpResponse::NotFound().json(
                ApiResponse::<TokenInfo>::error(&format!("Token with canister ID {} not found", canister_id))
            )
        },
        Err(e) => {
            error!("Failed to get token: {}", e);
            HttpResponse::InternalServerError().json(
                ApiResponse::<TokenInfo>::error(&format!("Failed to get token: {}", e))
            )
        }
    }
}

/// Delete a token
pub async fn delete_token(
    db_pool: web::Data<DbPool>,
    path: web::Path<String>,
) -> impl Responder {
    let canister_id = path.into_inner();
    info!("API: Delete token: {}", canister_id);
    
    let conn = match db_pool.get() {
        Ok(conn) => conn,
        Err(e) => {
            error!("Failed to get database connection: {}", e);
            return HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Database error"));
        }
    };
    
    // First check if the token exists
    match TokenInfo::find_by_canister_id(&conn, &canister_id) {
        Ok(Some(_)) => {
            // Delete the token from the database
            match TokenInfo::delete(&conn, &canister_id) {
                Ok(true) => {
                    info!("Token deleted: {}", canister_id);
                    HttpResponse::Ok().json(ApiResponse::success(true, "Token deleted"))
                },
                Ok(false) => {
                    // This shouldn't happen as we already checked for existence
                    error!("Token not found after existence check: {}", canister_id);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete token"))
                },
                Err(e) => {
                    error!("Failed to delete token: {}", e);
                    HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete token"))
                }
            }
        },
        Ok(None) => {
            HttpResponse::NotFound().json(ApiResponse::<bool>::error(&format!("Token with ID {} not found", canister_id)))
        },
        Err(e) => {
            error!("Failed to check token existence: {}", e);
            HttpResponse::InternalServerError().json(ApiResponse::<bool>::error("Failed to delete token"))
        }
    }
} 