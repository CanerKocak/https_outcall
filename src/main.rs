use actix_web::{web, App, HttpServer, middleware, guard, HttpRequest};
use actix_files as fs;
use log::{info, error};
use std::path::Path;
use env_logger::Env;
use actix_cors::Cors;
use dotenv::dotenv;

mod db;
mod ic;
mod api;
// mod jobs; // Commented out since we're using WebSockets instead
mod websocket;
mod websocket_handler;
mod canister_notifications;

use db::models::admin::Admin;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();
    
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
    
    // Initialize WebSocket server
    let websocket_server = websocket::init_websocket_server();
    
    // Start cache cleanup tasks
    canister_notifications::start_cache_cleanup_task();
    api::handlers::claude::start_cache_cleanup_task();
    
    // Start HTTP server
    info!("Starting server on [::]:8080");
    
    HttpServer::new(move || {
        // Configure CORS middleware
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);
            
        App::new()
            // Enable CORS middleware
            .wrap(cors)
            // Enable logger middleware
            .wrap(middleware::Logger::default())
            
            // Database connection pool
            .app_data(web::Data::new(db_pool.clone()))
            
            // WebSocket server data
            .app_data(web::Data::new(websocket_server.clone()))
            
            // API routes
            .configure(api::configure_routes)
            
            // WebSocket route
            .service(web::resource("/ws").route(web::get().to(websocket_handler::websocket_route)))
            
            // WebSocket status endpoint
            .service(web::resource("/ws-status").route(web::get().to(websocket_handler::websocket_status)))
            
            // Canister notification endpoint
            .service(
                web::resource("/miner-notifications")
                    .route(web::post().to(canister_notifications::handle_canister_notification))
            )
            
            // Serve static files
            .service(fs::Files::new("/static", "./static").show_files_listing())
            
            // Serve WebSocket test page - fixed with proper type annotation
            .service(web::resource("/test").route(web::get().guard(guard::Header("accept", "text/html")).to(
                |_req: HttpRequest| async {
                    actix_web::HttpResponse::Ok()
                        .content_type("text/html; charset=utf-8")
                        .body(include_str!("../static/websocket_test.html"))
                }
            )))
            
            // Default route - fixed to use a proper service
            .service(
                web::resource("/")
                    .route(web::get().to(|_req: HttpRequest| async {
                        actix_web::HttpResponse::Found()
                            .append_header(("Location", "/test"))
                            .finish()
                    }))
            )
    })
    .bind(("::", 8080))?
    .run()
    .await
}