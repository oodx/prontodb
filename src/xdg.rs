// XDG+ path management for ProntoDB
// Provides proper path isolation for testing and multi-instance setups

use std::path::{Path, PathBuf};
use std::env;

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

        let db_path = data_dir.join("pronto.db");
        let config_file = config_dir.join("pronto.conf");

        XdgPaths {
            home: home.to_path_buf(),
            data_dir,
            config_dir,
            cache_dir,
            runtime_dir,
            db_path,
            config_file,
        }
    }

    /// Create all necessary directories
    pub fn ensure_dirs(&self) -> Result<(), std::io::Error> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.config_dir)?;
        std::fs::create_dir_all(&self.cache_dir)?;
        
        if let Some(runtime_dir) = &self.runtime_dir {
            std::fs::create_dir_all(runtime_dir)?;
        }
        
        Ok(())
    }

    /// Get effective database path (supports PRONTO_DB override)
    pub fn get_db_path(&self) -> PathBuf {
        if let Ok(db_path) = env::var("PRONTO_DB") {
            PathBuf::from(db_path)
        } else {
            self.db_path.clone()
        }
    }

    /// Get effective config file path (supports PRONTO_CONFIG override)
    pub fn get_config_path(&self) -> PathBuf {
        if let Ok(config_path) = env::var("PRONTO_CONFIG") {
            PathBuf::from(config_path)
        } else {
            self.config_file.clone()
        }
    }

    // Private helper methods

    fn get_home_dir() -> PathBuf {
        if let Ok(home) = env::var("HOME") {
            PathBuf::from(home)
        } else if let Ok(userprofile) = env::var("USERPROFILE") {
            // Windows fallback
            PathBuf::from(userprofile)
        } else {
            // Ultimate fallback
            PathBuf::from("/tmp")
        }
    }

    fn get_data_dir(home: &Path) -> PathBuf {
        if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
            PathBuf::from(xdg_data_home).join("odx").join("prontodb")
        } else {
            home.join(".local").join("data").join("odx").join("prontodb")
        }
    }

    fn get_config_dir(home: &Path) -> PathBuf {
        if let Ok(xdg_config_home) = env::var("XDG_CONFIG_HOME") {
            PathBuf::from(xdg_config_home).join("odx").join("prontodb")
        } else {
            home.join(".local").join("etc").join("odx").join("prontodb")
        }
    }

    fn get_cache_dir(home: &Path) -> PathBuf {
        if let Ok(xdg_cache_home) = env::var("XDG_CACHE_HOME") {
            PathBuf::from(xdg_cache_home).join("odx").join("prontodb")
        } else {
            home.join(".cache").join("odx").join("prontodb")
        }
    }

    fn get_runtime_dir() -> Option<PathBuf> {
        env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(|dir| PathBuf::from(dir).join("odx").join("prontodb"))
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
            let paths = XdgPaths::from_home(temp_dir.path());
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
        
        // Verify files are in correct locations
        assert!(paths.db_path.ends_with("pronto.db"));
        assert!(paths.config_file.ends_with("pronto.conf"));
    }

    #[test]
    fn test_ensure_dirs_creates_structure() {
        let temp_dir = TempDir::new().unwrap();
        let paths = XdgPaths::from_home(temp_dir.path());

        paths.ensure_dirs().unwrap();

        assert!(paths.data_dir.exists());
        assert!(paths.config_dir.exists());
        assert!(paths.cache_dir.exists());
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
        assert_eq!(paths.get_db_path(), paths.db_path);
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
}