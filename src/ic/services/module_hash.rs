use ic_agent::Agent;
use candid::Principal;
use anyhow::{Result, Context};
use log::info;

/// Get module hash from a canister
pub async fn get_module_hash(agent: &Agent, canister_id: &str) -> Result<String> {
    info!("Getting module hash for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Get status from management canister to retrieve module hash
    let response = agent.query(&Principal::management_canister(), "canister_status")
        .with_arg(candid::encode_one(principal)?)
        .call()
        .await
        .context("Failed to call canister_status")?;
    
    // Decode the response
    #[derive(candid::CandidType, candid::Deserialize)]
    struct Status {
        module_hash: Option<Vec<u8>>,
    }
    
    let status: Status = candid::decode_one(&response)
        .context("Failed to decode canister status")?;
    
    // Extract and format module hash
    let module_hash = status.module_hash
        .ok_or_else(|| anyhow::anyhow!("No module hash found"))?;
    
    // Convert module hash to hex string
    let hash_hex = module_hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    info!("Successfully retrieved module hash for canister {}: {}", canister_id, hash_hex);
    Ok(hash_hex)
} 