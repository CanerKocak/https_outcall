use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use std::path::Path;
use log::{info, error};

use crate::db::schema;

pub type DbPool = Pool<SqliteConnectionManager>;

/// Initialize the database connection pool
pub fn init_pool(db_path: &Path) -> Result<DbPool, r2d2::Error> {
    info!("Initializing database connection pool at {:?}", db_path);
    
    // Create the database directory if it doesn't exist
    if let Some(parent) = db_path.parent() {
        // Create the directory and handle errors separately
        if !parent.exists() {
            match std::fs::create_dir_all(parent) {
                Ok(_) => info!("Created database directory: {:?}", parent),
                Err(e) => {
                    error!("Failed to create database directory: {}", e);
                    // Just panic here, as we can't proceed without the directory
                    panic!("Failed to create database directory: {}", e);
                }
            }
        }
    }
    
    // Create the connection manager
    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|conn| {
            // Enable foreign keys
            conn.execute_batch("PRAGMA foreign_keys = ON;")?;
            // Initialize the database schema
            schema::init_db(conn)?;
            Ok(())
        });
    
    // Create the connection pool
    let pool = Pool::builder()
        .max_size(10) // Maximum number of connections in the pool
        .build(manager)?;
    
    info!("Database connection pool initialized successfully");
    Ok(pool)
} 