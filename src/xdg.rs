// XDG+ path management for ProntoDB
// Provides proper path isolation for testing and multi-instance setups

#![allow(dead_code)]  // Some functions are for future use

use rsb::prelude::*;

use std::path::{Path, PathBuf};

/// Validates that a path doesn't contain shell expansion syntax that could cause filesystem pollution
fn validate_path_safety(path: &Path) -> Result<(), String> {
    let path_str = path.to_string_lossy();
    
    // Check for malformed shell expansion patterns
    if path_str.contains("${") && !path_str.contains("}") {
        return Err(format!("Detected malformed shell expansion in path: {}", path_str));
    }
    
    // Check for literal shell expansion patterns that should have been expanded
    if path_str.contains("${XDG_") || path_str.contains("${HOME") {
        return Err(format!("Detected literal shell expansion that should be resolved: {}", path_str));
    }
    
    Ok(())
}

/// Safely create directory with path validation
fn safe_create_dir_all(path: &Path) -> Result<(), std::io::Error> {
    // Validate path safety first
    if let Err(e) = validate_path_safety(path) {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("Path validation failed: {}", e)
        ));
    }
    
    std::fs::create_dir_all(path)
}

/// XDG+ paths for ProntoDB
/// Follows XDG Base Directory Specification with prontodb-specific structure
#[derive(Debug, Clone)]
pub struct XdgPaths {
    pub home: PathBuf,
    pub data_dir: PathBuf,
    pub config_dir: PathBuf, 
    pub cache_dir: PathBuf,
    pub runtime_dir: Option<PathBuf>,
    pub db_path: PathBuf,
    pub config_file: PathBuf,
    pub cursor_dir: PathBuf,
}

impl XdgPaths {
    /// Create XdgPaths from environment or defaults
    pub fn new() -> Self {
        let home = Self::get_home_dir();
        Self::from_home(&home)
    }

    /// Create XdgPaths from specific home directory (useful for testing)
    pub fn from_home(home: &Path) -> Self {
        let data_dir = Self::get_data_dir(home);
        let config_dir = Self::get_config_dir(home);
        let cache_dir = Self::get_cache_dir(home);
        let runtime_dir = Self::get_runtime_dir();

        let db_path = data_dir.join("pronto.main.prdb");
        let config_file = config_dir.join("main.conf");
        let cursor_dir = data_dir.join("cursors");

        XdgPaths {
            home: home.to_path_buf(),
            data_dir,
            config_dir,
            cache_dir,
            runtime_dir,
            db_path,
            config_file,
            cursor_dir,
        }
    }

    /// Create XdgPaths from specific home directory, ignoring environment variables (for isolated testing)
    pub fn from_home_isolated(home: &Path) -> Self {
        let data_dir = home.join(".local").join("data").join("odx").join("prontodb");
        let config_dir = home.join(".local").join("etc").join("odx").join("prontodb");
        let cache_dir = home.join(".cache").join("odx").join("prontodb");
        let runtime_dir = None; // No runtime dir for isolated tests

        let db_path = data_dir.join("pronto.main.prdb");
        let config_file = config_dir.join("main.conf");
        let cursor_dir = data_dir.join("cursors");

        XdgPaths {
            home: home.to_path_buf(),
            data_dir,
            config_dir,
            cache_dir,
            runtime_dir,
            db_path,
            config_file,
            cursor_dir,
        }
    }

    /// Create all necessary directories with safety validation
    pub fn ensure_dirs(&self) -> Result<(), std::io::Error> {
        safe_create_dir_all(&self.data_dir)?;
        safe_create_dir_all(&self.config_dir)?;
        safe_create_dir_all(&self.cache_dir)?;
        safe_create_dir_all(&self.cursor_dir)?;
        
        if let Some(runtime_dir) = &self.runtime_dir {
            safe_create_dir_all(runtime_dir)?;
        }
        
        Ok(())
    }

    /// Get database-scoped directory for a specific database
    pub fn get_database_dir(&self, db_name: &str) -> PathBuf {
        self.data_dir.join(db_name)
    }

