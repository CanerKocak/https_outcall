use std::fs::File;
use std::io::Write;
use std::path::Path;
use log::{info, error};
use anyhow::{Result, Context};

use crate::ic::candid::token::token_interface;
use crate::ic::candid::miner::miner_interface;

/// Generate Candid interface files for clients
pub fn generate_interface_files(output_dir: &str) -> Result<()> {
    info!("Generating Candid interface files to {}", output_dir);
    
    // Create output directory if it doesn't exist
    if !Path::new(output_dir).exists() {
        std::fs::create_dir_all(output_dir)
            .context(format!("Failed to create output directory: {}", output_dir))?;
    }
    
    // Generate token interface
    let token_idl = token_interface();
    let token_path = format!("{}/token.did", output_dir);
    let mut token_file = File::create(&token_path)
        .context(format!("Failed to create token interface file: {}", token_path))?;
    
    if let candid::IDLValue::Text(idl_text) = token_idl {
        token_file.write_all(idl_text.as_bytes())
            .context("Failed to write token interface")?;
        info!("Token interface generated: {}", token_path);
    } else {
        error!("Unexpected token interface format");
        return Err(anyhow::anyhow!("Unexpected token interface format"));
    }
    
    // Generate miner interface
    let miner_idl = miner_interface();
    let miner_path = format!("{}/miner.did", output_dir);
    let mut miner_file = File::create(&miner_path)
        .context(format!("Failed to create miner interface file: {}", miner_path))?;
    
    if let candid::IDLValue::Text(idl_text) = miner_idl {
        miner_file.write_all(idl_text.as_bytes())
            .context("Failed to write miner interface")?;
        info!("Miner interface generated: {}", miner_path);
    } else {
        error!("Unexpected miner interface format");
        return Err(anyhow::anyhow!("Unexpected miner interface format"));
    }
    
    info!("Interface files generated successfully");
    Ok(())
} 