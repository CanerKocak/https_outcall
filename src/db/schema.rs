use rusqlite::{Connection, Result};
use log::info;

/// Initialize the database schema
pub fn init_db(conn: &Connection) -> Result<()> {
    info!("Initializing database schema");
    
    // Create canisters table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS canisters (
            id TEXT PRIMARY KEY,
            principal TEXT NOT NULL,
            canister_id TEXT NOT NULL UNIQUE,
            type TEXT NOT NULL,
            module_hash TEXT,
            created_at INTEGER NOT NULL,
            last_updated INTEGER NOT NULL
        )",
        [],
    )?;

    // Create token_info table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS token_info (
            canister_id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            ticker TEXT NOT NULL,
            decimals INTEGER NOT NULL,
            total_supply INTEGER NOT NULL,
            transfer_fee INTEGER NOT NULL,
            logo TEXT,
            last_updated INTEGER NOT NULL,
            raw_info TEXT NOT NULL,
            FOREIGN KEY (canister_id) REFERENCES canisters (canister_id)
        )",
        [],
    )?;

    // Create miner_info table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS miner_info (
            canister_id TEXT PRIMARY KEY,
            miner_type TEXT NOT NULL,
            is_mining INTEGER NOT NULL,
            current_token TEXT,
            speed_percentage INTEGER NOT NULL,
            chunks_per_refresh INTEGER NOT NULL,
            last_updated INTEGER NOT NULL,
            raw_info TEXT NOT NULL,
            FOREIGN KEY (canister_id) REFERENCES canisters (canister_id)
        )",
        [],
    )?;

    // Create mining_stats table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS mining_stats (
            canister_id TEXT PRIMARY KEY,
            total_hashes INTEGER NOT NULL,
            blocks_mined INTEGER NOT NULL,
            chunks_since_refresh INTEGER NOT NULL,
            total_rewards INTEGER NOT NULL,
            last_hash_rate REAL NOT NULL,
            start_time INTEGER NOT NULL,
            last_updated INTEGER NOT NULL,
            FOREIGN KEY (canister_id) REFERENCES canisters (canister_id)
        )",
        [],
    )?;

    // Create indices for faster lookups
    conn.execute("CREATE INDEX IF NOT EXISTS idx_canisters_type ON canisters (type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_canisters_principal ON canisters (principal)", [])?;
    
    info!("Database schema initialized successfully");
    Ok(())
} 