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
            average_block_time REAL,
            formatted_block_time TEXT,
            block_time_rating TEXT,
            circulating_supply INTEGER NOT NULL DEFAULT 0,
            mining_progress_percentage TEXT NOT NULL DEFAULT '0',
            current_block_reward INTEGER NOT NULL DEFAULT 0,
            formatted_block_reward TEXT NOT NULL DEFAULT '0',
            current_block_height INTEGER NOT NULL DEFAULT 0,
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

    // Create verified_module_hashes table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS verified_module_hashes (
            id TEXT PRIMARY KEY,
            hash TEXT NOT NULL UNIQUE,
            description TEXT NOT NULL,
            canister_type TEXT NOT NULL,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            last_updated INTEGER NOT NULL
        )",
        [],
    )?;

    // Create admins table for authentication
    conn.execute(
        "CREATE TABLE IF NOT EXISTS admins (
            id TEXT PRIMARY KEY,
            username TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            api_key TEXT NOT NULL UNIQUE,
            is_active INTEGER NOT NULL DEFAULT 1,
            created_at INTEGER NOT NULL,
            last_updated INTEGER NOT NULL
        )",
        [],
    )?;

    // Create indices for faster lookups
    conn.execute("CREATE INDEX IF NOT EXISTS idx_canisters_type ON canisters (type)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_canisters_principal ON canisters (principal)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_verified_module_hashes_hash ON verified_module_hashes (hash)", [])?;
    conn.execute("CREATE INDEX IF NOT EXISTS idx_verified_module_hashes_type ON verified_module_hashes (canister_type)", [])?;
    
    // Insert default verified module hash
    conn.execute(
        "INSERT OR IGNORE INTO verified_module_hashes (id, hash, description, canister_type, is_active, created_at, last_updated)
         VALUES ('default-hash', '5471eb4e9e70f245d8db1a1673d43ab5ff9443c6d1588f5bdf052bdc7e88f0a5', 'Default verified token hash', 'token', 1, strftime('%s','now'), strftime('%s','now'))",
        [],
    )?;
    
    info!("Database schema initialized successfully");
    Ok(())
}
