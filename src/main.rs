use actix_web::{web, App, HttpServer};
use log::{info, error};
use std::path::Path;
use std::sync::Arc;
use env_logger::Env;

mod db;
mod ic;
mod api;
mod jobs;

use db::models::admin::Admin;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    info!("Starting ICP Canister Registry");
    
    // Initialize database
    let db_path = Path::new("data/registry.db");
    let db_pool = match db::init_pool(db_path) {
        Ok(pool) => {
            info!("Database initialized successfully");
            pool
        },
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()));
        }
    };
    
    // Create shared database pool
    let db_pool = Arc::new(db_pool);
    
    // Create default admin account if none exists
    match db_pool.get() {
        Ok(conn) => {
            if let Err(e) = Admin::create_admin_if_none_exists(&conn, "admin", "admin123") {
                error!("Failed to create default admin: {}", e);
            } else {
                info!("Checked/created default admin account");
            }
        },
        Err(e) => {
            error!("Failed to get database connection: {}", e);
        }
    }
    
    // Start background job scheduler
    let scheduler_db_pool = db_pool.clone();
    tokio::spawn(async move {
        jobs::start_scheduler(scheduler_db_pool).await;
    });
    
    // Start HTTP server
    info!("Starting server on [::]:8080");
    
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db_pool.clone()))
            .configure(api::configure_routes)
    })
    .bind(("::", 8080))?
    .run()
    .await
}