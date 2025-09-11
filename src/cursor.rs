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
    pub meta_context: Option<String>,
    pub created_at: String,
    pub user: String,
}

impl CursorData {
    #[allow(dead_code)]  // Methods for future cursor features
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
            meta_context: None,
            created_at,
            user,
        }
    }

    /// Set default project for this cursor
    #[allow(dead_code)]
    pub fn with_project(mut self, project: String) -> Self {
        self.default_project = Some(project);
        self
    }

    /// Set default namespace for this cursor
    #[allow(dead_code)]
    pub fn with_namespace(mut self, namespace: String) -> Self {
        self.default_namespace = Some(namespace);
        self
    }

    /// Set meta context for this cursor
    #[allow(dead_code)]
    pub fn with_meta_context(mut self, meta_context: String) -> Self {
        self.meta_context = Some(meta_context);
        self
    }
}

/// Cursor Manager handles cursor creation, listing, and management
#[derive(Debug)]
pub struct CursorManager {
    xdg: XdgPaths,
    cursor_dir: PathBuf,  // Legacy cursor directory for backward compatibility
}

impl CursorManager {
    /// Extract database name from a database path for directory organization
    /// This determines which database-scoped directory cursors should live in
    fn extract_database_name(&self, db_path: &PathBuf) -> String {
        // Strategy: Use the filename without extension as the database name
        // For paths like /path/to/mydb.db -> "mydb"
        // For paths like /path/to/custom.sqlite -> "custom"  
        // For paths ending in standard main locations -> "main"
        
        if let Some(filename) = db_path.file_stem().and_then(|s| s.to_str()) {
            // Special case: if this looks like a default XDG path, use "main"
            if *db_path == self.xdg.get_db_path() || db_path.to_string_lossy().contains("/prontodb/main/pronto.main.prdb") {
                return "main".to_string();
            }
            
            filename.to_string()
        } else {
            // Fallback for edge cases where we can't extract a name
            "main".to_string()
        }
    }
    /// Create new cursor manager
    pub fn new() -> Self {
        let xdg = XdgPaths::new();
        let cursor_dir = xdg.data_dir.join("cursors");  // Legacy directory
        
        // RSB directory validation for legacy compatibility
        fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory");
        
        Self { xdg, cursor_dir }
    }

    /// Create cursor manager from specific XDG paths (for testing)
    #[allow(dead_code)]
    pub fn from_xdg(xdg: XdgPaths) -> Self {
        let cursor_dir = xdg.data_dir.join("cursors");
        fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory for testing");
        Self { xdg, cursor_dir }
    }

    /// Set a cursor to point to a specific database path
    pub fn set_cursor(&self, name: &str, database_path: PathBuf, user: &str) {
        let cursor_data = CursorData::new(database_path.clone(), user.to_string());
        
        // Determine database name and use database-scoped storage
        let db_name = self.extract_database_name(&database_path);
        let cursor_file = self.get_cursor_file_path_scoped(name, user, &db_name);
        
        let json_content = serde_json::to_string_pretty(&cursor_data)
            .expect("Failed to serialize cursor data");
        fs::write(&cursor_file, json_content)
            .expect("Failed to write cursor file");
    }

    /// Set a cursor with project and namespace defaults
    #[allow(dead_code)]
    pub fn set_cursor_with_defaults(
        &self,
        name: &str,
        database_path: PathBuf,
        user: &str,
        project: Option<String>,
        namespace: Option<String>,
    ) {
        let mut cursor_data = CursorData::new(database_path.clone(), user.to_string());
        cursor_data.default_project = project;
        cursor_data.default_namespace = namespace;
        
        // Determine database name and use database-scoped storage
        let db_name = self.extract_database_name(&database_path);
        let cursor_file = self.get_cursor_file_path_scoped(name, user, &db_name);
        
        let json_content = serde_json::to_string_pretty(&cursor_data)
            .expect("Failed to serialize cursor data with defaults");
        fs::write(&cursor_file, json_content)
            .expect("Failed to write cursor file with defaults");
    }

