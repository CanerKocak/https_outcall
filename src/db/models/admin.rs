use rusqlite::{params, Connection, Result, Row};
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::Utc;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use std::fmt;

// Custom error type to handle argon2 password hash errors
#[derive(Debug)]
pub struct PasswordError(String);

impl fmt::Display for PasswordError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Password error: {}", self.0)
    }
}

impl std::error::Error for PasswordError {}

impl From<argon2::password_hash::Error> for PasswordError {
    fn from(err: argon2::password_hash::Error) -> Self {
        PasswordError(err.to_string())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Admin {
    pub id: String,
    pub username: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub api_key: String,
    pub is_active: bool,
    pub created_at: i64,
    pub last_updated: i64,
}

impl Admin {
    pub fn new(
        username: String,
        password: &str,
    ) -> Result<Self, anyhow::Error> {
        let now = Utc::now().timestamp();
        let id = Uuid::new_v4().to_string();
        
        // Generate API key
        let api_key = Uuid::new_v4().to_string();
        
        // Hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = match argon2.hash_password(password.as_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(e) => return Err(PasswordError(e.to_string()).into()),
        };
        
        Ok(Self {
            id,
            username,
            password_hash,
            api_key,
            is_active: true,
            created_at: now,
            last_updated: now,
        })
    }

    pub fn from_row(row: &Row) -> Result<Self> {
        Ok(Self {
            id: row.get("id")?,
            username: row.get("username")?,
            password_hash: row.get("password_hash")?,
            api_key: row.get("api_key")?,
            is_active: row.get::<_, i64>("is_active")? != 0,
            created_at: row.get("created_at")?,
            last_updated: row.get("last_updated")?,
        })
    }

    pub fn save(&self, conn: &Connection) -> Result<()> {
        conn.execute(
            "INSERT INTO admins (id, username, password_hash, api_key, is_active, created_at, last_updated)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
             ON CONFLICT(username) DO UPDATE SET
             password_hash = ?3,
             api_key = ?4,
             is_active = ?5,
             last_updated = ?7",
            params![
                self.id,
                self.username,
                self.password_hash,
                self.api_key,
                self.is_active as i64,
                self.created_at,
                self.last_updated,
            ],
        )?;
        Ok(())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, anyhow::Error> {
        let parsed_hash = match PasswordHash::new(&self.password_hash) {
            Ok(hash) => hash,
            Err(e) => return Err(PasswordError(e.to_string()).into()),
        };
        
        Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    pub fn find_by_username(conn: &Connection, username: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, api_key, is_active, created_at, last_updated
             FROM admins
             WHERE username = ?1",
        )?;
        
        let mut rows = stmt.query(params![username])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn find_by_api_key(conn: &Connection, api_key: &str) -> Result<Option<Self>> {
        let mut stmt = conn.prepare(
            "SELECT id, username, password_hash, api_key, is_active, created_at, last_updated
             FROM admins
             WHERE api_key = ?1 AND is_active = 1",
        )?;
        
        let mut rows = stmt.query(params![api_key])?;
        
        if let Some(row) = rows.next()? {
            Ok(Some(Self::from_row(row)?))
        } else {
            Ok(None)
        }
    }

    pub fn create_admin_if_none_exists(conn: &Connection, username: &str, password: &str) -> Result<(), anyhow::Error> {
        // Check if any admin exists
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM admins",
            [],
            |row| row.get(0),
        )?;
        
        if count == 0 {
            // No admins exist, create the default one
            let admin = Self::new(username.to_string(), password)?;
            admin.save(conn)?;
        }
        
        Ok(())
    }
} 