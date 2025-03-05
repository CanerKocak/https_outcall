use ic_agent::Agent;
use candid::Principal;
use anyhow::{Result, Context, anyhow};
use log::{info, warn, debug};
use serde_cbor::Value;

/// Get module hash from a canister using read_state_canister_info API (doesn't require controller privileges)
pub async fn get_module_hash(agent: &Agent, canister_id: &str) -> Result<String> {
    info!("Getting module hash for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Use read_state_canister_info to get the module_hash
    let path = "module_hash";
    debug!("Using read_state_canister_info with path: {}", path);
    
    // Call read_state_canister_info API
    let blob = agent.read_state_canister_info(principal, path)
        .await
        .context("Failed to call read_state_canister_info for module_hash")?;
    
    if blob.is_empty() {
        return Err(anyhow!("No module hash found for canister {}", canister_id));
    }
    
    // Convert module hash to hex string
    let hash_hex = blob.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>();
    
    info!("Successfully retrieved module hash for canister {}: {}", canister_id, hash_hex);
    Ok(hash_hex)
}

/// Get controllers for a canister using read_state_canister_info API (doesn't require controller privileges)
pub async fn get_controllers(agent: &Agent, canister_id: &str) -> Result<Vec<String>> {
    info!("Getting controllers for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Use read_state_canister_info to get the controllers
    let path = "controllers";
    debug!("Using read_state_canister_info with path: {}", path);
    
    // Call read_state_canister_info API
    let blob = agent.read_state_canister_info(principal, path)
        .await
        .context("Failed to call read_state_canister_info for controllers")?;
    
    if blob.is_empty() {
        return Err(anyhow!("No controllers found for canister {}", canister_id));
    }
    
    // Decode CBOR data
    let value: Value = serde_cbor::from_slice(&blob)
        .context("Failed to decode CBOR data for controllers")?;
    
    // Extract principals from CBOR data
    let controllers = match value {
        Value::Array(arr) => {
            let mut result = Vec::new();
            for item in arr {
                if let Value::Bytes(bytes) = item {
                    // Try to convert bytes to Principal
                    match Principal::try_from_slice(&bytes) {
                        Ok(principal) => result.push(principal.to_text()),
                        Err(e) => warn!("Failed to parse principal from bytes: {}", e),
                    }
                }
            }
            result
        },
        _ => return Err(anyhow!("Unexpected CBOR data format for controllers")),
    };
    
    info!("Successfully retrieved {} controllers for canister {}", controllers.len(), canister_id);
    for (i, controller) in controllers.iter().enumerate() {
        debug!("Controller {}: {}", i + 1, controller);
    }
    
    Ok(controllers)
}

/// Get both module hash and controllers in a single call
pub async fn get_canister_info(agent: &Agent, canister_id: &str) -> Result<(String, Vec<String>)> {
    let module_hash = get_module_hash(agent, canister_id).await?;
    let controllers = get_controllers(agent, canister_id).await?;
    
    Ok((module_hash, controllers))
} 