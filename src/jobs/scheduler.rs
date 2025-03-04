use tokio::time::{self, Duration};
use log::{info, error};
use std::sync::Arc;

use crate::db::DbPool;
use crate::jobs::tasks::{update_tokens, update_miners};

/// Start the background job scheduler
pub async fn start_scheduler(db_pool: Arc<DbPool>) {
    info!("Starting background job scheduler");
    
    // Spawn a task for updating token info
    let token_db_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60)); // Every minute
        loop {
            interval.tick().await;
            if let Err(e) = update_tokens::run(token_db_pool.clone()).await {
                error!("Error updating tokens: {}", e);
            }
        }
    });
    
    // Spawn a task for updating miner info
    let miner_db_pool = db_pool.clone();
    tokio::spawn(async move {
        let mut interval = time::interval(Duration::from_secs(60)); // Every minute
        loop {
            interval.tick().await;
            if let Err(e) = update_miners::run(miner_db_pool.clone()).await {
                error!("Error updating miners: {}", e);
            }
        }
    });
    
    info!("Background job scheduler started");
} 