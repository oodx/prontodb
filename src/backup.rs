// ProntoDB Backup/Restore Module
// Handles lifecycle backup and restore operations with comprehensive error handling

use crate::xdg::XdgPaths;
use rsb::prelude::*;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Backup result containing backup file path and metadata
#[derive(Debug)]
pub struct BackupResult {
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub timestamp: String,
    pub contents: Vec<String>,
}

/// Backup error types for better error handling
#[derive(Debug)]
pub enum BackupError {
    IoError(io::Error),
    CompressionError(String),
    ValidationError(String),
    #[allow(dead_code)]  // Reserved for future permission checks
    PermissionError(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BackupError::IoError(e) => write!(f, "IO Error: {}", e),
            BackupError::CompressionError(e) => write!(f, "Compression Error: {}", e),
            BackupError::ValidationError(e) => write!(f, "Validation Error: {}", e),
            BackupError::PermissionError(e) => write!(f, "Permission Error: {}", e),
        }
    }
}

impl From<io::Error> for BackupError {
    fn from(error: io::Error) -> Self {
        BackupError::IoError(error)
    }
}

/// Backup manager for ProntoDB lifecycle operations
pub struct BackupManager {
    paths: XdgPaths,
}

impl BackupManager {
    /// Create new backup manager
    pub fn new() -> Self {
        Self {
            paths: XdgPaths::new(),
        }
    }

