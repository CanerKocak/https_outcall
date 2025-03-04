use anyhow::{Result, Context};
use log::{info, error};
use std::sync::Arc;

use crate::db::DbPool;
use crate::db::models::canister::{Canister, CanisterType};
use crate::ic::agent::create_agent;
use crate::ic::services::token::get_token_info;

/// Run the update tokens task
pub async fn run(db_pool: Arc<DbPool>) -> Result<()> {
    info!("Running update tokens task");
    
    // Get a database connection
    let conn = db_pool.get().context("Failed to get database connection")?;
    
    // Get all token canisters
    let token_canisters = Canister::find_by_type(&conn, &CanisterType::Token)
        .context("Failed to get token canisters")?;
    
    info!("Found {} token canisters to update", token_canisters.len());
    
    // Create an IC agent
    let agent = create_agent("https://ic0.app").await
        .context("Failed to create IC agent")?;
    
    // Update each token
    for canister in token_canisters {
        info!("Updating token canister: {}", canister.canister_id);
        
        match get_token_info(&agent, &canister.canister_id).await {
            Ok(token_info) => {
                // Save the token info
                token_info.save(&conn).context("Failed to save token info")?;
                info!("Successfully updated token canister: {}", canister.canister_id);
            }
            Err(e) => {
                error!("Failed to update token canister {}: {}", canister.canister_id, e);
            }
        }
    }
    
    info!("Update tokens task completed");
    Ok(())
} 