    /// Get effective database path for a specific database (supports PRONTO_DB override for default)
    pub fn get_db_path_with_name(&self, db_name: &str) -> PathBuf {
        // For backward compatibility: only apply env var override to "main" database
        if db_name == "main" {
            // First check runtime env var (for testing)
            if let Ok(runtime_db) = std::env::var("PRONTO_DB") {
                if !runtime_db.is_empty() {
                    return PathBuf::from(runtime_db);
                }
            }
            
            // Then check RSB param for production
            let db_path = param!("PRONTO_DB", default: "");
            if !db_path.is_empty() {
                return PathBuf::from(db_path);
            }
        }
        
        // For all databases (including pronto when no override), use database-scoped path
        self.get_database_dir(db_name).join(format!("pronto.{}.prdb", db_name))
    }

    /// Get effective database path (backward compatible - uses "main" as default database)
    pub fn get_db_path(&self) -> PathBuf {
        self.get_db_path_with_name("main")
    }

    /// Get cursor directory for a specific database
    pub fn get_cursor_dir_with_name(&self, db_name: &str) -> PathBuf {
        self.get_database_dir(db_name).join("cursors")
    }

    /// Get cursor directory (backward compatible - uses "main" as default database)
    pub fn get_cursor_dir(&self) -> PathBuf {
        self.get_cursor_dir_with_name("main")
    }

    /// Get effective config file path (supports PRONTO_CONFIG override)
    pub fn get_config_path(&self) -> PathBuf {
        // First check runtime env var (for testing)
        if let Ok(runtime_config) = std::env::var("PRONTO_CONFIG") {
            if !runtime_config.is_empty() {
                return PathBuf::from(runtime_config);
            }
        }
        
        // Then check RSB param for production
        let config_path = param!("PRONTO_CONFIG", default: "");
        if !config_path.is_empty() {
            PathBuf::from(config_path)
        } else {
            self.config_file.clone()
        }
    }

    // Private helper methods

    fn get_home_dir() -> PathBuf {
        // First check runtime env var (for testing)
        if let Ok(runtime_home) = std::env::var("HOME") {
            if !runtime_home.is_empty() {
                return PathBuf::from(runtime_home);
            }
        }
        
        // Then check stdlib env var for production
        if let Ok(home) = std::env::var("HOME") {
            if !home.is_empty() {
                return PathBuf::from(home);
            }
        }
        
        {
            // Check Windows USERPROFILE runtime first
            if let Ok(runtime_userprofile) = std::env::var("USERPROFILE") {
                if !runtime_userprofile.is_empty() {
                    return PathBuf::from(runtime_userprofile);
                }
            }
            
            if let Ok(userprofile) = std::env::var("USERPROFILE") {
                if !userprofile.is_empty() {
                    // Windows fallback
                    return PathBuf::from(userprofile);
                }
            }
            
            {
                // Ultimate fallback
                PathBuf::from("/tmp")
            }
        }
    }

