use ic_agent::Agent;
use ic_agent::agent::http_transport::reqwest_transport::ReqwestHttpReplicaV2Transport;
use anyhow::{Result, Context};
use log::info;

/// Create an IC agent for interacting with canisters
pub async fn create_agent(url: &str) -> Result<Agent> {
    info!("Creating IC agent with URL: {}", url);
    
    // Create the transport
    let transport = ReqwestHttpReplicaV2Transport::create(url)
        .context("Failed to create transport")?;
    
    // Create the agent
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(ic_agent::identity::AnonymousIdentity)
        .build()
        .context("Failed to build agent")?;
    
    // Initialize the agent
    agent.fetch_root_key().await.context("Failed to fetch root key")?;
    
    info!("IC agent created successfully");
    Ok(agent)
} 