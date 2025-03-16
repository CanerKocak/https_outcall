use ic_agent::Agent;
use ic_agent::agent::http_transport::reqwest_transport::ReqwestHttpReplicaV2Transport;
use ic_agent::identity::{AnonymousIdentity, Identity, Secp256k1Identity};
use anyhow::{Result, Context, anyhow};
use log::{info, warn, error};
use std::path::{Path, PathBuf};
use std::fs::{File, create_dir_all};
use std::io::{Read, Write};
use std::sync::OnceLock;
use ring::signature::Ed25519KeyPair;

// Default identity directory and file path
const DEFAULT_IDENTITY_DIR: &str = "data/identity";
const DEFAULT_IDENTITY_FILE: &str = "identity.pem";

// Global identity file path
static IDENTITY_FILE_PATH: OnceLock<String> = OnceLock::new();

/// Set the global identity file path
pub fn set_identity_file_path(path: String) {
    let _ = IDENTITY_FILE_PATH.set(path);
}

/// Get the global identity file path
pub fn get_identity_file_path() -> Option<&'static String> {
    IDENTITY_FILE_PATH.get()
}

/// Initialize the identity system, ensuring a valid identity exists
pub fn init_identity() -> Result<String> {
    // Check if identity directory exists, create if not
    let identity_dir = PathBuf::from(DEFAULT_IDENTITY_DIR);
    if !identity_dir.exists() {
        info!("Creating identity directory: {}", identity_dir.display());
        create_dir_all(&identity_dir).context("Failed to create identity directory")?;
    }
    
    // Full path to the identity file
    let identity_path = identity_dir.join(DEFAULT_IDENTITY_FILE);
    let identity_path_str = identity_path.to_string_lossy().to_string();
    
    // Check if identity file exists, create if not
    if !identity_path.exists() {
        info!("Identity file not found, creating new identity at: {}", identity_path.display());
        create_new_identity_file(&identity_path_str).context("Failed to create new identity file")?;
    } else {
        info!("Using existing identity file: {}", identity_path.display());
        // Validate the identity file
        match validate_identity_file(&identity_path_str) {
            Ok(_) => info!("Identity file validated successfully"),
            Err(e) => {
                warn!("Invalid identity file: {}", e);
                warn!("Creating new identity file");
                create_new_identity_file(&identity_path_str).context("Failed to create new identity file")?;
            }
        }
    }
    
    // Set the global identity file path
    set_identity_file_path(identity_path_str.clone());
    
    Ok(identity_path_str)
}

/// Create a new identity file at the specified path
fn create_new_identity_file(path: &str) -> Result<()> {
    // Generate a random seed for the key pair
    let rng = ring::rand::SystemRandom::new();
    let pkcs8_bytes = ring::signature::Ed25519KeyPair::generate_pkcs8(&rng)
        .context("Failed to generate Ed25519 key pair")?;
    
    // Encode as PEM
    let pem = pem::encode(&pem::Pem {
        tag: "PRIVATE KEY".to_string(),
        contents: pkcs8_bytes.as_ref().to_vec(),
    });
    
    // Ensure the directory exists
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            create_dir_all(parent).context("Failed to create parent directory for identity file")?;
        }
    }
    
    // Write the PEM to file
    let mut file = File::create(path).context("Failed to create identity file")?;
    file.write_all(pem.as_bytes()).context("Failed to write identity to file")?;
    
    info!("Created new identity file at: {}", path);
    Ok(())
}

