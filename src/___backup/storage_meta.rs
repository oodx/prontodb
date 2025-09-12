// Meta namespace storage - maps meta contexts to projects
// Provides transparent routing without changing the core KV structure

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::addressing::Address;
use crate::addressing4::Address4;

pub struct MetaStorage {
    pub conn: Connection,
}

impl MetaStorage {
    pub fn new(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA mmap_size = 30000000;"
        )?;
        
        let storage = MetaStorage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> SqlResult<()> {
        // Meta mapping table - maps meta namespaces to actual projects
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS meta_mappings (
                meta TEXT NOT NULL,
                mapped_project TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                PRIMARY KEY (meta)
            )",
            [],
        )?;

        // Meta routing table - determines which project a meta+project combo maps to
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS meta_routes (
                meta TEXT NOT NULL,
                virtual_project TEXT NOT NULL,
                actual_project TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (meta, virtual_project)
            )",
            [],
        )?;

        // Original KV table remains unchanged - 3 layer addressing
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

        // Index for efficient lookups
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_kv_project_namespace 
             ON kv(project, namespace)",
            [],
        )?;

        Ok(())
    }

    /// Register a meta namespace mapping
    pub fn register_meta(&self, meta: &str, mapped_project: &str, description: Option<&str>) -> SqlResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn.execute(
            "INSERT OR REPLACE INTO meta_mappings 
             (meta, mapped_project, description, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![meta, mapped_project, description, now, now],
        )?;

        Ok(())
    }

    /// Get the actual project name for a meta namespace
    pub fn resolve_meta(&self, meta: &str) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT mapped_project FROM meta_mappings WHERE meta = ?1"
        )?;

        let project = stmt.query_row(params![meta], |row| row.get(0)).optional()?;
        Ok(project)
    }

    /// Convert a 4-layer address to 3-layer by resolving meta
    pub fn resolve_address(&self, addr4: &Address4) -> SqlResult<Address> {
        // First check if there's a specific route for meta+project
        let mut stmt = self.conn.prepare(
            "SELECT actual_project FROM meta_routes 
             WHERE meta = ?1 AND virtual_project = ?2"
        )?;

        let routed_project = stmt.query_row(
            params![&addr4.meta, &addr4.project],
            |row| row.get::<_, String>(0)
        ).optional()?;

        let actual_project = if let Some(routed) = routed_project {
            // Use specific route
            routed
        } else if let Some(mapped) = self.resolve_meta(&addr4.meta)? {
            // Use default meta mapping with project suffix
            format!("{}_{}", mapped, addr4.project)
        } else if &addr4.meta == "default" {
            // No meta mapping, use project as-is
            addr4.project.clone()
        } else {
            // Unknown meta, create isolated project
            format!("meta_{}_{}", addr4.meta, addr4.project)
        };

        Ok(Address {
            project: actual_project,
            namespace: addr4.namespace.clone(),
            key: addr4.key.clone(),
            context: addr4.context.clone(),
        })
    }

    /// Store using 4-layer address (converts to 3-layer internally)
    pub fn set(&self, addr4: &Address4, value: &str, ttl_seconds: Option<u64>) -> SqlResult<()> {
        let addr3 = self.resolve_address(addr4)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expires_at = ttl_seconds.map(|ttl| now + ttl as i64);

        self.conn.execute(
            "INSERT OR REPLACE INTO kv 
             (project, namespace, key, context, value, created_at, updated_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                &addr3.project,
                &addr3.namespace,
                &addr3.key,
                &addr3.context,
                value,
                now,
                now,
                expires_at,
            ],
        )?;

        Ok(())
    }

    /// Get using 4-layer address
    pub fn get(&self, addr4: &Address4) -> SqlResult<Option<String>> {
        let addr3 = self.resolve_address(addr4)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let mut stmt = self.conn.prepare(
            "SELECT value FROM kv 
             WHERE project = ?1 AND namespace = ?2 AND key = ?3 
             AND (context = ?4 OR (context IS NULL AND ?4 IS NULL))
             AND (expires_at IS NULL OR expires_at > ?5)"
        )?;

        let value = stmt.query_row(
            params![
                &addr3.project,
                &addr3.namespace,
                &addr3.key,
                &addr3.context,
                now,
            ],
            |row| row.get(0),
        ).optional()?;

        Ok(value)
    }

    /// Delete using 4-layer address
    pub fn del(&self, addr4: &Address4) -> SqlResult<bool> {
        let addr3 = self.resolve_address(addr4)?;
        
        let changes = self.conn.execute(
            "DELETE FROM kv 
             WHERE project = ?1 AND namespace = ?2 AND key = ?3 
             AND (context = ?4 OR (context IS NULL AND ?4 IS NULL))",
            params![
                &addr3.project,
                &addr3.namespace,
                &addr3.key,
                &addr3.context,
            ],
        )?;

        Ok(changes > 0)
    }

    /// List all registered meta namespaces
    pub fn list_metas(&self) -> SqlResult<Vec<(String, String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT meta, mapped_project, description 
             FROM meta_mappings 
             ORDER BY meta"
        )?;

        let metas = stmt.query_map([], |row| {
            Ok((
                row.get(0)?,
                row.get(1)?,
                row.get(2)?,
            ))
        })?
        .collect::<Result<Vec<_>, _>>()?;

        Ok(metas)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_meta_registration() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = MetaStorage::new(temp_file.path()).unwrap();

        // Register meta namespaces for pantheon users
        storage.register_meta("keeper", "pantheon_keeper", Some("Keeper's divine realm")).unwrap();
        storage.register_meta("lucas", "pantheon_lucas", Some("Lucas's engineering space")).unwrap();

        // Verify registration
        assert_eq!(storage.resolve_meta("keeper").unwrap(), Some("pantheon_keeper".to_string()));
        assert_eq!(storage.resolve_meta("lucas").unwrap(), Some("pantheon_lucas".to_string()));
    }

    #[test]
    fn test_meta_isolation() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = MetaStorage::new(temp_file.path()).unwrap();

        // Register users
        storage.register_meta("keeper", "pantheon_keeper", None).unwrap();
        storage.register_meta("lucas", "pantheon_lucas", None).unwrap();

        // Same logical address, different meta
        let keeper_addr = Address4::new("keeper", "config", "settings", "api_key");
        let lucas_addr = Address4::new("lucas", "config", "settings", "api_key");

        // Store different values
        storage.set(&keeper_addr, "keeper_secret", None).unwrap();
        storage.set(&lucas_addr, "lucas_secret", None).unwrap();

        // Each user sees only their data
        assert_eq!(storage.get(&keeper_addr).unwrap(), Some("keeper_secret".to_string()));
        assert_eq!(storage.get(&lucas_addr).unwrap(), Some("lucas_secret".to_string()));
    }

    #[test]
    fn test_address_resolution() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = MetaStorage::new(temp_file.path()).unwrap();

        storage.register_meta("keeper", "pantheon_keeper", None).unwrap();

        let addr4 = Address4::new("keeper", "myapp", "config", "setting");
        let addr3 = storage.resolve_address(&addr4).unwrap();

        // Meta "keeper" + project "myapp" becomes "pantheon_keeper_myapp"
        assert_eq!(addr3.project, "pantheon_keeper_myapp");
        assert_eq!(addr3.namespace, "config");
        assert_eq!(addr3.key, "setting");
    }
}