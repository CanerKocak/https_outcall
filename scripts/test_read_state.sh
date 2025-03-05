#!/bin/bash

# Test script for read_state implementation
# This script tests the ability to retrieve module hash and controllers
# without requiring controller privileges

set -e

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m' # No Color

echo -e "${YELLOW}Testing read_state implementation for canister information retrieval${NC}"

# Check if a canister ID was provided
if [ $# -eq 0 ]; then
    echo -e "${RED}Error: No canister ID provided${NC}"
    echo "Usage: $0 <canister_id>"
    echo "Example: $0 sk4hs-faaaa-aaaag-at3rq-cai"
    exit 1
fi

CANISTER_ID=$1
echo -e "${GREEN}Using canister ID: ${CANISTER_ID}${NC}"

# Create a temporary Rust project for testing
TEMP_DIR=$(mktemp -d)
mkdir -p ${TEMP_DIR}/src

# Create main.rs file
cat > ${TEMP_DIR}/src/main.rs << 'EOF'
use ic_agent::Agent;
use candid::Principal;
use serde_cbor::Value;
use anyhow::{Result, Context, anyhow};

/// Get module hash from a canister using read_state_canister_info API
async fn get_module_hash(agent: &Agent, canister_id: &str) -> Result<String> {
    println!("Getting module hash for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Use read_state_canister_info to get the module_hash
    let path = "module_hash";
    println!("Using read_state_canister_info with path: {}", path);
    
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
    
    println!("Successfully retrieved module hash: {}", hash_hex);
    Ok(hash_hex)
}

/// Get controllers for a canister using read_state_canister_info API
async fn get_controllers(agent: &Agent, canister_id: &str) -> Result<Vec<String>> {
    println!("Getting controllers for canister: {}", canister_id);
    
    // Parse the canister ID
    let principal = Principal::from_text(canister_id)
        .context(format!("Invalid canister ID: {}", canister_id))?;
    
    // Use read_state_canister_info to get the controllers
    let path = "controllers";
    println!("Using read_state_canister_info with path: {}", path);
    
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
                        Err(e) => println!("Failed to parse principal from bytes: {}", e),
                    }
                }
            }
            result
        },
        _ => return Err(anyhow!("Unexpected CBOR data format for controllers")),
    };
    
    println!("Successfully retrieved {} controllers:", controllers.len());
    for (i, controller) in controllers.iter().enumerate() {
        println!("  Controller {}: {}", i + 1, controller);
    }
    
    Ok(controllers)
}

#[tokio::main]
async fn main() -> Result<()> {
    // Get canister ID from command line
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        return Err(anyhow!("Please provide a canister ID as an argument"));
    }
    let canister_id = &args[1];
    
    // Create agent
    println!("Creating IC agent...");
    let transport = ic_agent::agent::http_transport::reqwest_transport::ReqwestHttpReplicaV2Transport::create("https://ic0.app")?;
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(ic_agent::identity::AnonymousIdentity)
        .build()?;
    
    // Initialize the agent
    println!("Fetching root key...");
    agent.fetch_root_key().await?;
    
    // Get module hash
    println!("\n--- Module Hash ---");
    match get_module_hash(&agent, canister_id).await {
        Ok(hash) => println!("Module hash: {}", hash),
        Err(e) => println!("Error getting module hash: {}", e),
    }
    
    // Get controllers
    println!("\n--- Controllers ---");
    match get_controllers(&agent, canister_id).await {
        Ok(controllers) => {
            println!("Controllers retrieved successfully");
        },
        Err(e) => println!("Error getting controllers: {}", e),
    }
    
    Ok(())
}
EOF

# Create a Cargo.toml file
cat > ${TEMP_DIR}/Cargo.toml << 'EOF'
[package]
name = "test_read_state"
version = "0.1.0"
edition = "2021"

[dependencies]
ic-agent = "0.30"
candid = "0.9"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
serde_cbor = "0.11"
EOF

echo -e "${YELLOW}Compiling test program...${NC}"
cd ${TEMP_DIR}
cargo build --quiet

echo -e "${YELLOW}Running test with canister ID: ${CANISTER_ID}${NC}"
cargo run --quiet -- ${CANISTER_ID}

echo -e "${GREEN}Test completed. Cleaning up...${NC}"
cd - > /dev/null
rm -rf ${TEMP_DIR}

echo -e "${GREEN}Done!${NC}" 