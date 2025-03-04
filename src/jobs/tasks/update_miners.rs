use anyhow::{Result, Context};
use log::{info, error};
use std::sync::Arc;

use crate::db::DbPool;
use crate::db::models::canister::{Canister, CanisterType};
use crate::ic::agent::create_agent;
use crate::ic::services::miner::get_miner_info;

/// Run the update miners task
pub async fn run(db_pool: Arc<DbPool>) -> Result<()> {
    info!("Running update miners task");
    
    // Get a database connection
    let conn = db_pool.get().context("Failed to get database connection")?;
    
    // Get all miner canisters
    let miner_canisters = Canister::find_by_type(&conn, &CanisterType::Miner)
        .context("Failed to get miner canisters")?;
    
    info!("Found {} miner canisters to update", miner_canisters.len());
    
    // Create an IC agent
    let agent = create_agent("https://ic0.app").await
        .context("Failed to create IC agent")?;
    
    // Update each miner
    for canister in miner_canisters {
        info!("Updating miner canister: {}", canister.canister_id);
        
        match get_miner_info(&agent, &canister.canister_id).await {
            Ok((miner_info, mining_stats_opt)) => {
                // Save the miner info
                miner_info.save(&conn).context("Failed to save miner info")?;
                
                // Save the mining stats if available
                if let Some(mining_stats) = mining_stats_opt {
                    mining_stats.save(&conn).context("Failed to save mining stats")?;
                }
                
                info!("Successfully updated miner canister: {}", canister.canister_id);
            }
            Err(e) => {
                error!("Failed to update miner canister {}: {}", canister.canister_id, e);
            }
        }
    }
    
    info!("Update miners task completed");
    Ok(())
} 