/// Validate an identity file to ensure it's a valid PEM file
fn validate_identity_file(path: &str) -> Result<()> {
    // Try to read the file
    let pem_path = Path::new(path);
    let mut pem_file = File::open(pem_path).context("Failed to open PEM file")?;
    let mut pem_content = Vec::new();
    pem_file.read_to_end(&mut pem_content).context("Failed to read PEM file")?;
    
    // Try to parse it as a Secp256k1Identity
    match Secp256k1Identity::from_pem(&*pem_content) {
        Ok(_) => return Ok(()),
        Err(_) => {}
    }
    
    // If it fails as Secp256k1, try as Ed25519KeyPair
    let pem_str = String::from_utf8(pem_content.clone())
        .context("Failed to convert PEM content to string")?;
    
    // Try to parse the PEM
    match pem::parse(pem_str) {
        Ok(parsed_pem) => {
            // Try to create an Ed25519KeyPair from the parsed PEM
            match Ed25519KeyPair::from_pkcs8(&parsed_pem.contents) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow!("Invalid Ed25519 key pair: {:?}", e))
            }
        },
        Err(e) => Err(anyhow!("Invalid PEM format: {}", e))
    }
}

/// Create an IC agent for interacting with canisters
/// This will use the global identity file if available, or initialize a new one if needed
pub async fn create_agent(url: &str) -> Result<Agent> {
    // Check if we have a global identity file path
    let identity_path = if let Some(path) = get_identity_file_path() {
        path.clone()
    } else {
        // Initialize identity system if no path is set
        match init_identity() {
            Ok(path) => path,
            Err(e) => {
                error!("Failed to initialize identity: {}", e);
                warn!("Falling back to anonymous identity");
                return create_agent_with_anonymous_identity(url).await;
            }
        }
    };
    
    // Try to create an agent with the identity
    match create_agent_with_identity_from_pem(url, &identity_path).await {
        Ok(agent) => Ok(agent),
        Err(e) => {
            warn!("Failed to create agent with identity file {}: {}", identity_path, e);
            warn!("Falling back to anonymous identity");
            create_agent_with_anonymous_identity(url).await
        }
    }
}

/// Create an IC agent with anonymous identity
pub async fn create_agent_with_anonymous_identity(url: &str) -> Result<Agent> {
    info!("Creating IC agent with anonymous identity and URL: {}", url);
    
    // Create the transport
    let transport = ReqwestHttpReplicaV2Transport::create(url)
        .context("Failed to create transport")?;
    
    // Create the agent with anonymous identity
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(AnonymousIdentity)
        .build()
        .context("Failed to build agent")?;
    
    // Initialize the agent
    agent.fetch_root_key().await.context("Failed to fetch root key")?;
    
    info!("IC agent with anonymous identity created successfully");
    Ok(agent)
}

/// Create an IC agent with a specific identity
pub async fn create_agent_with_identity(url: &str, identity: Box<dyn Identity>) -> Result<Agent> {
    info!("Creating IC agent with custom identity and URL: {}", url);
    
    // Create the transport
    let transport = ReqwestHttpReplicaV2Transport::create(url)
        .context("Failed to create transport")?;
    
    // Create the agent with the provided identity
    let agent = Agent::builder()
        .with_transport(transport)
        .with_identity(identity)
        .build()
        .context("Failed to build agent")?;
    
    // Initialize the agent
    agent.fetch_root_key().await.context("Failed to fetch root key")?;
    
    info!("IC agent with custom identity created successfully");
    Ok(agent)
}

/// Create an IC agent with a Secp256k1 identity from a PEM file
pub async fn create_agent_with_identity_from_pem(url: &str, pem_file_path: &str) -> Result<Agent> {
    info!("Creating IC agent with identity from PEM file: {}", pem_file_path);
    
    // Read the PEM file
    let pem_path = Path::new(pem_file_path);
    let mut pem_file = File::open(pem_path).context("Failed to open PEM file")?;
    let mut pem_content = Vec::new();
    pem_file.read_to_end(&mut pem_content).context("Failed to read PEM file")?;
    
    // Create the identity
    let identity = Secp256k1Identity::from_pem(&*pem_content)
        .context("Failed to create Secp256k1Identity from PEM file")?;
    
    // Create the agent with the identity
    create_agent_with_identity(url, Box::new(identity)).await
}
