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
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO token_info (canister_id, name, ticker, decimals, total_supply, transfer_fee, logo, last_updated, raw_info)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
             ON CONFLICT(canister_id) DO UPDATE SET
             name = ?2,
             ticker = ?3,
             decimals = ?4,
             total_supply = ?5,
             transfer_fee = ?6,
             logo = ?7,
             last_updated = ?8,
             raw_info = ?9",
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
            ],
        )?;
        Ok(())
    }

    pub fn find_by_canister_id(conn: &Connection, canister_id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT canister_id, name, ticker, decimals, total_supply, transfer_fee, logo, last_updated, raw_info
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
            "SELECT t.canister_id, t.name, t.ticker, t.decimals, t.total_supply, t.transfer_fee, t.logo, t.last_updated, t.raw_info
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