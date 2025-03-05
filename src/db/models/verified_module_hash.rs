use rusqlite::{params, Connection, Result, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VerifiedModuleHash {
    pub id: String,
    pub hash: String,
    pub description: String,
    pub canister_type: String,
    pub is_active: bool,
    pub created_at: i64,
    pub last_updated: i64,
}

impl VerifiedModuleHash {
    pub fn new(
        hash: String,
        description: String,
        canister_type: String,
    ) -> Self {
        let now = Utc::now().timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            hash,
            description,
            canister_type,
            is_active: true,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            hash: row.get("hash")?,
            description: row.get("description")?,
            canister_type: row.get("canister_type")?,
            is_active: row.get::<_, i64>("is_active")? != 0,
            created_at: row.get("created_at")?,
            last_updated: row.get("last_updated")?,
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO verified_module_hashes (id, hash, description, canister_type, is_active, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(hash) DO UPDATE SET
             description = ?3,
             canister_type = ?4,
             is_active = ?5,
             last_updated = ?7",
            params![
                self.id,
                self.hash,
                self.description,
                self.canister_type,
                self.is_active as i64,
                self.created_at,
                self.last_updated,
            ],
        )?;
        Ok(())
    }

    pub fn find_all(conn: &Connection) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, hash, description, canister_type, is_active, created_at, last_updated
             FROM verified_module_hashes
             ORDER BY last_updated DESC",
        )?;
        
        let rows = stmt.query_map([], |row| Self::from_row(row))?;
        
        let mut hashes = Vec::new();
        for hash in rows {
            hashes.push(hash?);
        }
        
        Ok(hashes)
    }

    pub fn find_by_hash(conn: &Connection, hash: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, hash, description, canister_type, is_active, created_at, last_updated
             FROM verified_module_hashes
             WHERE hash = ?1",
        )?;
        
        let mut rows = stmt.query(params![hash])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_canister_type(conn: &Connection, canister_type: &str) -> Result<Vec<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, hash, description, canister_type, is_active, created_at, last_updated
             FROM verified_module_hashes
             WHERE canister_type = ?1 AND is_active = 1
             ORDER BY last_updated DESC",
        )?;
        
        let rows = stmt.query_map(params![canister_type], |row| Self::from_row(row))?;
        
        let mut hashes = Vec::new();
        for hash in rows {
            hashes.push(hash?);
        }
        
        Ok(hashes)
    }

    pub fn is_hash_verified(conn: &Connection, hash: &str, canister_type: &str) -> Result<bool> {
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM verified_module_hashes
             WHERE hash = ?1 AND canister_type = ?2 AND is_active = 1",
            params![hash, canister_type],
            |row| row.get(0),
        )?;
        
        Ok(count > 0)
    }

    pub fn delete(conn: &Connection, hash: &str) -> Result<bool> {
        let rows_affected = conn.execute(
            "DELETE FROM verified_module_hashes WHERE hash = ?1",
            params![hash],
        )?;
        
        Ok(rows_affected > 0)
    }
} 