    /// Create timestamped backup of all ProntoDB data
    pub fn create_backup(&self, output_dir: Option<&Path>) -> Result<BackupResult, BackupError> {
        let timestamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
        let backup_filename = format!("prontodb-backup-{}.tar.gz", timestamp);
        
        let output_path = if let Some(dir) = output_dir {
            dir.join(&backup_filename)
        } else {
            std::env::current_dir()
                .map_err(BackupError::IoError)?
                .join(&backup_filename)
        };

        info!("Creating backup at: {}", output_path.display());

        // Ensure all directories exist before backup
        self.paths.ensure_dirs().map_err(BackupError::IoError)?;

        // Create temporary staging directory
        let temp_dir = tempfile::tempdir().map_err(BackupError::IoError)?;
        let staging_path = temp_dir.path().join("prontodb-backup");
        fs::create_dir_all(&staging_path).map_err(BackupError::IoError)?;

        let mut backup_contents = Vec::new();

        // Copy database files
        let db_staging = staging_path.join("database");
        fs::create_dir_all(&db_staging).map_err(BackupError::IoError)?;
        
        let db_path = self.paths.get_db_path();
        if db_path.exists() {
            let db_filename = db_path.file_name()
                .ok_or_else(|| BackupError::ValidationError("Invalid database path".to_string()))?;
            let dest_db = db_staging.join(db_filename);
            fs::copy(&db_path, &dest_db).map_err(BackupError::IoError)?;
            backup_contents.push(format!("database/{}", db_filename.to_string_lossy()));
            info!("Backed up database: {}", db_path.display());
        }

        // Copy cursor directory if it exists
        if self.paths.cursor_dir.exists() {
            let cursor_staging = staging_path.join("cursors");
            self.copy_directory(&self.paths.cursor_dir, &cursor_staging)?;
            
            // List cursor files
            if let Ok(entries) = fs::read_dir(&self.paths.cursor_dir) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        backup_contents.push(format!("cursors/{}", filename));
                    }
                }
            }
            info!("Backed up cursors: {}", self.paths.cursor_dir.display());
        }

        // Copy config files if they exist
        let config_staging = staging_path.join("config");
        fs::create_dir_all(&config_staging).map_err(BackupError::IoError)?;
        
        let config_path = self.paths.get_config_path();
        if config_path.exists() {
            let config_filename = config_path.file_name()
                .ok_or_else(|| BackupError::ValidationError("Invalid config path".to_string()))?;
            let dest_config = config_staging.join(config_filename);
            fs::copy(&config_path, &dest_config).map_err(BackupError::IoError)?;
            backup_contents.push(format!("config/{}", config_filename.to_string_lossy()));
            info!("Backed up config: {}", config_path.display());
        }

        // Copy any additional config files in config directory
        if self.paths.config_dir.exists() {
            if let Ok(entries) = fs::read_dir(&self.paths.config_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_file() && path != config_path {
                        if let Some(filename) = path.file_name() {
                            let dest = config_staging.join(filename);
                            fs::copy(&path, &dest).map_err(BackupError::IoError)?;
                            backup_contents.push(format!("config/{}", filename.to_string_lossy()));
                        }
                    }
                }
            }
        }

        // Create backup metadata
        let metadata = self.create_backup_metadata(&timestamp, &backup_contents)?;
        let metadata_path = staging_path.join("backup-metadata.json");
        fs::write(&metadata_path, metadata).map_err(BackupError::IoError)?;
        backup_contents.push("backup-metadata.json".to_string());

        // Create tar.gz archive
        self.create_archive(&staging_path, &output_path)?;

        // Get final file size
        let metadata = fs::metadata(&output_path).map_err(BackupError::IoError)?;
        let size_bytes = metadata.len();

        info!("Backup created successfully: {} ({} bytes)", output_path.display(), size_bytes);

        Ok(BackupResult {
            file_path: output_path,
            size_bytes,
            timestamp,
            contents: backup_contents,
        })
    }

    /// Restore from backup file
    pub fn restore_backup(&self, backup_path: &Path) -> Result<(), BackupError> {
        if !backup_path.exists() {
            return Err(BackupError::ValidationError(
                format!("Backup file not found: {}", backup_path.display())
            ));
        }

        info!("Restoring from backup: {}", backup_path.display());

        // Create temporary extraction directory
        let temp_dir = tempfile::tempdir().map_err(BackupError::IoError)?;
        let extract_path = temp_dir.path().join("restore");

        // Extract archive
        self.extract_archive(backup_path, &extract_path)?;

        // Validate backup structure
        let metadata_path = extract_path.join("backup-metadata.json");
        if !metadata_path.exists() {
            return Err(BackupError::ValidationError(
                "Invalid backup: missing metadata".to_string()
            ));
        }

        // Ensure destination directories exist
        self.paths.ensure_dirs().map_err(BackupError::IoError)?;

        // Restore database
        let db_source = extract_path.join("database");
        if db_source.exists() {
            if let Ok(entries) = fs::read_dir(&db_source) {
                for entry in entries.flatten() {
                    let source = entry.path();
                    if source.is_file() {
                        let dest = self.paths.data_dir.join(entry.file_name());
                        fs::copy(&source, &dest).map_err(BackupError::IoError)?;
                        info!("Restored database file: {}", dest.display());
                    }
                }
            }
        }

        // Restore cursors
        let cursor_source = extract_path.join("cursors");
        if cursor_source.exists() {
            self.copy_directory(&cursor_source, &self.paths.cursor_dir)?;
            info!("Restored cursors to: {}", self.paths.cursor_dir.display());
        }

        // Restore config
        let config_source = extract_path.join("config");
        if config_source.exists() {
            if let Ok(entries) = fs::read_dir(&config_source) {
                for entry in entries.flatten() {
                    let source = entry.path();
                    if source.is_file() {
                        let dest = self.paths.config_dir.join(entry.file_name());
                        fs::copy(&source, &dest).map_err(BackupError::IoError)?;
                        info!("Restored config file: {}", dest.display());
                    }
                }
            }
        }

        info!("Backup restored successfully");
        Ok(())
    }

    /// List existing backup files in directory
    pub fn list_backups(&self, search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError> {
        let dir = search_dir.unwrap_or_else(|| Path::new("."));
        let mut backups = Vec::new();

        if dir.exists() && dir.is_dir() {
            for entry in fs::read_dir(dir).map_err(BackupError::IoError)? {
                let entry = entry.map_err(BackupError::IoError)?;
                let path = entry.path();
                
                if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                    if filename.starts_with("prontodb-backup-") && filename.ends_with(".tar.gz") {
                        backups.push(path);
                    }
                }
            }
        }

        // Sort by filename (which includes timestamp)
        backups.sort();
        Ok(backups)
    }

    // Private helper methods

    fn copy_directory(&self, source: &Path, dest: &Path) -> Result<(), BackupError> {
        fs::create_dir_all(dest).map_err(BackupError::IoError)?;
        
        for entry in fs::read_dir(source).map_err(BackupError::IoError)? {
            let entry = entry.map_err(BackupError::IoError)?;
            let source_path = entry.path();
            let dest_path = dest.join(entry.file_name());
            
            if source_path.is_file() {
                fs::copy(&source_path, &dest_path).map_err(BackupError::IoError)?;
            } else if source_path.is_dir() {
                self.copy_directory(&source_path, &dest_path)?;
            }
        }
        
        Ok(())
    }

    fn create_backup_metadata(&self, timestamp: &str, contents: &[String]) -> Result<String, BackupError> {
        let metadata = serde_json::json!({
            "version": "0.1.0",
            "timestamp": timestamp,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "prontodb_version": env!("CARGO_PKG_VERSION"),
            "contents": contents,
            "paths": {
                "data_dir": self.paths.data_dir.to_string_lossy(),
                "config_dir": self.paths.config_dir.to_string_lossy(),
                "cursor_dir": self.paths.cursor_dir.to_string_lossy(),
                "db_path": self.paths.get_db_path().to_string_lossy(),
                "config_path": self.paths.get_config_path().to_string_lossy()
            }
        });

        serde_json::to_string_pretty(&metadata)
            .map_err(|e| BackupError::ValidationError(format!("Failed to create metadata: {}", e)))
    }

    fn create_archive(&self, source_dir: &Path, output_path: &Path) -> Result<(), BackupError> {
        // Use tar command for maximum compatibility
        let output = Command::new("tar")
            .args([
                "-czf",
                output_path.to_str().ok_or_else(|| 
                    BackupError::ValidationError("Invalid output path".to_string())
                )?,
                "-C",
                source_dir.to_str().ok_or_else(|| 
                    BackupError::ValidationError("Invalid source path".to_string())
                )?,
                "."
            ])
            .output()
            .map_err(|e| BackupError::CompressionError(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::CompressionError(
                format!("tar command failed: {}", error_msg)
            ));
        }

        Ok(())
    }

    fn extract_archive(&self, archive_path: &Path, extract_dir: &Path) -> Result<(), BackupError> {
        fs::create_dir_all(extract_dir).map_err(BackupError::IoError)?;

        let output = Command::new("tar")
            .args([
                "-xzf",
                archive_path.to_str().ok_or_else(|| 
                    BackupError::ValidationError("Invalid archive path".to_string())
                )?,
                "-C",
                extract_dir.to_str().ok_or_else(|| 
                    BackupError::ValidationError("Invalid extract path".to_string())
                )?
            ])
            .output()
            .map_err(|e| BackupError::CompressionError(format!("Failed to run tar: {}", e)))?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(BackupError::CompressionError(
                format!("tar extraction failed: {}", error_msg)
            ));
        }

        Ok(())
    }
}

