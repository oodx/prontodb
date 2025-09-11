// Global Cursor Caching System for ProntoDB
// Provides lightweight persistent database selection using simple text files
// Location: ~/.local/etc/prontodb/cursor and ~/.local/etc/prontodb/cursor_<user>

use std::fs;
use std::path::{Path, PathBuf};
use std::io::{self, ErrorKind};

/// Global cursor cache manager for persistent database selection
pub struct CursorCache {
    cache_dir: PathBuf,
}

impl CursorCache {
    /// Create new cursor cache manager
    /// Uses ~/.local/etc/prontodb/ directory for cache files
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
        let cache_dir = PathBuf::from(&home)
            .join(".local")
            .join("etc")  
            .join("prontodb");
        
        // Ensure cache directory exists
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cursor cache directory {}: {}", cache_dir.display(), e);
        }
        
        Self { cache_dir }
    }

    /// Create cursor cache from specific directory (for testing)
    #[allow(dead_code)]
    pub fn from_dir(cache_dir: PathBuf) -> Self {
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Warning: Failed to create cursor cache directory {}: {}", cache_dir.display(), e);
        }
        Self { cache_dir }
    }

    /// Set global cursor (database selection) for a user
    /// File format: Simple text file containing just the database name
    pub fn set_cursor(&self, database: &str, user: Option<&str>) -> Result<(), io::Error> {
        let cache_file = self.get_cache_file_path(user);
        
        // Write database name to cache file
        fs::write(&cache_file, database.trim())?;
        
        Ok(())
    }

    /// Get current cursor (database selection) for a user
    /// Returns None if no cursor is cached or file doesn't exist
    pub fn get_cursor(&self, user: Option<&str>) -> Option<String> {
        let cache_file = self.get_cache_file_path(user);
        
        match fs::read_to_string(&cache_file) {
            Ok(content) => {
                let database = content.trim();
                if database.is_empty() {
                    None
                } else {
                    Some(database.to_string())
                }
            }
            Err(ref e) if e.kind() == ErrorKind::NotFound => None,
            Err(e) => {
                eprintln!("Warning: Failed to read cursor cache {}: {}", cache_file.display(), e);
                None
            }
        }
    }

    /// Clear cursor cache for a user
    pub fn clear_cursor(&self, user: Option<&str>) -> Result<(), io::Error> {
        let cache_file = self.get_cache_file_path(user);
        
        match fs::remove_file(&cache_file) {
            Ok(()) => Ok(()),
            Err(ref e) if e.kind() == ErrorKind::NotFound => Ok(()), // Already doesn't exist
            Err(e) => Err(e),
        }
    }

    /// Check if cursor cache exists for a user
    #[allow(dead_code)]
    pub fn has_cursor(&self, user: Option<&str>) -> bool {
        let cache_file = self.get_cache_file_path(user);
        cache_file.exists() && self.get_cursor(user).is_some()
    }

    /// List all cursor cache files (for debugging/admin purposes)
    #[allow(dead_code)]  // Future feature for cursor management
    pub fn list_all_cursors(&self) -> Vec<(String, String)> {
        let mut cursors = Vec::new();
        
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename == "cursor" {
                        // Global cursor file
                        if let Some(database) = self.get_cursor(None) {
                            cursors.push(("default".to_string(), database));
                        }
                    } else if filename.starts_with("cursor_") {
                        // User-specific cursor file
                        let user = filename.strip_prefix("cursor_").unwrap_or("unknown");
                        if let Some(database) = self.get_cursor(Some(user)) {
                            cursors.push((user.to_string(), database));
                        }
                    }
                }
            }
        }
        
        cursors
    }

    /// Get cache file path for a user
    /// Format: "cursor" for global, "cursor_<user>" for user-specific
    fn get_cache_file_path(&self, user: Option<&str>) -> PathBuf {
        match user {
            Some(u) if !u.is_empty() && u != "default" => {
                self.cache_dir.join(format!("cursor_{}", u))
            }
            _ => {
                // Default user or None uses global cursor file
                self.cache_dir.join("cursor")
            }
        }
    }

    /// Get the cache directory path (for testing and debugging)
    #[allow(dead_code)]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }
}

