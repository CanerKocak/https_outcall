use rusqlite::{params, Connection, Result, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum CanisterType {
    Token,
    Miner,
    Wallet,
    Ledger,
}

impl ToString for CanisterType {
    fn to_string(&self) -> String {
        match self {
            CanisterType::Token => "token".to_string(),
            CanisterType::Miner => "miner".to_string(),
            CanisterType::Wallet => "wallet".to_string(),
            CanisterType::Ledger => "ledger".to_string(),
        }
    }
}

impl TryFrom<String> for CanisterType {
    type Error = anyhow::Error;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "token" => Ok(CanisterType::Token),
            "miner" => Ok(CanisterType::Miner),
            "wallet" => Ok(CanisterType::Wallet),
            "ledger" => Ok(CanisterType::Ledger),
            _ => Err(anyhow::anyhow!("Invalid canister type: {}", s)),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Canister {
    pub id: String,
    pub principal: String,
    pub canister_id: String,
    pub canister_type: CanisterType,
    pub module_hash: Option<String>,
    pub created_at: i64,
    pub last_updated: i64,
}

impl Canister {
    pub fn new(
        principal: String,
        canister_id: String,
        canister_type: CanisterType,
        module_hash: Option<String>,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            principal,
            canister_id,
            canister_type,
            module_hash,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        let canister_type_str: String = row.get("type")?;
        let canister_type = CanisterType::try_from(canister_type_str)
            .map_err(|_e| rusqlite::Error::InvalidColumnType(0, "Invalid canister type".to_string(), rusqlite::types::Type::Text))?;

        Ok(Self {
            id: row.get("id")?,
            principal: row.get("principal")?,
            canister_id: row.get("canister_id")?,
            canister_type,
            module_hash: row.get("module_hash")?,
            created_at: row.get("created_at")?,
            last_updated: row.get("last_updated")?,
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO canisters (id, principal, canister_id, type, module_hash, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(canister_id) DO UPDATE SET
             principal = ?2,
             type = ?4,
             module_hash = ?5,
             last_updated = ?7",
            params![
                self.id,
                self.principal,
                self.canister_id,
                self.canister_type.to_string(),
                self.module_hash,
                self.created_at,
                self.last_updated,
            ],
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub fn find_by_id(conn: &Connection, id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, principal, canister_id, type, module_hash, created_at, last_updated
             FROM canisters
             WHERE id = ?1",
        )?;
        
        let mut rows = stmt.query(params![id])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_canister_id(conn: &Connection, canister_id: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, principal, canister_id, type, module_hash, created_at, last_updated
             FROM canisters
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
            "SELECT id, principal, canister_id, type, module_hash, created_at, last_updated
             FROM canisters
             ORDER BY last_updated DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut canisters = Vec::new();
        for canister in rows {
            canisters.push(canister?);
        }
        
        Ok(canisters)
    }

    pub fn find_by_type(conn: &Connection, canister_type: &CanisterType) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, principal, canister_id, type, module_hash, created_at, last_updated
             FROM canisters
             WHERE type = ?1
             ORDER BY last_updated DESC",
        )?;
        
        let rows = stmt.query_map(params![canister_type.to_string()], |row| Self::from_row(row))?;
        
        let mut canisters = Vec::new();
        for canister in rows {
            canisters.push(canister?);
        }
        
        Ok(canisters)
    }

    pub fn delete(conn: &Connection, canister_id: &str) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM canisters WHERE canister_id = ?1",
            params![canister_id],
        )?;
        
        Ok(rows_affected > 0)
    }
} 