impl Default for BackupManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_backup_manager_creation() {
        let manager = BackupManager::new();
        assert!(manager.paths.data_dir.to_string_lossy().contains("prontodb"));
    }

    #[test]
    fn test_backup_and_restore_cycle() {
        let temp_dir = TempDir::new().unwrap();
        let backup_dir = temp_dir.path().join("backups");
        fs::create_dir_all(&backup_dir).unwrap();

        // Create a test environment
        let test_home = temp_dir.path().join("test_home");
        let paths = XdgPaths::from_home_isolated(&test_home);
        paths.ensure_dirs().unwrap();

        // Create some test data
        fs::write(&paths.db_path, "test database content").unwrap();
        fs::write(&paths.cursor_dir.join("test.cursor"), "test cursor").unwrap();

        let manager = BackupManager { paths: paths.clone() };

        // Create backup
        let backup_result = manager.create_backup(Some(&backup_dir)).unwrap();
        assert!(backup_result.file_path.exists());
        assert!(backup_result.size_bytes > 0);
        assert!(!backup_result.contents.is_empty());

        // Clean up test environment
        fs::remove_dir_all(&test_home).unwrap();

        // Restore backup
        manager.restore_backup(&backup_result.file_path).unwrap();

        // Verify restored data
        assert!(paths.db_path.exists());
        assert!(paths.cursor_dir.join("test.cursor").exists());
        
        let restored_db = fs::read_to_string(&paths.db_path).unwrap();
        assert_eq!(restored_db, "test database content");
    }

    #[test]
    fn test_list_backups() {
        let temp_dir = TempDir::new().unwrap();
        let manager = BackupManager::new();

        // Create some fake backup files
        let backup1 = temp_dir.path().join("prontodb-backup-20241201-120000.tar.gz");
        let backup2 = temp_dir.path().join("prontodb-backup-20241202-130000.tar.gz");
        let not_backup = temp_dir.path().join("other-file.txt");

        fs::write(&backup1, "fake backup 1").unwrap();
        fs::write(&backup2, "fake backup 2").unwrap();
        fs::write(&not_backup, "not a backup").unwrap();

        let backups = manager.list_backups(Some(temp_dir.path())).unwrap();
        assert_eq!(backups.len(), 2);
        assert!(backups.contains(&backup1));
        assert!(backups.contains(&backup2));
        assert!(!backups.iter().any(|p| p == &not_backup));
    }
}