use ic_agent::Agent;
use candid::{Decode, Encode, Principal};
use anyhow::{Result, Context, anyhow};
use log::info;

use crate::ic::candid::miner::{MiningStats as CandidMiningStats, Result as CandidResult};
use crate::db::models::miner_info::{MinerInfo as DbMinerInfo, MinerType as DbMinerType};
use crate::db::models::mining_stats::MiningStats as DbMiningStats;

/// Get miner info from a miner canister
pub async fn get_miner_info(agent: &Agent, canister_id: &str) -> Result<(DbMinerInfo, Option<DbMiningStats>)> {
    info!("Getting miner info for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Prepare the arguments - empty tuple for no arguments
    let arg_bytes = Encode!(&()).context("Failed to encode arguments")?;
    
    // Call the canister to get miner info
    let info_response = agent.query(&principal, "get_info")
        .with_arg(arg_bytes)
        .call()
        .await
        .context("Failed to call get_info")?;
    
    // Decode the response
    let result = Decode!(info_response.as_slice(), CandidResult)
        .context("Failed to decode response")?;
    
    // Extract the miner info
    let miner_info = match result {
        CandidResult::Ok(info) => info,
        CandidResult::Err(err) => return Err(anyhow!("Canister error: {}", err)),
    };
    
    // Convert to JSON for raw_info
    let raw_info = serde_json::to_string(&miner_info)
        .context("Failed to serialize miner info")?;
    
    // Convert miner type
    let db_miner_type = match miner_info.miner_type {
        crate::ic::candid::miner::MinerType::Premium => DbMinerType::Premium,
        crate::ic::candid::miner::MinerType::Normal => DbMinerType::Normal,
        crate::ic::candid::miner::MinerType::Lite => DbMinerType::Lite,
    };
    
    // Convert current token
    let current_token = miner_info.current_token.map(|p| p.to_string());
    
    // Convert to database model
    let db_miner_info = DbMinerInfo::new(
        canister_id.to_string(),
        db_miner_type,
        miner_info.is_mining,
        current_token,
        miner_info.speed_percentage,
        miner_info.chunks_per_refresh,
        raw_info,
    );
    
    // Get mining stats if available
    let db_mining_stats = get_mining_stats(agent, canister_id).await?;
    
    info!("Successfully retrieved miner info for canister: {}", canister_id);
    Ok((db_miner_info, db_mining_stats))
}

/// Get mining stats from a miner canister
async fn get_mining_stats(agent: &Agent, canister_id: &str) -> Result<Option<DbMiningStats>> {
    info!("Getting mining stats for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Prepare the arguments - empty tuple for no arguments
    let arg_bytes = Encode!(&()).context("Failed to encode arguments")?;
    
    // Call the canister to get mining stats
    let stats_response = agent.query(&principal, "get_mining_stats")
        .with_arg(arg_bytes)
        .call()
        .await
        .context("Failed to call get_mining_stats")?;
    
    // Decode the response
    let stats_opt = Decode!(stats_response.as_slice(), Option<CandidMiningStats>)
        .context("Failed to decode mining stats response")?;
    
    // Return None if no stats available
    let stats = match stats_opt {
        Some(stats) => stats,
        None => return Ok(None),
    };
    
    // Convert to database model
    let db_mining_stats = DbMiningStats::new(
        canister_id.to_string(),
        stats.total_hashes,
        stats.blocks_mined,
        stats.chunks_since_refresh,
        stats.total_rewards,
        stats.last_hash_rate,
        stats.start_time,
    );
    
    info!("Successfully retrieved mining stats for canister: {}", canister_id);
    Ok(Some(db_mining_stats))
} 