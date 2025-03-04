use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MinerType {
    Premium,
    Normal,
    Lite,
}

impl ToString for MinerType {
    fn to_string(&self) -> String {
        match self {
            MinerType::Premium => "Premium".to_string(),
            MinerType::Normal => "Normal".to_string(),
            MinerType::Lite => "Lite".to_string(),
        }
    }
}

impl TryFrom<String> for MinerType {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "Premium" => Ok(MinerType::Premium),
            "Normal" => Ok(MinerType::Normal),
            "Lite" => Ok(MinerType::Lite),
            _ => Err(anyhow::anyhow!("Invalid miner type: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MinerInfo {
    pub canister_id: String,
    pub miner_type: MinerType,
    pub is_mining: bool,
    pub current_token: Option<String>,
    pub speed_percentage: u8,
    pub chunks_per_refresh: u64,
    pub last_updated: i64,
    pub raw_info: String,
}

impl MinerInfo {
    pub fn new(
        canister_id: String,
        miner_type: MinerType,
        is_mining: bool,
        current_token: Option<String>,
        speed_percentage: u8,
        chunks_per_refresh: u64,
        raw_info: String,
    ) -> Self {
        Self {
            canister_id,
            miner_type,
            is_mining,
            current_token,
            speed_percentage,
            chunks_per_refresh,
            last_updated: Utc::now().timestamp(),
            raw_info,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        let miner_type_str: String = row.get("miner_type")?;
        let miner_type = MinerType::try_from(miner_type_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "Invalid miner type".to_string(), rusqlite::types::Type::Text))?;

        Ok(Self {
            canister_id: row.get("canister_id")?,
            miner_type,
            is_mining: row.get::<_, i64>("is_mining")? != 0,
            current_token: row.get("current_token")?,
            speed_percentage: row.get("speed_percentage")?,
            chunks_per_refresh: row.get("chunks_per_refresh")?,
            last_updated: row.get("last_updated")?,
            raw_info: row.get("raw_info")?,
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO miner_info (canister_id, miner_type, is_mining, current_token, speed_percentage, chunks_per_refresh, last_updated, raw_info)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
             ON CONFLICT(canister_id) DO UPDATE SET
             miner_type = ?2,
             is_mining = ?3,
             current_token = ?4,
             speed_percentage = ?5,
             chunks_per_refresh = ?6,
             last_updated = ?7,
             raw_info = ?8",
            params![
                self.canister_id,
                self.miner_type.to_string(),
                self.is_mining as i64,
                self.current_token,
                self.speed_percentage,
                self.chunks_per_refresh,
                self.last_updated,
                self.raw_info,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_canister_id(conn: &Connection, canister_id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT canister_id, miner_type, is_mining, current_token, speed_percentage, chunks_per_refresh, last_updated, raw_info
             FROM miner_info
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
            "SELECT m.canister_id, m.miner_type, m.is_mining, m.current_token, m.speed_percentage, m.chunks_per_refresh, m.last_updated, m.raw_info
             FROM miner_info m
             JOIN canisters c ON m.canister_id = c.canister_id
             ORDER BY m.last_updated DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut miners = Vec::new();
        for miner in rows {
            miners.push(miner?);
        }
        
        Ok(miners)
    }

    pub fn find_by_token(conn: &Connection, token_canister_id: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT m.canister_id, m.miner_type, m.is_mining, m.current_token, m.speed_percentage, m.chunks_per_refresh, m.last_updated, m.raw_info
             FROM miner_info m
             JOIN canisters c ON m.canister_id = c.canister_id
             WHERE m.current_token = ?1
             ORDER BY m.last_updated DESC",
        )?;
        
        let rows = stmt.query_map(params![token_canister_id], |row| Self::from_row(row))?;
        
        let mut miners = Vec::new();
        for miner in rows {
            miners.push(miner?);
        }
        
        Ok(miners)
    }

    pub fn delete(conn: &Connection, canister_id: &str) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM miner_info WHERE canister_id = ?1",
            params![canister_id],
        )?;
        
        Ok(rows_affected > 0)
    }
} 