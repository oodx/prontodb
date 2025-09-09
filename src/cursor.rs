// Cursor Management for ProntoDB
// Provides database context switching and multi-user support following RSB patterns

use rsb::prelude::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::xdg::XdgPaths;

/// Cursor file format for database context storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CursorData {
    pub database_path: PathBuf,
    pub default_project: Option<String>,
    pub default_namespace: Option<String>,
    pub created_at: String,
    pub user: String,
}

impl CursorData {
    /// Create new cursor data
    pub fn new(database_path: PathBuf, user: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let created_at = chrono::DateTime::from_timestamp(created_at as i64, 0)
            .unwrap()
            .format("%Y-%m-%dT%H:%M:%SZ")
            .to_string();

        Self {
            database_path,
            default_project: None,
            default_namespace: None,
            created_at,
            user,
        }
    }

    /// Set default project for this cursor
    pub fn with_project(mut self, project: String) -> Self {
        self.default_project = Some(project);
        self
    }

    /// Set default namespace for this cursor
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.default_namespace = Some(namespace);
        self
    }
}

/// Cursor Manager handles cursor creation, listing, and management
#[derive(Debug)]
pub struct CursorManager {
    xdg: XdgPaths,
    cursor_dir: PathBuf,
}

impl CursorManager {
    /// Create new cursor manager
    pub fn new() -> Self {
        let xdg = XdgPaths::new();
        let cursor_dir = xdg.data_dir.join("cursors");
        
        // RSB directory validation
        fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory");
        
        Self { xdg, cursor_dir }
    }

    /// Create cursor manager from specific XDG paths (for testing)
    pub fn from_xdg(xdg: XdgPaths) -> Self {
        let cursor_dir = xdg.data_dir.join("cursors");
        fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory for testing");
        Self { xdg, cursor_dir }
    }

    /// Set a cursor to point to a specific database path
    pub fn set_cursor(&self, name: &str, database_path: PathBuf, user: &str) {
        let cursor_data = CursorData::new(database_path, user.to_string());
        let cursor_file = self.get_cursor_file_path(name, user);
        
        let json_content = serde_json::to_string_pretty(&cursor_data)
            .expect("Failed to serialize cursor data");
        fs::write(&cursor_file, json_content)
            .expect("Failed to write cursor file");
    }

    /// Set a cursor with project and namespace defaults
    pub fn set_cursor_with_defaults(
        &self,
        name: &str,
        database_path: PathBuf,
        user: &str,
        project: Option<String>,
        namespace: Option<String>,
    ) {
        let mut cursor_data = CursorData::new(database_path, user.to_string());
        cursor_data.default_project = project;
        cursor_data.default_namespace = namespace;
        
        let cursor_file = self.get_cursor_file_path(name, user);
        let json_content = serde_json::to_string_pretty(&cursor_data)
            .expect("Failed to serialize cursor data with defaults");
        fs::write(&cursor_file, json_content)
            .expect("Failed to write cursor file with defaults");
    }

    /// Get cursor data for a named cursor
    pub fn get_cursor(&self, name: &str, user: &str) -> Result<CursorData, Box<dyn std::error::Error>> {
        let cursor_file = self.get_cursor_file_path(name, user);
        
        if !cursor_file.exists() {
            return Err(format!("Cursor '{}' not found for user '{}'", name, user).into());
        }
        
        let content = fs::read_to_string(&cursor_file)?;
        let cursor_data: CursorData = serde_json::from_str(&content)?;
        
        Ok(cursor_data)
    }

    /// Get the active cursor (default cursor for user)
    pub fn get_active_cursor(&self, user: &str) -> Result<Option<CursorData>, Box<dyn std::error::Error>> {
        match self.get_cursor("default", user) {
            Ok(cursor) => Ok(Some(cursor)),
            Err(_) => Ok(None),
        }
    }

