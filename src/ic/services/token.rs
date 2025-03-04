use ic_agent::Agent;
use candid::{Decode, Encode, Principal};
use anyhow::{Result, Context, anyhow};
use log::info;

use crate::ic::candid::token::{Result as CandidResult};
use crate::db::models::token_info::TokenInfo as DbTokenInfo;

/// Get token info from a token canister
pub async fn get_token_info(agent: &Agent, canister_id: &str) -> Result<DbTokenInfo> {
    info!("Getting token info for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Prepare the arguments - empty tuple for no arguments
    let arg_bytes = Encode!(&()).context("Failed to encode arguments")?;
    
    // Call the canister
    let response = agent.query(&principal, "get_info")
        .with_arg(arg_bytes)
        .call()
        .await
        .context("Failed to call get_info")?;
    
    // Decode the response
    let result = Decode!(response.as_slice(), CandidResult)
        .context("Failed to decode response")?;
    
    // Extract the token info
    let token_info = match result {
        CandidResult::Ok(info) => info,
        CandidResult::Err(err) => return Err(anyhow!("Canister error: {}", err)),
    };
    
    // Convert to JSON for raw_info
    let raw_info = serde_json::to_string(&token_info)
        .context("Failed to serialize token info")?;
    
    // Convert to database model
    let db_token_info = DbTokenInfo::new(
        canister_id.to_string(),
        token_info.name,
        token_info.ticker,
        token_info.decimals,
        token_info.total_supply,
        token_info.transfer_fee,
        token_info.logo,
        raw_info,
    );
    
    info!("Successfully retrieved token info for canister: {}", canister_id);
    Ok(db_token_info)
} 