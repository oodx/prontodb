// Storage module for SQLite database operations
// Handles all database interactions for ProntoDB

#![allow(dead_code)]  // Some functions are used via pub api

use rusqlite::{Connection, Result, params};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use crate::addressing::Address;

pub struct Storage {
    conn: Connection,
}

impl Storage {
    // Open or create database at given path
    pub fn open(db_path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                rusqlite::Error::SqliteFailure(
                    rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
                    Some(format!("Failed to create directory: {}", e)),
                )
            })?;
        }

        let conn = Connection::open(db_path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch("PRAGMA journal_mode = WAL;")?;
        
        // Set busy timeout (5 seconds default)
        conn.execute_batch("PRAGMA busy_timeout = 5000;")?;

        let storage = Storage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    // Initialize database schema
    fn init_schema(&self) -> Result<()> {
        // Main key-value table
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS kv (
                project TEXT NOT NULL,
                namespace TEXT NOT NULL,
                key TEXT NOT NULL,
                context TEXT,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER,
                PRIMARY KEY (project, namespace, key, context)
            )",
            [],
        )?;

        // System namespaces table (for TTL namespaces)
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS sys_namespaces (
                project TEXT NOT NULL,
                namespace TEXT NOT NULL,
                is_ttl BOOLEAN NOT NULL DEFAULT 0,
                default_ttl INTEGER,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (project, namespace)
            )",
            [],
        )?;

        // Indexes for efficient queries
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_expiry ON kv(expires_at) 
             WHERE expires_at IS NOT NULL",
            [],
        )?;

        Ok(())
    }

    // Set a value
    pub fn set(&self, addr: &Address, value: &str, ttl: Option<u64>) -> Result<()> {
        let now = current_timestamp();
        let expires_at = ttl.map(|t| now + t as i64);

        self.conn.execute(
            "INSERT OR REPLACE INTO kv 
             (project, namespace, key, context, value, created_at, updated_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6, ?7)",
            params![
                addr.project,
                addr.namespace,
                addr.key,
                addr.context,
                value,
                now,
                expires_at
            ],
        )?;

        Ok(())
    }

    // Get a value
    pub fn get(&self, addr: &Address) -> Result<Option<String>> {
        let now = current_timestamp();
        
        let mut stmt = self.conn.prepare(
            "SELECT value, expires_at FROM kv 
             WHERE project = ?1 AND namespace = ?2 AND key = ?3 
             AND (context = ?4 OR (?4 IS NULL AND context IS NULL))"
        )?;

        let result: Result<(String, Option<i64>)> = stmt.query_row(
            params![addr.project, addr.namespace, addr.key, addr.context],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );

        match result {
            Ok((value, expires_at)) => {
                // Check if expired
                if let Some(exp) = expires_at {
                    if exp <= now {
                        // Expired - optionally delete it
                        self.delete(addr)?;
                        Ok(None)
                    } else {
                        Ok(Some(value))
                    }
                } else {
                    Ok(Some(value))
                }
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // Delete a value
    pub fn delete(&self, addr: &Address) -> Result<()> {
        self.conn.execute(
            "DELETE FROM kv 
             WHERE project = ?1 AND namespace = ?2 AND key = ?3 
             AND (context = ?4 OR (?4 IS NULL AND context IS NULL))",
            params![addr.project, addr.namespace, addr.key, addr.context],
        )?;
        Ok(())
    }

    // List keys with optional prefix
    pub fn list_keys(
        &self,
        project: &str,
        namespace: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<String>> {
        let now = current_timestamp();
        
        let query = if let Some(prefix) = prefix {
            format!(
                "SELECT key FROM kv 
                 WHERE project = ?1 AND namespace = ?2 
                 AND key LIKE '{}%'
                 AND (expires_at IS NULL OR expires_at > ?3)
                 ORDER BY key",
                prefix.replace("'", "''")
            )
        } else {
            "SELECT key FROM kv 
             WHERE project = ?1 AND namespace = ?2 
             AND (expires_at IS NULL OR expires_at > ?3)
             ORDER BY key".to_string()
        };

        let mut stmt = self.conn.prepare(&query)?;
        let keys = stmt.query_map(params![project, namespace, now], |row| {
            row.get(0)
        })?;

        keys.collect()
    }

    // Scan key-value pairs
    pub fn scan(
        &self,
        project: &str,
        namespace: &str,
        prefix: Option<&str>,
    ) -> Result<Vec<(String, String)>> {
        let now = current_timestamp();
        
        let query = if let Some(prefix) = prefix {
            format!(
                "SELECT key, value FROM kv 
                 WHERE project = ?1 AND namespace = ?2 
                 AND key LIKE '{}%'
                 AND (expires_at IS NULL OR expires_at > ?3)
                 ORDER BY key",
                prefix.replace("'", "''")
            )
        } else {
            "SELECT key, value FROM kv 
             WHERE project = ?1 AND namespace = ?2 
             AND (expires_at IS NULL OR expires_at > ?3)
             ORDER BY key".to_string()
        };

        let mut stmt = self.conn.prepare(&query)?;
        let pairs = stmt.query_map(params![project, namespace, now], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        pairs.collect()
    }

    // Create a TTL namespace
    pub fn create_ttl_namespace(
        &self,
        project: &str,
        namespace: &str,
        default_ttl: u64,
    ) -> Result<()> {
        let now = current_timestamp();
        
        self.conn.execute(
            "INSERT OR REPLACE INTO sys_namespaces 
             (project, namespace, is_ttl, default_ttl, created_at)
             VALUES (?1, ?2, 1, ?3, ?4)",
            params![project, namespace, default_ttl, now],
        )?;

        Ok(())
    }

    // Check if namespace is TTL-enabled and get default TTL
    pub fn get_namespace_ttl(&self, project: &str, namespace: &str) -> Result<Option<u64>> {
        let mut stmt = self.conn.prepare(
            "SELECT default_ttl FROM sys_namespaces 
             WHERE project = ?1 AND namespace = ?2 AND is_ttl = 1"
        )?;

        match stmt.query_row(params![project, namespace], |row| {
            row.get::<_, i64>(0)
        }) {
            Ok(ttl) => Ok(Some(ttl as u64)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e),
        }
    }

    // List all projects
    pub fn list_projects(&self) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT project FROM kv ORDER BY project"
        )?;

        let projects = stmt.query_map([], |row| row.get(0))?;
        projects.collect()
    }

    // List namespaces for a project
    pub fn list_namespaces(&self, project: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT namespace FROM kv 
             WHERE project = ?1 
             ORDER BY namespace"
        )?;

        let namespaces = stmt.query_map(params![project], |row| row.get(0))?;
        namespaces.collect()
    }
}

// Helper to get current Unix timestamp
fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

// Helper to get database path from environment or default
pub fn get_db_path() -> PathBuf {
    if let Ok(path) = std::env::var("PRONTO_DB") {
        PathBuf::from(path)
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home)
            .join(".local")
            .join("data")
            .join("odx")
            .join("prontodb")
            .join("pronto.db")
    } else {
        PathBuf::from("/tmp/pronto.db")
    }
}