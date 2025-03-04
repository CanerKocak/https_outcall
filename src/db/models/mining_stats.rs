use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MiningStats {
    pub canister_id: String,
    pub total_hashes: u64,
    pub blocks_mined: u64,
    pub chunks_since_refresh: u64,
    pub total_rewards: u64,
    pub last_hash_rate: f64,
    pub start_time: u64,
    pub last_updated: i64,
}

impl MiningStats {
    pub fn new(
        canister_id: String,
        total_hashes: u64,
        blocks_mined: u64,
        chunks_since_refresh: u64,
        total_rewards: u64,
        last_hash_rate: f64,
        start_time: u64,
    ) -> Self {
        Self {
            canister_id,
            total_hashes,
            blocks_mined,
            chunks_since_refresh,
            total_rewards,
            last_hash_rate,
            start_time,
            last_updated: Utc::now().timestamp(),
        }
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            canister_id: row.get("canister_id")?,
            total_hashes: row.get("total_hashes")?,
            blocks_mined: row.get("blocks_mined")?,
            chunks_since_refresh: row.get("chunks_since_refresh")?,
            total_rewards: row.get("total_rewards")?,
            last_hash_rate: row.get("last_hash_rate")?,
            start_time: row.get("start_time")?,
            last_updated: row.get("last_updated")?,
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO mining_stats (canister_id, total_hashes, blocks_mined, chunks_since_refresh, total_rewards, last_hash_rate, start_time, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(canister_id) DO UPDATE SET
             total_hashes = ?2,
             blocks_mined = ?3,
             chunks_since_refresh = ?4,
             total_rewards = ?5,
             last_hash_rate = ?6,
             start_time = ?7,
             last_updated = ?8",
            params![
                self.canister_id,
                self.total_hashes,
                self.blocks_mined,
                self.chunks_since_refresh,
                self.total_rewards,
                self.last_hash_rate,
                self.start_time,
                self.last_updated,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_canister_id(conn: &Connection, canister_id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT canister_id, total_hashes, blocks_mined, chunks_since_refresh, total_rewards, last_hash_rate, start_time, last_updated
             FROM mining_stats
             WHERE canister_id = ?1",
        )?;
        
        let mut rows = stmt.query(params![canister_id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT ms.canister_id, ms.total_hashes, ms.blocks_mined, ms.chunks_since_refresh, ms.total_rewards, ms.last_hash_rate, ms.start_time, ms.last_updated
             FROM mining_stats ms
             JOIN canisters c ON ms.canister_id = c.canister_id
             ORDER BY ms.last_updated DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut stats = Vec::new();
        for stat in rows {
            stats.push(stat?);
        }
        
        Ok(stats)
    }

    pub fn delete(conn: &Connection, canister_id: &str) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM mining_stats WHERE canister_id = ?1",
            params![canister_id],
        )?;
        
        Ok(rows_affected > 0)
    }
} 