impl Default for CursorCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_cache() -> (TempDir, CursorCache) {
        let temp_dir = TempDir::new().unwrap();
        let cache_dir = temp_dir.path().join("etc").join("prontodb");
        let cache = CursorCache::from_dir(cache_dir);
        (temp_dir, cache)
    }

    #[test]
    fn test_cursor_cache_creation() {
        let (_temp, cache) = setup_test_cache();
        assert!(cache.cache_dir.exists());
    }

    #[test]
    fn test_set_and_get_global_cursor() {
        let (_temp, cache) = setup_test_cache();

        // Set global cursor
        cache.set_cursor("staging", None).unwrap();
        
        // Get global cursor
        let cursor = cache.get_cursor(None);
        assert_eq!(cursor, Some("staging".to_string()));
    }

    #[test]
    fn test_set_and_get_user_cursor() {
        let (_temp, cache) = setup_test_cache();

        // Set user-specific cursor
        cache.set_cursor("prod", Some("alice")).unwrap();
        
        // Get user cursor
        let cursor = cache.get_cursor(Some("alice"));
        assert_eq!(cursor, Some("prod".to_string()));
        
        // Global cursor should be empty
        let global = cache.get_cursor(None);
        assert_eq!(global, None);
    }

    #[test]
    fn test_default_user_uses_global() {
        let (_temp, cache) = setup_test_cache();

        // Set cursor for "default" user should use global file
        cache.set_cursor("test", Some("default")).unwrap();
        
        // Both default user and global should return same value
        let default_cursor = cache.get_cursor(Some("default"));
        let global_cursor = cache.get_cursor(None);
        
        assert_eq!(default_cursor, Some("test".to_string()));
        assert_eq!(global_cursor, Some("test".to_string()));
    }

    #[test]
    fn test_has_cursor() {
        let (_temp, cache) = setup_test_cache();

        // Initially no cursor
        assert!(!cache.has_cursor(None));
        assert!(!cache.has_cursor(Some("alice")));
        
        // Set cursors
        cache.set_cursor("staging", None).unwrap();
        cache.set_cursor("prod", Some("alice")).unwrap();
        
        // Now should have cursors
        assert!(cache.has_cursor(None));
        assert!(cache.has_cursor(Some("alice")));
        assert!(!cache.has_cursor(Some("bob")));
    }

    #[test]
    fn test_clear_cursor() {
        let (_temp, cache) = setup_test_cache();

        // Set and verify cursor
        cache.set_cursor("test", Some("alice")).unwrap();
        assert!(cache.has_cursor(Some("alice")));
        
        // Clear cursor
        cache.clear_cursor(Some("alice")).unwrap();
        assert!(!cache.has_cursor(Some("alice")));
        
        // Clear non-existing cursor should not error
        cache.clear_cursor(Some("nonexistent")).unwrap();
    }

    #[test]
    fn test_list_all_cursors() {
        let (_temp, cache) = setup_test_cache();

        // Set multiple cursors
        cache.set_cursor("staging", None).unwrap();
        cache.set_cursor("prod", Some("alice")).unwrap();
        cache.set_cursor("dev", Some("bob")).unwrap();
        
        let cursors = cache.list_all_cursors();
        assert_eq!(cursors.len(), 3);
        
        // Check cursor contents
        assert!(cursors.contains(&("default".to_string(), "staging".to_string())));
        assert!(cursors.contains(&("alice".to_string(), "prod".to_string())));
        assert!(cursors.contains(&("bob".to_string(), "dev".to_string())));
    }

    #[test]
    fn test_file_format() {
        let (_temp, cache) = setup_test_cache();

        // Set cursor and verify file contents directly
        cache.set_cursor("mydb", Some("alice")).unwrap();
        
        let cache_file = cache.get_cache_file_path(Some("alice"));
        let content = fs::read_to_string(&cache_file).unwrap();
        
        assert_eq!(content.trim(), "mydb");
        assert!(cache_file.file_name().unwrap() == "cursor_alice");
    }

    #[test]
    fn test_whitespace_handling() {
        let (_temp, cache) = setup_test_cache();

        // Set cursor with extra whitespace
        cache.set_cursor("  staging  \n", None).unwrap();
        
        // Should be trimmed when retrieved
        let cursor = cache.get_cursor(None);
        assert_eq!(cursor, Some("staging".to_string()));
    }

    #[test]
    fn test_empty_cursor_handling() {
        let (_temp, cache) = setup_test_cache();

        // Set empty cursor
        cache.set_cursor("", None).unwrap();
        
        // Should return None for empty content
        let cursor = cache.get_cursor(None);
        assert_eq!(cursor, None);
        
        // But file should still exist
        let cache_file = cache.get_cache_file_path(None);
        assert!(cache_file.exists());
    }
}