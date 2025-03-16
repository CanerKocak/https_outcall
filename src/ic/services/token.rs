use ic_agent::Agent;
use candid::{Decode, Encode, Principal};
use anyhow::{Result, Context, anyhow};
use log::info;

use crate::ic::candid::token::{AllInfoResult, TokenAllInfo};
use crate::db::models::token_info::TokenInfo as DbTokenInfo;

/// Get token all info from a token canister
pub async fn get_token_all_info(agent: &Agent, canister_id: &str) -> Result<DbTokenInfo> {
    info!("Getting token all info for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Prepare the arguments - empty tuple for no arguments
    let arg_bytes = Encode!(&()).context("Failed to encode arguments")?;
    
    // Call the canister
    let response = agent.query(&principal, "get_all_info")
        .with_arg(arg_bytes)
        .call()
        .await
        .context("Failed to call get_all_info")?;
    
    // Decode the response
    let result = Decode!(response.as_slice(), AllInfoResult)
        .context("Failed to decode response")?;
    
    // Extract the token info
    let token_all_info = match result {
        AllInfoResult::Ok(info) => info,
        AllInfoResult::Err(err) => return Err(anyhow!("Canister error: {}", err)),
    };
    
    // Convert to JSON for raw_info
    let raw_info = serde_json::to_string(&token_all_info)
        .context("Failed to serialize token info")?;
    
    // Convert to database model with all info
    let db_token_info = DbTokenInfo::new_all_info(
        canister_id.to_string(),
        token_all_info.name,
        token_all_info.ticker,
        token_all_info.decimals,
        token_all_info.total_supply,
        token_all_info.transfer_fee,
        token_all_info.logo,
        token_all_info.average_block_time,
        token_all_info.formatted_block_time,
        token_all_info.block_time_rating,
        token_all_info.circulating_supply,
        token_all_info.mining_progress_percentage,
        token_all_info.current_block_reward,
        token_all_info.formatted_block_reward,
        token_all_info.current_block_height,
        raw_info,
    );
    
    info!("Successfully retrieved token all info for canister: {}", canister_id);
    Ok(db_token_info)
}

// Keep the old function for backward compatibility but make it call the new one
pub async fn get_token_info(agent: &Agent, canister_id: &str) -> Result<DbTokenInfo> {
    get_token_all_info(agent, canister_id).await
}