    /// Set a cursor with meta context for enhanced 4-layer addressing
    pub fn set_cursor_with_meta(
        &self,
        name: &str,
        database_path: PathBuf,
        user: &str,
        meta_context: Option<String>,
        project: Option<String>,
        namespace: Option<String>,
    ) {
        let mut cursor_data = CursorData::new(database_path.clone(), user.to_string());
        cursor_data.meta_context = meta_context;
        cursor_data.default_project = project;
        cursor_data.default_namespace = namespace;
        
        // Determine database name and use database-scoped storage
        let db_name = self.extract_database_name(&database_path);
        let cursor_file = self.get_cursor_file_path_scoped(name, user, &db_name);
        
        let json_content = serde_json::to_string_pretty(&cursor_data)
            .expect("Failed to serialize cursor data with meta context");
        fs::write(&cursor_file, json_content)
            .expect("Failed to write cursor file with meta context");
    }

    /// Get cursor data for a named cursor (with backward compatibility)
    pub fn get_cursor(&self, name: &str, user: &str) -> Result<CursorData, Box<dyn std::error::Error>> {
        // Try to find cursor with backward compatibility
        if let Some((cursor_file, _)) = self.find_cursor_file_with_fallback(name, user) {
            let content = fs::read_to_string(&cursor_file)?;
            let cursor_data: CursorData = serde_json::from_str(&content)?;
            Ok(cursor_data)
        } else {
            Err(format!("Cursor '{}' not found for user '{}'", name, user).into())
        }
    }
    
