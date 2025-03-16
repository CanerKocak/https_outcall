use rusqlite::{params, Connection, Result, Row};
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenInfo {
    pub canister_id: String,
    pub name: String,
    pub ticker: String,
    pub decimals: u8,
    pub total_supply: u64,
    pub transfer_fee: u64,
    pub logo: Option<String>,
    pub last_updated: i64,
    pub raw_info: String,
    // Additional fields from TokenAllInfo
    pub average_block_time: Option<f64>,
    pub formatted_block_time: Option<String>,
    pub block_time_rating: Option<String>,
    pub circulating_supply: u64,
    pub mining_progress_percentage: String,
    pub current_block_reward: u64,
    pub formatted_block_reward: String,
    pub current_block_height: u64,
}

impl TokenInfo {
    pub fn new(
        canister_id: String,
        name: String,
        ticker: String,
        decimals: u8,
        total_supply: u64,
        transfer_fee: u64,
        logo: Option<String>,
        raw_info: String,
    ) -> Self {
        Self {
            canister_id,
            name,
            ticker,
            decimals,
            total_supply,
            transfer_fee,
            logo,
            last_updated: Utc::now().timestamp(),
            raw_info,
            average_block_time: None,
            formatted_block_time: None,
            block_time_rating: None,
            circulating_supply: 0,
            mining_progress_percentage: "0".to_string(),
            current_block_reward: 0,
            formatted_block_reward: "0".to_string(),
            current_block_height: 0,
        }
    }

    pub fn new_all_info(
        canister_id: String,
        name: String,
        ticker: String,
        decimals: u8,
        total_supply: u64,
        transfer_fee: u64,
        logo: Option<String>,
        average_block_time: Option<f64>,
        formatted_block_time: Option<String>,
        block_time_rating: Option<String>,
        circulating_supply: u64,
        mining_progress_percentage: String,
        current_block_reward: u64,
        formatted_block_reward: String,
        current_block_height: u64,
        raw_info: String,
    ) -> Self {
        Self {
            canister_id,
            name,
            ticker,
            decimals,
            total_supply,
            transfer_fee,
            logo,
            last_updated: Utc::now().timestamp(),
            raw_info,
            average_block_time,
            formatted_block_time,
            block_time_rating,
            circulating_supply,
            mining_progress_percentage,
            current_block_reward,
            formatted_block_reward,
            current_block_height,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            canister_id: row.get("canister_id")?,
            name: row.get("name")?,
            ticker: row.get("ticker")?,
            decimals: row.get("decimals")?,
            total_supply: row.get("total_supply")?,
            transfer_fee: row.get("transfer_fee")?,
            logo: row.get("logo")?,
            last_updated: row.get("last_updated")?,
            raw_info: row.get("raw_info")?,
            average_block_time: row.get("average_block_time").unwrap_or(None),
            formatted_block_time: row.get("formatted_block_time").unwrap_or(None),
            block_time_rating: row.get("block_time_rating").unwrap_or(None),
            circulating_supply: row.get("circulating_supply").unwrap_or(0),
            mining_progress_percentage: row.get("mining_progress_percentage").unwrap_or_else(|_| "0".to_string()),
            current_block_reward: row.get("current_block_reward").unwrap_or(0),
            formatted_block_reward: row.get("formatted_block_reward").unwrap_or_else(|_| "0".to_string()),
            current_block_height: row.get("current_block_height").unwrap_or(0),
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO token_info (
                canister_id, name, ticker, decimals, total_supply, transfer_fee, logo, last_updated, raw_info,
                average_block_time, formatted_block_time, block_time_rating, circulating_supply,
                mining_progress_percentage, current_block_reward, formatted_block_reward, current_block_height
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
             ON CONFLICT(canister_id) DO UPDATE SET
             name = ?2,
             ticker = ?3,
             decimals = ?4,
             total_supply = ?5,
             transfer_fee = ?6,
             logo = ?7,
             last_updated = ?8,
             raw_info = ?9,
             average_block_time = ?10,
             formatted_block_time = ?11,
             block_time_rating = ?12,
             circulating_supply = ?13,
             mining_progress_percentage = ?14,
             current_block_reward = ?15,
             formatted_block_reward = ?16,
             current_block_height = ?17",
            params![
                self.canister_id,
                self.name,
                self.ticker,
                self.decimals,
                self.total_supply,
                self.transfer_fee,
                self.logo,
                self.last_updated,
                self.raw_info,
                self.average_block_time,
                self.formatted_block_time,
                self.block_time_rating,
                self.circulating_supply,
                self.mining_progress_percentage,
                self.current_block_reward,
                self.formatted_block_reward,
                self.current_block_height,
            ],
        )?;
        Ok(())
    }

    pub fn find_by_canister_id(conn: &Connection, canister_id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT canister_id, name, ticker, decimals, total_supply, transfer_fee, logo, last_updated, raw_info,
             average_block_time, formatted_block_time, block_time_rating, circulating_supply,
             mining_progress_percentage, current_block_reward, formatted_block_reward, current_block_height
             FROM token_info
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
            "SELECT t.canister_id, t.name, t.ticker, t.decimals, t.total_supply, t.transfer_fee, t.logo, t.last_updated, t.raw_info,
             t.average_block_time, t.formatted_block_time, t.block_time_rating, t.circulating_supply,
             t.mining_progress_percentage, t.current_block_reward, t.formatted_block_reward, t.current_block_height
             FROM token_info t
             JOIN canisters c ON t.canister_id = c.canister_id
             ORDER BY t.last_updated DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut tokens = Vec::new();
        for token in rows {
            tokens.push(token?);
        }
        
        Ok(tokens)
    }

    pub fn delete(conn: &Connection, canister_id: &str) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM token_info WHERE canister_id = ?1",
            params![canister_id],
        )?;
        
        Ok(rows_affected > 0)
    }
}
