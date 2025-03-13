use actix_web::{web, App, HttpServer, middleware, guard, HttpRequest};
use actix_files as fs;
use log::{info, error};
use std::path::Path;
use env_logger::Env;
use actix_cors::Cors;
use dotenv::dotenv;
use std::env;
use std::fs::File;
use rustls::{Certificate, PrivateKey, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys};
use std::io::{BufReader, Read};
use actix_web::http::Protocol;

mod db;
mod ic;
mod api;
// mod jobs; // Commented out since we're using WebSockets instead
mod websocket;
mod websocket_handler;
mod canister_notifications;

use db::models::admin::Admin;

fn load_rustls_config() -> Result<ServerConfig, std::io::Error> {
    // This function is now simplified since DigitalOcean handles SSL
    // We'll keep a minimal version for local development
    let cert_path = env::var("SSL_CERT_PATH").unwrap_or_else(|_| "certs/cert.pem".to_string());
    let key_path = env::var("SSL_KEY_PATH").unwrap_or_else(|_| "certs/key.pem".to_string());
    
    info!("Loading TLS configuration with cert_path: {}, key_path: {}", cert_path, key_path);
    
    // Load certificate and private key files
    let cert_file = File::open(&cert_path)?;
    let key_file = File::open(&key_path)?;
    
    // Read certificate and private key data
    let mut cert_reader = BufReader::new(cert_file);
    let mut key_reader = BufReader::new(key_file);
    
    // Parse certificate
    let cert_chain: Vec<Certificate> = certs(&mut cert_reader)?
        .into_iter()
        .map(Certificate)
        .collect();
    
    // Parse private key
    let keys: Vec<PrivateKey> = pkcs8_private_keys(&mut key_reader)?
        .into_iter()
        .map(PrivateKey)
        .collect();
    
    if keys.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "No private keys found"));
    }
    
    // Build TLS config
    let config = ServerConfig::builder()
        .with_safe_defaults()
        .with_no_client_auth()
        .with_single_cert(cert_chain, keys[0].clone())
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e.to_string()))?;
    
    Ok(config)
}

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
    
    // Create app factory
    let app_factory = move || {
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
            
            // Health check endpoint for load balancer
            .service(web::resource("/health").route(web::get().to(|_req: HttpRequest| async {
                actix_web::HttpResponse::Ok()
                    .content_type("application/json")
                    .body(r#"{"status":"ok"}"#)
            })))
            
            // Canister notification endpoint
            .service(
                web::resource("/miner-notifications")
                    .route(web::post().to(canister_notifications::handle_canister_notification))
            )
            
            // Serve static files
            .service(fs::Files::new("/static", "./static").show_files_listing())
            
            // Serve WebSocket test page - fixed with proper type annotation
            .service(web::resource("/test").route(web::get().to(
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
    };
    
    // Check if we should use HTTPS
    let use_https = env::var("USE_HTTPS").unwrap_or_else(|_| "false".to_string()) == "true";
    
    // Start HTTP server
    let mut server = HttpServer::new(app_factory);
    
    // For production with DigitalOcean load balancer, we only need to bind to port 8080
    // The load balancer handles SSL termination
    info!("Starting HTTP server on 0.0.0.0:8080 with HTTP/2 and HTTP/1.1 support");
    server = server.bind("0.0.0.0:8080")?
        // Add support for both HTTP/2 and HTTP/1.1 protocols
        .protocols(vec![Protocol::HTTP_2, Protocol::HTTP_11]);
    
    // Optionally bind to other ports for local development
    if use_https && env::var("LOCAL_DEV").unwrap_or_else(|_| "false".to_string()) == "true" {
        match load_rustls_config() {
            Ok(config) => {
                info!("Local development: Also binding to HTTPS on 0.0.0.0:443 and HTTP on 0.0.0.0:80");
                server = server.bind_rustls("0.0.0.0:443", config)?;
                server = server.bind("0.0.0.0:80")?;
            },
            Err(e) => {
                error!("Failed to load TLS configuration for local development: {}", e);
            }
        }
    }
    
    server.run().await
}