    /// Find cursor file with backward compatibility
    /// Returns (cursor_file_path, is_legacy) where is_legacy indicates if found in old location
    fn find_cursor_file_with_fallback(&self, name: &str, user: &str) -> Option<(PathBuf, bool)> {
        // First, try to find in database-scoped locations
        // We need to check all possible database directories since we don't know which one
        // This is expensive but necessary for backward compatibility
        
        // Get all possible database directories
        let data_dir_entries = match fs::read_dir(&self.xdg.data_dir) {
            Ok(entries) => entries,
            Err(_) => return None,
        };
        
        for entry in data_dir_entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                if let Some(db_name) = path.file_name().and_then(|n| n.to_str()) {
                    // Skip the legacy cursors directory
                    if db_name == "cursors" { continue; }
                    
                    let cursor_file = self.get_cursor_file_path_scoped(name, user, db_name);
                    if cursor_file.exists() {
                        return Some((cursor_file, false));  // Found in new location
                    }
                }
            }
        }
        
        // Fallback: check legacy location
        let legacy_cursor_file = self.get_cursor_file_path(name, user);
        if legacy_cursor_file.exists() {
            Some((legacy_cursor_file, true))  // Found in legacy location
        } else {
            None  // Not found anywhere
        }
    }

    /// Get the active cursor (default cursor for user)
    #[allow(dead_code)]
    pub fn get_active_cursor(&self, user: &str) -> Result<Option<CursorData>, Box<dyn std::error::Error>> {
        match self.get_cursor("default", user) {
            Ok(cursor) => Ok(Some(cursor)),
            Err(_) => Ok(None),
        }
    }

    /// List all cursors for a user (searches both legacy and database-scoped locations)
    #[allow(dead_code)]
    pub fn list_cursors(&self, user: &str) -> Result<HashMap<String, CursorData>, Box<dyn std::error::Error>> {
        let mut cursors = HashMap::new();
        let user_suffix = if user == "default" { ".cursor".to_string() } else { format!(".{}.cursor", user) };
        
        // Search database-scoped locations first
        if self.xdg.data_dir.exists() {
            for entry in fs::read_dir(&self.xdg.data_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    if let Some(db_name) = path.file_name().and_then(|n| n.to_str()) {
                        // Skip the legacy cursors directory
                        if db_name == "cursors" { continue; }
                        
                        let cursor_dir = path.join("cursors");
                        if cursor_dir.exists() {
                            self.scan_cursor_directory(&cursor_dir, user, &user_suffix, &mut cursors)?;
                        }
                    }
                }
            }
        }
        
        // Search legacy location
        if self.cursor_dir.exists() {
            self.scan_cursor_directory(&self.cursor_dir, user, &user_suffix, &mut cursors)?;
        }
        
        Ok(cursors)
    }
    
    /// Helper method to scan a cursor directory and add found cursors to the map
    fn scan_cursor_directory(&self, cursor_dir: &PathBuf, user: &str, user_suffix: &str, cursors: &mut HashMap<String, CursorData>) -> Result<(), Box<dyn std::error::Error>> {
        for entry in fs::read_dir(cursor_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // More precise matching for default user to avoid matching user-specific cursors
                let matches_user = if user == "default" {
                    // For default user, match files that end with ".cursor" but NOT ".{someuser}.cursor"
                    // This means the filename should NOT contain a pattern like ".someuser.cursor"
                    if filename.ends_with(".cursor") {
                        // Check if it's a user-specific cursor (contains ".{user}.cursor" pattern)
                        let before_cursor = filename.strip_suffix(".cursor").unwrap_or("");
                        !before_cursor.contains('.')
                    } else {
                        false
                    }
                } else {
                    // For specific users, match exact suffix
                    filename.ends_with(user_suffix)
                };
                
                if matches_user {
                    let cursor_name = if user == "default" {
                        filename.strip_suffix(".cursor").unwrap_or(filename)
                    } else {
                        filename.strip_suffix(user_suffix).unwrap_or(filename)
                    };
                    
                    // Only add if not already found (database-scoped takes precedence over legacy)
                    if !cursors.contains_key(cursor_name) {
                        if let Ok(content) = fs::read_to_string(&path) {
                            if let Ok(cursor_data) = serde_json::from_str::<CursorData>(&content) {
                                cursors.insert(cursor_name.to_string(), cursor_data);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// List all cursors across all users (for admin purposes)
    #[allow(dead_code)]
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

    /// Delete a cursor (searches both legacy and database-scoped locations)
    #[allow(dead_code)]
    pub fn delete_cursor(&self, name: &str, user: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Try to find cursor with fallback and delete it
        if let Some((cursor_file, _)) = self.find_cursor_file_with_fallback(name, user) {
            fs::remove_file(&cursor_file)?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Reset (clear) all cursors for a user or globally
    #[allow(dead_code)]
    pub fn reset_cursors(&self, user: Option<&str>) -> Result<usize, Box<dyn std::error::Error>> {
        let mut deleted_count = 0;
        
        if let Some(user) = user {
            // Reset cursors for specific user only
            let user_cursor_file = self.cursor_dir.join(format!("cursor_{}", user));
            if user_cursor_file.exists() {
                fs::remove_file(&user_cursor_file)?;
                deleted_count += 1;
            }
        } else {
            // Reset all cursors (global and per-user)
            let cursor_dir = &self.cursor_dir;
            if cursor_dir.exists() {
                for entry in fs::read_dir(cursor_dir)? {
                    let entry = entry?;
                    let path = entry.path();
                    if path.is_file() && path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.starts_with("cursor"))
                        .unwrap_or(false) 
                    {
                        fs::remove_file(&path)?;
                        deleted_count += 1;
                    }
                }
            }
        }
        
        Ok(deleted_count)
    }

    /// Resolve database path from cursor name and user
    /// Returns the cursor's database path if found, or None to use default
    /// Checks for working directory .prontodb file first, then global cursors
    pub fn resolve_database_path(&self, cursor_name: Option<&str>, user: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let name = cursor_name.unwrap_or("default");
        
        // First, check for working directory cursor override
        if let Some(local_path) = self.resolve_working_directory_cursor(name, user)? {
            return Ok(Some(local_path));
        }
        
        // Fall back to global cursor system
        match self.get_cursor(name, user) {
            Ok(cursor) => Ok(Some(cursor.database_path)),
            Err(_) => Ok(None),
        }
    }

    /// Check for working directory cursor file (.prontodb)
    /// Format supports both simple path and JSON with per-user/per-cursor settings
    /// Controlled by PRONTO_WORK_MODE environment variable (opt-in)
    fn resolve_working_directory_cursor(&self, cursor_name: &str, user: &str) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        // Check if working directory mode is enabled via environment variable
        let work_mode_enabled = match std::env::var("PRONTO_WORK_MODE") {
            Ok(val) => val == "1" || val.to_lowercase() == "true" || val.to_lowercase() == "on",
            Err(_) => false,
        };
        
        if !work_mode_enabled {
            return Ok(None);
        }
        
        let current_dir = std::env::current_dir()?;
        let prontodb_file = current_dir.join(".prontodb");
        
        if !prontodb_file.exists() {
            return Ok(None);
        }
        
        let content = fs::read_to_string(&prontodb_file)?;
        let content = content.trim();
        
        // Try parsing as JSON first (advanced format)
        if content.starts_with('{') {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Check user-specific cursor
                if let Some(user_cursors) = config.get("users").and_then(|u| u.get(user)) {
                    if let Some(cursor_path) = user_cursors.get(cursor_name).and_then(|v| v.as_str()) {
                        return Ok(Some(PathBuf::from(cursor_path)));
                    }
                }
                
                // Check default cursors
                if let Some(default_cursors) = config.get("cursors") {
                    if let Some(cursor_path) = default_cursors.get(cursor_name).and_then(|v| v.as_str()) {
                        return Ok(Some(PathBuf::from(cursor_path)));
                    }
                }
                
                // Check simple path for default cursor
                if cursor_name == "default" {
                    if let Some(path) = config.get("path").and_then(|v| v.as_str()) {
                        return Ok(Some(PathBuf::from(path)));
                    }
                }
            }
        } else {
            // Simple format - just a path for default cursor
            if cursor_name == "default" {
                return Ok(Some(PathBuf::from(content)));
            }
        }
        
        Ok(None)
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

    /// Get cursor file path for name and user (legacy method - still used for backward compatibility)
    fn get_cursor_file_path(&self, name: &str, user: &str) -> PathBuf {
        if user == "default" {
            self.cursor_dir.join(format!("{}.cursor", name))
        } else {
            self.cursor_dir.join(format!("{}.{}.cursor", name, user))
        }
    }
    
    /// Get cursor file path for name and user in database-scoped directory
    fn get_cursor_file_path_scoped(&self, name: &str, user: &str, db_name: &str) -> PathBuf {
        let db_cursor_dir = self.xdg.get_cursor_dir_with_name(db_name);
        
        // Ensure database cursor directory exists
        if let Err(e) = fs::create_dir_all(&db_cursor_dir) {
            eprintln!("Warning: Failed to create cursor directory {}: {}", db_cursor_dir.display(), e);
            // Fall back to legacy path
            return self.get_cursor_file_path(name, user);
        }
        
        if user == "default" {
            db_cursor_dir.join(format!("{}.cursor", name))
        } else {
            db_cursor_dir.join(format!("{}.{}.cursor", name, user))
        }
    }
    
    /// Migrate a legacy cursor to the new database-scoped structure
    /// Returns true if migration was performed, false if no migration needed
    pub fn migrate_legacy_cursor(&self, name: &str, user: &str) -> Result<bool, Box<dyn std::error::Error>> {
        // Check if cursor exists in legacy location
        let legacy_file = self.get_cursor_file_path(name, user);
        if !legacy_file.exists() {
            return Ok(false);  // No legacy cursor to migrate
        }
        
        // Read the legacy cursor data
        let content = fs::read_to_string(&legacy_file)?;
        let cursor_data: CursorData = serde_json::from_str(&content)?;
        
        // Determine database name from the cursor's database path
        let db_name = self.extract_database_name(&cursor_data.database_path);
        let new_file = self.get_cursor_file_path_scoped(name, user, &db_name);
        
        // Check if already exists in new location
        if new_file.exists() {
            // Already migrated, just clean up legacy file
            fs::remove_file(&legacy_file)?;
            return Ok(true);
        }
        
        // Copy to new location
        fs::write(&new_file, &content)?;
        
        // Remove legacy file
        fs::remove_file(&legacy_file)?;
        
        Ok(true)
    }
    
    /// Migrate all legacy cursors to database-scoped structure  
    /// This is a convenience method to migrate all cursors for a user
    #[allow(dead_code)]
    pub fn migrate_all_legacy_cursors(&self, user: &str) -> Result<usize, Box<dyn std::error::Error>> {
        let mut migrated_count = 0;
        
        // Read all files from legacy cursor directory
        if !self.cursor_dir.exists() {
            return Ok(0);  // No legacy directory
        }
        
        for entry in fs::read_dir(&self.cursor_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                // Parse cursor name and user from filename
                let (cursor_name, cursor_user) = if filename.ends_with(".cursor") {
                    if filename.contains('.') && filename != ".cursor" {
                        // Format: name.user.cursor or just name.cursor
                        let parts: Vec<&str> = filename.rsplitn(3, '.').collect();
                        if parts.len() == 3 && parts[0] == "cursor" {
                            // name.user.cursor
                            (parts[2], parts[1])
                        } else if parts.len() == 2 && parts[0] == "cursor" {
                            // name.cursor (default user)
                            (parts[1], "default")
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    }
                } else {
                    continue;  // Not a cursor file
                };
                
                // Only migrate cursors for the specified user
                if cursor_user != user {
                    continue;
                }
                
                // Attempt migration
                match self.migrate_legacy_cursor(cursor_name, cursor_user) {
                    Ok(true) => migrated_count += 1,
                    Ok(false) => {}, // Nothing to migrate
                    Err(e) => eprintln!("Warning: Failed to migrate cursor {}.{}: {}", cursor_name, cursor_user, e),
                }
            }
        }
        
        Ok(migrated_count)
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
        assert!(cursor.meta_context.is_none());
    }

    #[test]
    fn test_cursor_data_with_meta_context() {
        let db_path = PathBuf::from("/test/db.sqlite");
        let cursor = CursorData::new(db_path, "testuser".to_string())
            .with_meta_context("company_engineering".to_string())
            .with_project("bashfx".to_string())
            .with_namespace("config".to_string());
        
        assert_eq!(cursor.meta_context, Some("company_engineering".to_string()));
        assert_eq!(cursor.default_project, Some("bashfx".to_string()));
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
    
    #[test]
    fn test_database_scoped_cursor_storage() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        // Create cursors pointing to different databases
        cursor_manager.set_cursor("prod", PathBuf::from("/path/to/production.db"), "alice");
        cursor_manager.set_cursor("staging", PathBuf::from("/path/to/staging.db"), "alice");
        cursor_manager.set_cursor("test", PathBuf::from("/path/to/test.db"), "alice");
        
        // Verify cursors are stored in database-scoped directories
        let prod_cursor_file = test_xdg.paths.get_cursor_dir_with_name("production").join("prod.alice.cursor");
        let staging_cursor_file = test_xdg.paths.get_cursor_dir_with_name("staging").join("staging.alice.cursor");
        let test_cursor_file = test_xdg.paths.get_cursor_dir_with_name("test").join("test.alice.cursor");
        
        assert!(prod_cursor_file.exists(), "Production cursor should exist in production/cursors/");
        assert!(staging_cursor_file.exists(), "Staging cursor should exist in staging/cursors/");
        assert!(test_cursor_file.exists(), "Test cursor should exist in test/cursors/");
        
        // Verify they can be retrieved
        let prod_cursor = cursor_manager.get_cursor("prod", "alice").unwrap();
        let staging_cursor = cursor_manager.get_cursor("staging", "alice").unwrap();
        let test_cursor = cursor_manager.get_cursor("test", "alice").unwrap();
        
        assert_eq!(prod_cursor.database_path, PathBuf::from("/path/to/production.db"));
        assert_eq!(staging_cursor.database_path, PathBuf::from("/path/to/staging.db"));
        assert_eq!(test_cursor.database_path, PathBuf::from("/path/to/test.db"));
    }
    
    #[test]
    fn test_extract_database_name() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        // Test various path formats
        assert_eq!(cursor_manager.extract_database_name(&PathBuf::from("/path/to/myapp.db")), "myapp");
        assert_eq!(cursor_manager.extract_database_name(&PathBuf::from("/path/to/staging.sqlite")), "staging");
        assert_eq!(cursor_manager.extract_database_name(&PathBuf::from("/custom/production.db")), "production");
        
        // Test default main path
        let main_path = test_xdg.paths.get_db_path();
        assert_eq!(cursor_manager.extract_database_name(&main_path), "main");
        
        // Test edge case
        assert_eq!(cursor_manager.extract_database_name(&PathBuf::from("/no/extension")), "extension");
    }
    
    #[test] 
    fn test_legacy_cursor_migration() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        // Create a cursor in legacy location (simulate old cursor)
        let legacy_file = cursor_manager.get_cursor_file_path("legacy", "alice");
        let cursor_data = CursorData::new(PathBuf::from("/path/to/myapp.db"), "alice".to_string());
        let json_content = serde_json::to_string_pretty(&cursor_data).unwrap();
        fs::create_dir_all(legacy_file.parent().unwrap()).unwrap();
        fs::write(&legacy_file, &json_content).unwrap();
        
        // Verify it exists in legacy location
        assert!(legacy_file.exists());
        
        // Verify backward compatibility - should be able to read it
        let retrieved_cursor = cursor_manager.get_cursor("legacy", "alice").unwrap();
        assert_eq!(retrieved_cursor.database_path, PathBuf::from("/path/to/myapp.db"));
        
        // Test migration
        let migrated = cursor_manager.migrate_legacy_cursor("legacy", "alice").unwrap();
        assert!(migrated, "Migration should have occurred");
        
        // Verify legacy file is gone
        assert!(!legacy_file.exists(), "Legacy file should be removed after migration");
        
        // Verify new file exists in database-scoped location
        let new_file = test_xdg.paths.get_cursor_dir_with_name("myapp").join("legacy.alice.cursor");
        assert!(new_file.exists(), "Cursor should exist in database-scoped location");
        
        // Verify cursor can still be retrieved
        let retrieved_cursor = cursor_manager.get_cursor("legacy", "alice").unwrap();
        assert_eq!(retrieved_cursor.database_path, PathBuf::from("/path/to/myapp.db"));
    }

    #[test]
    fn test_set_cursor_with_meta_context() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        let db_path = PathBuf::from("/path/to/work.db");
        cursor_manager.set_cursor_with_meta(
            "work",
            db_path.clone(),
            "alice",
            Some("company_engineering".to_string()),
            Some("bashfx".to_string()),
            Some("config".to_string()),
        );
        
        let cursor = cursor_manager.get_cursor("work", "alice").unwrap();
        assert_eq!(cursor.database_path, db_path);
        assert_eq!(cursor.meta_context, Some("company_engineering".to_string()));
        assert_eq!(cursor.default_project, Some("bashfx".to_string()));
        assert_eq!(cursor.default_namespace, Some("config".to_string()));
        assert_eq!(cursor.user, "alice");
    }
    
    #[test]
    fn test_set_cursor_with_meta_context_none() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        let db_path = PathBuf::from("/path/to/personal.db");
        cursor_manager.set_cursor_with_meta(
            "personal",
            db_path.clone(),
            "bob",
            None, // No meta context
            None,
            None,
        );
        
        let cursor = cursor_manager.get_cursor("personal", "bob").unwrap();
        assert_eq!(cursor.database_path, db_path);
        assert!(cursor.meta_context.is_none());
        assert!(cursor.default_project.is_none());
        assert!(cursor.default_namespace.is_none());
        assert_eq!(cursor.user, "bob");
    }

    #[test]
    fn test_meta_namespace_backward_compatibility() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        // Create an old cursor without meta context
        cursor_manager.set_cursor("legacy", PathBuf::from("/legacy.db"), "alice");
        
        // Verify it can be read and has no meta context
        let cursor = cursor_manager.get_cursor("legacy", "alice").unwrap();
        assert!(cursor.meta_context.is_none());
        assert_eq!(cursor.database_path, PathBuf::from("/legacy.db"));
    }

    #[test]
    fn test_meta_namespace_json_serialization() {
        let cursor_data = CursorData::new(PathBuf::from("/test.db"), "testuser".to_string())
            .with_meta_context("company_engineering".to_string())
            .with_project("bashfx".to_string());
        
        // Test serialization
        let json = serde_json::to_string(&cursor_data).unwrap();
        assert!(json.contains("meta_context"));
        assert!(json.contains("company_engineering"));
        
        // Test deserialization
        let deserialized: CursorData = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.meta_context, Some("company_engineering".to_string()));
        assert_eq!(deserialized.default_project, Some("bashfx".to_string()));
    }

    #[test]
    fn test_meta_namespace_cursor_listing() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        // Create cursors with and without meta context
        cursor_manager.set_cursor_with_meta(
            "work",
            PathBuf::from("/work.db"),
            "alice",
            Some("company_engineering".to_string()),
            Some("project1".to_string()),
            Some("config".to_string()),
        );
        
        cursor_manager.set_cursor("personal", PathBuf::from("/personal.db"), "alice");
        
        let cursors = cursor_manager.list_cursors("alice").unwrap();
        assert_eq!(cursors.len(), 2);
        
        let work_cursor = &cursors["work"];
        assert_eq!(work_cursor.meta_context, Some("company_engineering".to_string()));
        
        let personal_cursor = &cursors["personal"];
        assert!(personal_cursor.meta_context.is_none());
    }

    #[test]
    fn test_meta_namespace_cursor_deletion() {
        let test_xdg = TestXdg::new().unwrap();
        let cursor_manager = CursorManager::from_xdg(test_xdg.paths.clone());
        
        cursor_manager.set_cursor_with_meta(
            "temp_meta",
            PathBuf::from("/temp.db"),
            "alice",
            Some("test_org".to_string()),
            None,
            None,
        );
        
        // Verify it exists
        assert!(cursor_manager.get_cursor("temp_meta", "alice").is_ok());
        
        // Delete it
        let deleted = cursor_manager.delete_cursor("temp_meta", "alice").unwrap();
        assert!(deleted);
        
        // Verify it's gone
        assert!(cursor_manager.get_cursor("temp_meta", "alice").is_err());
    }
}