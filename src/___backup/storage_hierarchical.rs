// Hierarchical storage with proper meta.project.namespace isolation
// Each keystore table is unique based on hashed prefix

use rusqlite::{params, Connection, OptionalExtension, Result as SqlResult};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::addressing4::Address4;

pub struct HierarchicalStorage {
    pub conn: Connection,
}

impl HierarchicalStorage {
    pub fn new(path: &Path) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        
        // Enable WAL mode for better concurrency
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA temp_store = MEMORY;
             PRAGMA mmap_size = 30000000;"
        )?;
        
        let storage = HierarchicalStorage { conn };
        storage.init_schema()?;
        Ok(storage)
    }

    fn init_schema(&self) -> SqlResult<()> {
        // Meta table - top level
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS meta_registry (
                meta_name TEXT PRIMARY KEY,
                description TEXT,
                created_at INTEGER NOT NULL
            )",
            [],
        )?;

        // Projects table - second level
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS project_registry (
                meta_name TEXT NOT NULL,
                project_name TEXT NOT NULL,
                description TEXT,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (meta_name, project_name),
                FOREIGN KEY (meta_name) REFERENCES meta_registry(meta_name)
            )",
            [],
        )?;

        // Namespace table - third level  
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS namespace_registry (
                meta_name TEXT NOT NULL,
                project_name TEXT NOT NULL,
                namespace_name TEXT NOT NULL,
                table_hash TEXT NOT NULL,  -- Deterministic hash for keystore table name
                is_ttl BOOLEAN NOT NULL DEFAULT 0,
                ttl_seconds INTEGER,
                created_at INTEGER NOT NULL,
                PRIMARY KEY (meta_name, project_name, namespace_name),
                FOREIGN KEY (meta_name, project_name) REFERENCES project_registry(meta_name, project_name)
            )",
            [],
        )?;

        // Create default hierarchy: global.main
        self.ensure_default_hierarchy()?;

        Ok(())
    }

    fn ensure_default_hierarchy(&self) -> SqlResult<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Create default meta
        self.conn.execute(
            "INSERT OR IGNORE INTO meta_registry (meta_name, description, created_at)
             VALUES ('default', 'Default global namespace', ?1)",
            params![now],
        )?;

        // Create main project under default
        self.conn.execute(
            "INSERT OR IGNORE INTO project_registry (meta_name, project_name, description, created_at)
             VALUES ('default', 'main', 'Default main project', ?1)",
            params![now],
        )?;

        // Create default namespace under default.main
        let table_hash = self.compute_table_hash("default", "main", "default");
        self.conn.execute(
            "INSERT OR IGNORE INTO namespace_registry 
             (meta_name, project_name, namespace_name, table_hash, created_at)
             VALUES ('default', 'main', 'default', ?1, ?2)",
            params![table_hash, now],
        )?;

        // Create the actual keystore table for default.main.default
        self.create_keystore_table(&table_hash)?;

        Ok(())
    }

    fn compute_table_hash(&self, meta: &str, project: &str, namespace: &str) -> String {
        let mut hasher = DefaultHasher::new();
        format!("{}.{}.{}", meta, project, namespace).hash(&mut hasher);
        format!("keys_{:x}", hasher.finish())
    }

    fn create_keystore_table(&self, table_hash: &str) -> SqlResult<()> {
        let create_sql = format!(
            "CREATE TABLE IF NOT EXISTS {} (
                key_name TEXT PRIMARY KEY,
                context TEXT,
                value TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                expires_at INTEGER
            )",
            table_hash
        );
        
        self.conn.execute(&create_sql, [])?;
        
        // Create index for context queries
        let index_sql = format!(
            "CREATE INDEX IF NOT EXISTS idx_{}_context ON {}(key_name, context)",
            table_hash, table_hash
        );
        self.conn.execute(&index_sql, [])?;
        
        Ok(())
    }

    fn ensure_hierarchy(&self, meta: &str, project: &str, namespace: &str) -> SqlResult<String> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        // Ensure meta exists
        self.conn.execute(
            "INSERT OR IGNORE INTO meta_registry (meta_name, description, created_at)
             VALUES (?1, ?2, ?3)",
            params![meta, format!("Auto-created meta: {}", meta), now],
        )?;

        // Ensure project exists
        self.conn.execute(
            "INSERT OR IGNORE INTO project_registry (meta_name, project_name, description, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![meta, project, format!("Auto-created project: {}.{}", meta, project), now],
        )?;

        // Get or create namespace with table hash
        let table_hash = self.compute_table_hash(meta, project, namespace);
        
        self.conn.execute(
            "INSERT OR IGNORE INTO namespace_registry 
             (meta_name, project_name, namespace_name, table_hash, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![meta, project, namespace, table_hash, now],
        )?;

        // Create the keystore table if it doesn't exist
        self.create_keystore_table(&table_hash)?;

        Ok(table_hash)
    }

    pub fn set(&self, address: &Address4, value: &str, ttl_seconds: Option<u64>) -> SqlResult<()> {
        // Ensure hierarchy exists and get table hash
        let table_hash = self.ensure_hierarchy(&address.meta, &address.project, &address.namespace)?;
        
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let expires_at = ttl_seconds.map(|ttl| now + ttl as i64);

        // Insert into the specific keystore table
        let insert_sql = format!(
            "INSERT OR REPLACE INTO {} 
             (key_name, context, value, created_at, updated_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            table_hash
        );

        self.conn.execute(
            &insert_sql,
            params![
                &address.key,
                &address.context,
                value,
                now,
                now,
                expires_at,
            ],
        )?;

        Ok(())
    }

    pub fn get(&self, address: &Address4) -> SqlResult<Option<String>> {
        // Get table hash for this namespace
        let table_hash = match self.get_table_hash(&address.meta, &address.project, &address.namespace)? {
            Some(hash) => hash,
            None => return Ok(None), // Namespace doesn't exist
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let query_sql = format!(
            "SELECT value FROM {} 
             WHERE key_name = ?1 
             AND (context = ?2 OR (context IS NULL AND ?2 IS NULL))
             AND (expires_at IS NULL OR expires_at > ?3)",
            table_hash
        );

        let value = self.conn.prepare(&query_sql)?
            .query_row(
                params![&address.key, &address.context, now],
                |row| row.get(0),
            ).optional()?;

        Ok(value)
    }

    pub fn del(&self, address: &Address4) -> SqlResult<bool> {
        // Get table hash for this namespace
        let table_hash = match self.get_table_hash(&address.meta, &address.project, &address.namespace)? {
            Some(hash) => hash,
            None => return Ok(false), // Namespace doesn't exist
        };

        let delete_sql = format!(
            "DELETE FROM {} 
             WHERE key_name = ?1 
             AND (context = ?2 OR (context IS NULL AND ?2 IS NULL))",
            table_hash
        );

        let changes = self.conn.execute(
            &delete_sql,
            params![&address.key, &address.context],
        )?;

        Ok(changes > 0)
    }

    fn get_table_hash(&self, meta: &str, project: &str, namespace: &str) -> SqlResult<Option<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT table_hash FROM namespace_registry 
             WHERE meta_name = ?1 AND project_name = ?2 AND namespace_name = ?3"
        )?;

        let hash = stmt.query_row(
            params![meta, project, namespace],
            |row| row.get(0),
        ).optional()?;

        Ok(hash)
    }

    pub fn list_keys(&self, meta: &str, project: &str, namespace: &str) -> SqlResult<Vec<String>> {
        let table_hash = match self.get_table_hash(meta, project, namespace)? {
            Some(hash) => hash,
            None => return Ok(vec![]),
        };

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        let query_sql = format!(
            "SELECT key_name FROM {} 
             WHERE (expires_at IS NULL OR expires_at > ?1)
             ORDER BY key_name",
            table_hash
        );

        let mut stmt = self.conn.prepare(&query_sql)?;
        let keys = stmt.query_map(params![now], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;

        Ok(keys)
    }

    pub fn list_namespaces(&self, meta: &str, project: &str) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT namespace_name FROM namespace_registry 
             WHERE meta_name = ?1 AND project_name = ?2 
             ORDER BY namespace_name"
        )?;
        
        let namespaces = stmt.query_map(params![meta, project], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        
        Ok(namespaces)
    }

    pub fn list_projects(&self, meta: &str) -> SqlResult<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT project_name FROM project_registry 
             WHERE meta_name = ?1 
             ORDER BY project_name"
        )?;
        
        let projects = stmt.query_map(params![meta], |row| row.get(0))?
            .collect::<Result<Vec<String>, _>>()?;
        
        Ok(projects)
    }

    pub fn list_metas(&self) -> SqlResult<Vec<(String, Option<String>)>> {
        let mut stmt = self.conn.prepare(
            "SELECT meta_name, description FROM meta_registry ORDER BY meta_name"
        )?;

        let metas = stmt.query_map([], |row| {
            Ok((row.get(0)?, row.get(1)?))
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
    fn test_hierarchical_isolation() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = HierarchicalStorage::new(temp_file.path()).unwrap();

        // Same key in different meta.project.namespace should be completely isolated
        let addr1 = Address4::new("keeper", "app", "config", "api_key");
        let addr2 = Address4::new("lucas", "app", "config", "api_key");
        let addr3 = Address4::new("keeper", "tools", "config", "api_key");

        storage.set(&addr1, "keeper_secret", None).unwrap();
        storage.set(&addr2, "lucas_secret", None).unwrap();
        storage.set(&addr3, "keeper_tools_secret", None).unwrap();

        // Each should retrieve only its own value
        assert_eq!(storage.get(&addr1).unwrap(), Some("keeper_secret".to_string()));
        assert_eq!(storage.get(&addr2).unwrap(), Some("lucas_secret".to_string()));
        assert_eq!(storage.get(&addr3).unwrap(), Some("keeper_tools_secret".to_string()));
    }

    #[test]
    fn test_table_hash_deterministic() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = HierarchicalStorage::new(temp_file.path()).unwrap();

        let hash1 = storage.compute_table_hash("keeper", "app", "config");
        let hash2 = storage.compute_table_hash("keeper", "app", "config");
        let hash3 = storage.compute_table_hash("lucas", "app", "config");

        // Same inputs produce same hash
        assert_eq!(hash1, hash2);
        // Different inputs produce different hash
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_auto_hierarchy_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let storage = HierarchicalStorage::new(temp_file.path()).unwrap();

        // Using a completely new hierarchy should auto-create it
        let addr = Address4::new("pantheon", "divine", "secrets", "keeper_power");
        storage.set(&addr, "unlimited", None).unwrap();

        // Should be able to retrieve it
        assert_eq!(storage.get(&addr).unwrap(), Some("unlimited".to_string()));

        // Verify hierarchy was created
        let metas = storage.list_metas().unwrap();
        assert!(metas.iter().any(|(name, _)| name == "pantheon"));

        let projects = storage.list_projects("pantheon").unwrap();
        assert!(projects.contains(&"divine".to_string()));

        let namespaces = storage.list_namespaces("pantheon", "divine").unwrap();
        assert!(namespaces.contains(&"secrets".to_string()));
    }
}