    /// List all cursors for a user
    pub fn list_cursors(&self, user: &str) -> Result<HashMap<String, CursorData>, Box<dyn std::error::Error>> {
        let mut cursors = HashMap::new();
        let user_suffix = if user == "default" { ".cursor".to_string() } else { format!(".{}.cursor", user) };
        
        if !self.cursor_dir.exists() {
            return Ok(cursors);
        }
        
        for entry in fs::read_dir(&self.cursor_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(&user_suffix) {
                    let cursor_name = if user == "default" {
                        filename.strip_suffix(".cursor").unwrap_or(filename)
                    } else {
                        filename.strip_suffix(&format!(".{}.cursor", user)).unwrap_or(filename)
                    };
                    
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cursor_data) = serde_json::from_str::<CursorData>(&content) {
                            cursors.insert(cursor_name.to_string(), cursor_data);
                        }
                    }
                }
            }
        }
        
        Ok(cursors)
    }

    /// List all cursors across all users (for admin purposes)
    pub fn list_all_cursors(&self) -> Result<HashMap<String, CursorData>, Box<dyn std::error::Error>> {
        let mut cursors = HashMap::new();
        
        if !self.cursor_dir.exists() {
            return Ok(cursors);
        }
        
        for entry in fs::read_dir(&self.cursor_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                if filename.ends_with(".cursor") {
                    let cursor_key = filename.strip_suffix(".cursor").unwrap_or(filename);
                    
                    if let Ok(content) = fs::read_to_string(&path) {
                        if let Ok(cursor_data) = serde_json::from_str::<CursorData>(&content) {
                            cursors.insert(cursor_key.to_string(), cursor_data);
                        }
                    }
                }
            }
        }
        
        Ok(cursors)
    }

    /// Delete a cursor
    pub fn delete_cursor(&self, name: &str, user: &str) -> Result<bool, Box<dyn std::error::Error>> {
        let cursor_file = self.get_cursor_file_path(name, user);
        
        if cursor_file.exists() {
            fs::remove_file(&cursor_file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolve database path from cursor name and user
    /// Returns the cursor's database path if found, or None to use default
    pub fn resolve_database_path(&self, cursor_name: Option<&str>, user: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let name = cursor_name.unwrap_or("default");
        
        match self.get_cursor(name, user) {
            Ok(cursor) => Ok(Some(cursor.database_path)),
            Err(_) => Ok(None),
        }
    }

    /// Ensure default cursor exists for a user
    pub fn ensure_default_cursor(&self, user: &str) -> Result<(), Box<dyn std::error::Error>> {
        let default_cursor_file = self.get_cursor_file_path("default", user);
        
        if !default_cursor_file.exists() {
            // Create default cursor pointing to the XDG database path
            // Always use the direct db_path for consistent behavior in tests
            let default_db_path = self.xdg.db_path.clone();
            self.set_cursor("default", default_db_path, user);
        }
        
        Ok(())
    }

    /// Get cursor file path for name and user
    fn get_cursor_file_path(&self, name: &str, user: &str) -> PathBuf {
        if user == "default" {
            self.cursor_dir.join(format!("{}.cursor", name))
        } else {
            self.cursor_dir.join(format!("{}.{}.cursor", name, user))
        }
    }
}

impl Default for CursorManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::xdg::test_utils::TestXdg;
    use tempfile::TempDir;

    #[test]
    fn test_cursor_data_creation() {
        let db_path = PathBuf::from("/test/db.sqlite");
        let cursor = CursorData::new(db_path.clone(), "testuser".to_string());
        
        assert_eq!(cursor.database_path, db_path);
        assert_eq!(cursor.user, "testuser");
        assert!(cursor.default_project.is_none());
        assert!(cursor.default_namespace.is_none());
        assert!(!cursor.created_at.is_empty());
    }

    #[test]
    fn test_cursor_data_with_defaults() {
        let db_path = PathBuf::from("/test/db.sqlite");
        let cursor = CursorData::new(db_path, "testuser".to_string())
            .with_project("myproject".to_string())
            .with_namespace("config".to_string());
        
        assert_eq!(cursor.default_project, Some("myproject".to_string()));
        assert_eq!(cursor.default_namespace, Some("config".to_string()));
    }

    #[test]
    fn test_cursor_manager_creation() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        assert!(cursor_manager.cursor_dir.exists());
    }

    #[test]
    fn test_set_and_get_cursor() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        let db_path = PathBuf::from("/test/custom.db");
        cursor_manager.set_cursor("test", db_path.clone(), "alice");
        
        let cursor = cursor_manager.get_cursor("test", "alice").unwrap();
        assert_eq!(cursor.database_path, db_path);
        assert_eq!(cursor.user, "alice");
    }

    #[test]
    fn test_cursor_file_naming() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        // Default user gets simple naming
        let default_path = cursor_manager.get_cursor_file_path("test", "default");
        assert!(default_path.to_str().unwrap().ends_with("test.cursor"));
        
        // Named users get user suffix
        let alice_path = cursor_manager.get_cursor_file_path("test", "alice");
        assert!(alice_path.to_str().unwrap().ends_with("test.alice.cursor"));
    }

    #[test]
    fn test_list_cursors() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        // Create test cursors
        cursor_manager.set_cursor("default", PathBuf::from("/test1.db"), "alice");
        cursor_manager.set_cursor("staging", PathBuf::from("/test2.db"), "alice");
        cursor_manager.set_cursor("default", PathBuf::from("/test3.db"), "bob");
        
        let alice_cursors = cursor_manager.list_cursors("alice").unwrap();
        assert_eq!(alice_cursors.len(), 2);
        assert!(alice_cursors.contains_key("default"));
        assert!(alice_cursors.contains_key("staging"));
        
        let bob_cursors = cursor_manager.list_cursors("bob").unwrap();
        assert_eq!(bob_cursors.len(), 1);
        assert!(bob_cursors.contains_key("default"));
    }

    #[test]
    fn test_resolve_database_path() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        let custom_db = PathBuf::from("/custom/path.db");
        cursor_manager.set_cursor("prod", custom_db.clone(), "alice");
        
        // Resolve existing cursor
        let resolved = cursor_manager.resolve_database_path(Some("prod"), "alice").unwrap();
        assert_eq!(resolved, Some(custom_db));
        
        // Resolve non-existing cursor
        let resolved = cursor_manager.resolve_database_path(Some("nonexistent"), "alice").unwrap();
        assert_eq!(resolved, None);
    }

    #[test]
    fn test_ensure_default_cursor() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        cursor_manager.ensure_default_cursor("alice").unwrap();
        
        let cursor = cursor_manager.get_cursor("default", "alice").unwrap();
        // The path should be the test XDG path without any environment override
        let expected_path = test_xdg.paths.db_path.clone(); // Use direct field, not get_db_path()
        assert_eq!(cursor.database_path, expected_path);
        assert_eq!(cursor.user, "alice");
    }

    #[test]
    fn test_delete_cursor() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths);
        
        cursor_manager.set_cursor("temp", PathBuf::from("/temp.db"), "alice");
        
        // Verify it exists
        assert!(cursor_manager.get_cursor("temp", "alice").is_ok());
        
        // Delete it
        let deleted = cursor_manager.delete_cursor("temp", "alice").unwrap();
        assert!(deleted);
        
        // Verify it's gone
        assert!(cursor_manager.get_cursor("temp", "alice").is_err());
        
        // Try to delete again
        let deleted = cursor_manager.delete_cursor("temp", "alice").unwrap();
        assert!(!deleted);
    }
}