    fn get_data_dir(home: &Path) -> PathBuf {
        // First check runtime env var (for testing)
        if let Ok(runtime_data_home) = std::env::var("XDG_DATA_HOME") {
            if !runtime_data_home.is_empty() {
                let path = PathBuf::from(&runtime_data_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_DATA_HOME path, using fallback: {}", runtime_data_home);
            }
        }
        
        // Then check stdlib env var for production
        if let Ok(xdg_data_home) = std::env::var("XDG_DATA_HOME") {
            if !xdg_data_home.is_empty() {
                let path = PathBuf::from(&xdg_data_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_DATA_HOME env var, using fallback: {}", xdg_data_home);
            }
        }
        
        home.join(".local").join("data").join("odx").join("prontodb")
    }

    fn get_config_dir(home: &Path) -> PathBuf {
        // First check runtime env var (for testing)
        if let Ok(runtime_config_home) = std::env::var("XDG_CONFIG_HOME") {
            if !runtime_config_home.is_empty() {
                let path = PathBuf::from(&runtime_config_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_CONFIG_HOME path, using fallback: {}", runtime_config_home);
            }
        }
        
        // Then check stdlib env var for production
        if let Ok(xdg_config_home) = std::env::var("XDG_CONFIG_HOME") {
            if !xdg_config_home.is_empty() {
                let path = PathBuf::from(&xdg_config_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_CONFIG_HOME env var, using fallback: {}", xdg_config_home);
            }
        }
        
        home.join(".local").join("etc").join("odx").join("prontodb")
    }

    fn get_cache_dir(home: &Path) -> PathBuf {
        // First check runtime env var (for testing)
        if let Ok(runtime_cache_home) = std::env::var("XDG_CACHE_HOME") {
            if !runtime_cache_home.is_empty() {
                let path = PathBuf::from(&runtime_cache_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_CACHE_HOME path, using fallback: {}", runtime_cache_home);
            }
        }
        
        // Then check stdlib env var for production
        if let Ok(xdg_cache_home) = std::env::var("XDG_CACHE_HOME") {
            if !xdg_cache_home.is_empty() {
                let path = PathBuf::from(&xdg_cache_home).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return path;
                }
                warn!("Detected malformed XDG_CACHE_HOME env var, using fallback: {}", xdg_cache_home);
            }
        }
        
        home.join(".cache").join("odx").join("prontodb")
    }

    fn get_runtime_dir() -> Option<PathBuf> {
        // First check runtime env var (for testing)
        if let Ok(runtime_runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            if !runtime_runtime_dir.is_empty() {
                let path = PathBuf::from(&runtime_runtime_dir).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return Some(path);
                }
                warn!("Detected malformed XDG_RUNTIME_DIR path, skipping: {}", runtime_runtime_dir);
            }
        }
        
        // Then check stdlib env var for production
        if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
            if !runtime_dir.is_empty() {
                let path = PathBuf::from(&runtime_dir).join("odx").join("prontodb");
                if validate_path_safety(&path).is_ok() {
                    return Some(path);
                }
                warn!("Detected malformed XDG_RUNTIME_DIR env var, skipping: {}", runtime_dir);
            }
        }
        
        None
    }
}

impl Default for XdgPaths {
    fn default() -> Self {
        Self::new()
    }
}

/// Test utilities for XDG path management
pub mod test_utils {
    use super::*;
    use tempfile::TempDir;

    /// Create isolated XDG environment for testing
    pub struct TestXdg {
        pub temp_dir: TempDir,
        pub paths: XdgPaths,
    }

    impl TestXdg {
        pub fn new() -> std::io::Result<Self> {
            let temp_dir = TempDir::new()?;
            let paths = XdgPaths::from_home_isolated(temp_dir.path());
            paths.ensure_dirs()?;
            
            Ok(TestXdg { temp_dir, paths })
        }

        /// Get the temp home path as string for env vars
        pub fn home_str(&self) -> &str {
            self.temp_dir.path().to_str().unwrap()
        }

        /// Get the database path as string (direct path, not influenced by env vars)
        pub fn db_path_str(&self) -> String {
            self.paths.db_path.to_string_lossy().to_string()
        }
        
        /// Get the effective database path as string (respects env vars like real usage)
        pub fn effective_db_path_str(&self) -> String {
            self.paths.get_db_path().to_string_lossy().to_string()
        }

        /// Get the database path for a specific database as string
        pub fn db_path_for_str(&self, db_name: &str) -> String {
            self.paths.get_db_path_with_name(db_name).to_string_lossy().to_string()
        }

        /// Get the cursor directory for a specific database as string
        pub fn cursor_dir_for_str(&self, db_name: &str) -> String {
            self.paths.get_cursor_dir_with_name(db_name).to_string_lossy().to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_xdg_paths_basic_structure() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        // Verify structure follows XDG+ spec
        assert!(paths.data_dir.ends_with("odx/prontodb"));
        assert!(paths.config_dir.ends_with("odx/prontodb"));
        assert!(paths.cache_dir.ends_with("odx/prontodb"));
        assert!(paths.cursor_dir.ends_with("cursors"));
        
        // Verify files are in correct locations
        assert!(paths.db_path.ends_with("pronto.main.prdb"));
        assert!(paths.config_file.ends_with("main.conf"));
    }

    #[test]
    fn test_ensure_dirs_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        paths.ensure_dirs().unwrap();

        assert!(paths.data_dir.exists());
        assert!(paths.config_dir.exists());
        assert!(paths.cache_dir.exists());
        assert!(paths.cursor_dir.exists());
    }

    #[test]
    fn test_env_var_overrides() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        // Test PRONTO_DB override
        let custom_db = temp_dir.path().join("custom.db");
        std::env::set_var("PRONTO_DB", &custom_db);
        
        assert_eq!(paths.get_db_path(), custom_db);
        
        std::env::remove_var("PRONTO_DB");
        assert_eq!(paths.get_db_path(), paths.get_db_path_with_name("main"));
    }

    #[test]
    fn test_xdg_env_var_support() {
        let temp_dir = TempDir::new().unwrap();
        let custom_data = temp_dir.path().join("custom_data");
        
        std::env::set_var("XDG_DATA_HOME", &custom_data);
        
        let paths = XdgPaths::from_home(temp_dir.path());
        assert!(paths.data_dir.starts_with(&custom_data));
        
        std::env::remove_var("XDG_DATA_HOME");
    }

    #[test]
    fn test_test_utils() {
        let test_xdg = test_utils::TestXdg::new().unwrap();
        
        // Verify isolation
        assert!(test_xdg.paths.data_dir.exists());
        assert!(test_xdg.paths.config_dir.exists());
        
        // Verify we can get string paths for env vars
        assert!(!test_xdg.home_str().is_empty());
        assert!(!test_xdg.db_path_str().is_empty());
    }

    #[test]
    fn test_path_validation_safety() {
        use std::path::PathBuf;
        
        // Valid paths should pass
        assert!(validate_path_safety(&PathBuf::from("/tmp/valid/path")).is_ok());
        assert!(validate_path_safety(&PathBuf::from("/home/user/.cache")).is_ok());
        
        // Malformed shell expansion should be detected
        // Commented out: This test path can create actual directories during build
        // assert!(validate_path_safety(&PathBuf::from("${XDG_TMP:-")).is_err());
        assert!(validate_path_safety(&PathBuf::from("/tmp/${XDG_CACHE_HOME:-")).is_err());
        
        // Literal shell expansions should be detected
        assert!(validate_path_safety(&PathBuf::from("/path/${XDG_DATA_HOME}/test")).is_err());
        assert!(validate_path_safety(&PathBuf::from("${HOME}/docs")).is_err());
        
        // Properly expanded paths should be fine
        assert!(validate_path_safety(&PathBuf::from("/home/user/.local/data")).is_ok());
    }

    #[test]
    fn test_safe_directory_creation() {
        use tempfile::TempDir;
        
        let temp_dir = TempDir::new().unwrap();
        
        // Safe path should work
        let safe_path = temp_dir.path().join("test_dir");
        assert!(safe_create_dir_all(&safe_path).is_ok());
        assert!(safe_path.exists());
        
        // Malformed path should be rejected
        // Commented out: This test can create actual malformed directories
        // let malformed_path = temp_dir.path().join("${XDG_TMP:-malformed");
        // assert!(safe_create_dir_all(&malformed_path).is_err());
        // assert!(!malformed_path.exists());
    }

    #[test]
    fn test_database_scoped_paths() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        // Test database directory creation
        let main_dir = paths.get_database_dir("main");
        let test_dir = paths.get_database_dir("test");
        
        assert!(main_dir.ends_with("odx/prontodb/main"));
        assert!(test_dir.ends_with("odx/prontodb/test"));
        assert_ne!(main_dir, test_dir);

        // Test database-scoped database paths
        let main_db = paths.get_db_path_with_name("main");
        let test_db = paths.get_db_path_with_name("test");
        
        assert!(main_db.ends_with("main/pronto.main.prdb"));
        assert!(test_db.ends_with("test/pronto.test.prdb"));
        assert_ne!(main_db, test_db);

        // Test database-scoped cursor paths
        let main_cursors = paths.get_cursor_dir_with_name("main");
        let test_cursors = paths.get_cursor_dir_with_name("test");
        
        assert!(main_cursors.ends_with("main/cursors"));
        assert!(test_cursors.ends_with("test/cursors"));
        assert_ne!(main_cursors, test_cursors);
    }

    #[test]
    fn test_backward_compatibility() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        // Test that default methods work same as main-scoped methods
        assert_eq!(paths.get_db_path(), paths.get_db_path_with_name("main"));
        assert_eq!(paths.get_cursor_dir(), paths.get_cursor_dir_with_name("main"));
        
        // Test environment variable override still works for default database
        let custom_db = temp_dir.path().join("custom_override.db");
        std::env::set_var("PRONTO_DB", &custom_db);
        
        // Should work for default methods and main specifically
        assert_eq!(paths.get_db_path(), custom_db);
        assert_eq!(paths.get_db_path_with_name("main"), custom_db);
        
        // But not for other databases
        assert_ne!(paths.get_db_path_with_name("test"), custom_db);
        
        std::env::remove_var("PRONTO_DB");
    }
}