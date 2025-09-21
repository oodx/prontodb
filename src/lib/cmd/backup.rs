// // ProntoDB Backup Command Implementation
// // Comprehensive backup with database and cursor files in tar.gz format

// use crate::xdg::XdgPaths;
// use rsb::prelude::*;
// use std::fs;
// use std::io;
// use std::path::{Path, PathBuf};
// use std::process::Command;
// use chrono::{DateTime, Utc};

// /// Get database paths consistently for backup and restore operations
// /// This ensures both operations use identical path resolution to prevent
// /// backup/restore path mismatches (fixes potential odx directory inconsistencies)
// fn get_database_paths(database: &str) -> (PathBuf, PathBuf) {
//     let paths = XdgPaths::new();
//     let database_dir = paths.get_database_dir(database);
//     let database_parent = database_dir.parent()
//         .expect("Database directory must have a parent directory")
//         .to_path_buf();
    
//     debug!("Database paths for '{}': dir={}, parent={}", 
//            database, database_dir.display(), database_parent.display());
    
//     (database_dir, database_parent)
// }

// /// Backup result containing backup file path and basic info
// #[derive(Debug)]
// pub struct BackupResult {
//     pub file_path: PathBuf,
//     pub size_bytes: u64,
//     pub db_name: String,
//     pub date_str: String,
//     pub increment: Option<u32>,
//     pub files_included: Vec<String>,
// }

// /// Backup error types for better error handling
// #[derive(Debug)]
// pub enum BackupError {
//     IoError(io::Error),
//     ValidationError(String),
//     DatabaseNotFound(String),
// }

// impl std::fmt::Display for BackupError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             BackupError::IoError(e) => write!(f, "IO Error: {}", e),
//             BackupError::ValidationError(e) => write!(f, "Validation Error: {}", e),
//             BackupError::DatabaseNotFound(e) => write!(f, "Database Not Found: {}", e),
//         }
//     }
// }

// impl From<io::Error> for BackupError {
//     fn from(error: io::Error) -> Self {
//         BackupError::IoError(error)
//     }
// }

// /// Backup configuration for database backup operations  
// pub struct BackupConfig<'a> {
//     pub output_path: Option<&'a str>,
//     pub database: &'a str,  // SP-4: Database name for scoped backups
// }

// /// Create a comprehensive backup using database-scoped architecture (SP-4 simplification)
// pub fn backup_database(config: BackupConfig, database: &str) -> Result<BackupResult, BackupError> {
//     // Use consistent path resolution helper
//     let (database_dir, _database_parent) = get_database_paths(database);
    
//     // Check if source database directory exists
//     if !database_dir.exists() {
//         return Err(BackupError::DatabaseNotFound(format!("Database directory not found: {}", database_dir.display())));
//     }
    
//     // For backward compatibility, also check if database has any content
//     let paths = XdgPaths::new();
//     let db_file = paths.get_db_path_with_name(database);
//     let cursor_dir = paths.get_cursor_dir_with_name(database);
    
//     if !db_file.exists() && !cursor_dir.exists() {
//         return Err(BackupError::DatabaseNotFound(format!("Database '{}' has no data files", database)));
//     }

//     // Determine backup directory
//     let backup_dir = if let Some(output) = config.output_path {
//         PathBuf::from(output)
//     } else {
//         // Default to zindex/cache/backup
//         let home = std::env::var("HOME")
//             .map_err(|_| BackupError::ValidationError("HOME environment variable not set".to_string()))?;
//         PathBuf::from(home)
//             .join("repos")
//             .join("zindex")
//             .join("cache")
//             .join("backup")
//     };

//     // Ensure backup directory exists
//     fs::create_dir_all(&backup_dir)?;

//     // Generate backup filename with auto-increment
//     let db_name = database;
    
//     let now: DateTime<Utc> = Utc::now();
//     let date_str = now.format("%Y%m%d").to_string();
    
//     let (backup_filename, increment) = generate_unique_backup_name(&backup_dir, &db_name, &date_str)?;
//     let backup_path = backup_dir.join(&backup_filename);

//     // SP-4: Simple directory-based backup - tar the entire database directory
//     let files_included = collect_backup_files(&database_dir)?;
    
//     // Create tar.gz backup using simplified approach
//     create_tar_gz_backup(&backup_path, &database_dir, database)?;
    
//     let metadata = fs::metadata(&backup_path)?;
    
//     Ok(BackupResult {
//         file_path: backup_path,
//         size_bytes: metadata.len(),
//         db_name: db_name.to_string(),
//         date_str,
//         increment,
//         files_included,
//     })
// }

// fn generate_unique_backup_name(backup_dir: &Path, db_name: &str, date_str: &str) -> Result<(String, Option<u32>), BackupError> {
//     // Format: prontodb_dbname_date.tar.gz or prontodb_dbname_date_nXX.tar.gz
//     let base_name = format!("prontodb_{}_{}", db_name, date_str);
    
//     // Check if base name is available
//     let candidate = format!("{}.tar.gz", base_name);
//     let candidate_path = backup_dir.join(&candidate);
    
//     if !candidate_path.exists() {
//         return Ok((candidate, None));
//     }

//     // Find next available increment
//     for i in 1..=99 {
//         let candidate = format!("{}_n{:02}.tar.gz", base_name, i);
//         let candidate_path = backup_dir.join(&candidate);
        
//         if !candidate_path.exists() {
//             return Ok((candidate, Some(i)));
//         }
//     }
    
//     Err(BackupError::ValidationError("Too many backup files for this date (maximum 99)".to_string()))
// }

// /// SP-4: Simplified tar.gz backup - directly tar the database directory
// fn create_tar_gz_backup(output_path: &Path, database_dir: &Path, database_name: &str) -> Result<(), BackupError> {
//     // Get parent directory of database directory for tar -C parameter
//     let parent_dir = database_dir.parent()
//         .ok_or_else(|| BackupError::ValidationError("Database directory has no parent".to_string()))?;
    
//     // Create tar.gz using tar command - much simpler!
//     let output = Command::new("tar")
//         .arg("czf")
//         .arg(output_path)
//         .arg("-C")
//         .arg(parent_dir)
//         .arg(database_name)  // Just tar the database directory by name
//         .output()
//         .map_err(|e| BackupError::IoError(e))?;
    
//     if !output.status.success() {
//         let stderr = String::from_utf8_lossy(&output.stderr);
//         return Err(BackupError::ValidationError(format!("tar command failed: {}", stderr)));
//     }
    
//     Ok(())
// }

// /// Collect list of files that will be included in backup (for reporting)
// fn collect_backup_files(database_dir: &Path) -> Result<Vec<String>, BackupError> {
//     let mut files_included = Vec::new();
    
//     if !database_dir.exists() {
//         return Ok(files_included);
//     }
    
//     // Walk the database directory to report what will be backed up
//     for entry in fs::read_dir(database_dir)? {
//         let entry = entry?;
//         let path = entry.path();
//         let name = path.file_name()
//             .unwrap_or_else(|| std::ffi::OsStr::new("unknown"))
//             .to_string_lossy();
        
//         if path.is_file() {
//             files_included.push(name.to_string());
//         } else if path.is_dir() {
//             // Count files in subdirectories (like cursors/)
//             if let Ok(subdir_entries) = fs::read_dir(&path) {
//                 let mut subdir_count = 0;
//                 for sub_entry in subdir_entries {
//                     if let Ok(sub_entry) = sub_entry {
//                         if sub_entry.path().is_file() {
//                             subdir_count += 1;
//                         }
//                     }
//                 }
//                 if subdir_count > 0 {
//                     files_included.push(format!("{}/({} files)", name, subdir_count));
//                 }
//             }
//         }
//     }
    
//     Ok(files_included)
// }

// /// List available backup files in a directory
// pub fn list_backups(search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError> {
//     let search_path = if let Some(dir) = search_dir {
//         dir.to_path_buf()
//     } else {
//         // Default to zindex/cache/backup
//         let home = std::env::var("HOME")
//             .map_err(|_| BackupError::ValidationError("HOME environment variable not set".to_string()))?;
//         PathBuf::from(home)
//             .join("repos")
//             .join("zindex")
//             .join("cache")
//             .join("backup")
//     };

//     if !search_path.exists() {
//         return Ok(Vec::new());
//     }

//     let mut backups = Vec::new();
//     for entry in fs::read_dir(search_path)? {
//         let entry = entry?;
//         let path = entry.path();
        
//         if let Some(filename) = path.file_name() {
//             let filename_str = filename.to_string_lossy();
//             if filename_str.starts_with("prontodb_") && (filename_str.ends_with(".tar.gz") || filename_str.ends_with(".db")) {
//                 backups.push(path);
//             }
//         }
//     }

//     backups.sort();
//     Ok(backups)
// }

// /// SP-4: Restore from a backup file using database-scoped architecture
// pub fn restore_backup(backup_path: &Path, config: BackupConfig) -> Result<(), BackupError> {
//     if !backup_path.exists() {
//         return Err(BackupError::DatabaseNotFound(format!("Backup file not found: {}", backup_path.display())));
//     }

//     let paths = XdgPaths::new();
//     let backup_str = backup_path.to_string_lossy();
    
//     // Handle legacy .db files (simple copy to database-scoped location)
//     if backup_str.ends_with(".db") {
//         let target_path = paths.get_db_path_with_name(config.database);
        
//         if let Some(parent) = target_path.parent() {
//             fs::create_dir_all(parent)?;
//         }
//         fs::copy(backup_path, target_path)?;
//         return Ok(());
//     }
    
//     // SP-4: Handle tar.gz files - extract directly to database directory
//     if backup_str.ends_with(".tar.gz") {
//         // Use consistent path resolution helper (same as backup)
//         let (_database_dir, database_parent) = get_database_paths(config.database);
        
//         // Ensure parent directory exists
//         fs::create_dir_all(&database_parent)?;
        
//         // Extract tar.gz directly to database parent directory
//         let output = Command::new("tar")
//             .arg("xzf")
//             .arg(backup_path)
//             .arg("-C")
//             .arg(&database_parent)
//             .output()
//             .map_err(|e| BackupError::IoError(e))?;
        
//         if !output.status.success() {
//             let stderr = String::from_utf8_lossy(&output.stderr);
//             return Err(BackupError::ValidationError(format!("tar extraction failed: {}", stderr)));
//         }
        
//         return Ok(());
//     }
    
//     Err(BackupError::ValidationError(format!("Unknown backup format: {}", backup_str)))
// }

// // Command interface function for RSB pre-dispatcher
// pub fn handle_backup_command(args: rsb::args::Args) -> i32 {
//     use std::path::PathBuf;
    
//     // Parse backup options
//     let mut output_dir: Option<String> = None;
//     let mut list_backup_files = false;
//     let mut restore_file = None;
//     let mut quiet = false;
//     let mut cursor_name: Option<String> = None;  // Deprecated - will map to database
//     let mut _user = "default".to_string();
//     let mut database = "main".to_string();  // SP-4: Database name support
    
//     let arg_list = args.all();
//     let mut i = 0;
//     while i < arg_list.len() {
//         match arg_list[i].as_str() {
//             "--output" | "-o" if i + 1 < arg_list.len() => {
//                 output_dir = Some(arg_list[i + 1].clone());
//                 i += 2;
//             }
//             "--list" | "-l" => {
//                 list_backup_files = true;
//                 i += 1;
//             }
//             "--restore" | "-r" if i + 1 < arg_list.len() => {
//                 restore_file = Some(PathBuf::from(&arg_list[i + 1]));
//                 i += 2;
//             }
//             "--cursor" if i + 1 < arg_list.len() => {
//                 cursor_name = Some(arg_list[i + 1].clone());
//                 i += 2;
//             }
//             "--database" if i + 1 < arg_list.len() => {
//                 database = arg_list[i + 1].clone();
//                 i += 2;
//             }
//             "--user" if i + 1 < arg_list.len() => {
//                 _user = arg_list[i + 1].clone();
//                 i += 2;
//             }
//             "--quiet" | "-q" => {
//                 quiet = true;
//                 i += 1;
//             }
//             "--help" | "-h" => {
//                 println!("prontodb backup - Simple database backup with auto-incrementing filenames");
//                 println!();
//                 println!("USAGE:");
//                 println!("  prontodb backup [OPTIONS]");
//                 println!();
//                 println!("OPTIONS:");
//                 println!("  -o, --output <DIR>     Output directory for backup (default: ~/repos/zindex/cache/backup)");
//                 println!("  -l, --list             List existing backup files");
//                 println!("  -r, --restore <FILE>   Restore from backup file");
//                 println!("  --cursor <NAME>        Use specific cursor database (deprecated - use --database)");
//                 println!("  --database <NAME>      Backup specific database (default: 'main')");
//                 println!("  --user <USER>          Use specific user context (default: 'default')");
//                 println!("  -q, --quiet            Suppress output messages");
//                 println!("  -h, --help             Show this help message");
//                 println!();
//                 println!("FILENAME FORMAT:");
//                 println!("  prontodb_<dbname>_<YYYYMMDD>.tar.gz         # First backup of the day");
//                 println!("  prontodb_<dbname>_<YYYYMMDD>_n01.tar.gz     # Second backup of the day");
//                 println!("  prontodb_<dbname>_<YYYYMMDD>_n02.tar.gz     # Third backup of the day");
//                 println!();
//                 println!("BACKUP CONTENTS:");
//                 println!("  - Database file (pronto.main.prdb)");
//                 println!("  - All cursor files (cursors/*.cursor)");
//                 println!("  - Compressed tar.gz format for safety");
//                 println!();
//                 println!("EXAMPLES:");
//                 println!("  prontodb backup                                    # Create backup with default settings");
//                 println!("  prontodb backup --output /path/to/backups          # Create backup in specific directory");
//                 println!("  prontodb backup --database prod                     # Backup specific database");
//                 println!("  prontodb backup --cursor prod --user alice         # Backup specific user's cursor database (deprecated)");
//                 println!("  prontodb backup --list                             # List existing backups");
//                 println!("  prontodb backup --restore prontodb_pronto_20250909.tar.gz  # Restore from backup");
//                 return 0;
//             }
//             _ => {
//                 eprintln!("backup: Unknown option '{}'", arg_list[i]);
//                 eprintln!("Use 'prontodb backup --help' for usage information");
//                 return 1;
//             }
//         }
//     }
    
//     // Handle list operation
//     if list_backup_files {
//         let search_dir = output_dir.as_deref().map(Path::new);
//         match list_backups(search_dir) {
//             Ok(backups) => {
//                 if backups.is_empty() {
//                     if !quiet {
//                         println!("No backup files found");
//                     }
//                 } else {
//                     if !quiet {
//                         println!("Found {} backup file(s):", backups.len());
//                     }
//                     for backup in backups {
//                         let metadata = std::fs::metadata(&backup)
//                             .map(|m| format!("{} bytes", m.len()))
//                             .unwrap_or_else(|_| "unknown size".to_string());
                        
//                         let modified = std::fs::metadata(&backup)
//                             .and_then(|m| m.modified())
//                             .map(|t| t.duration_since(std::time::UNIX_EPOCH)
//                                      .map(|d| d.as_secs())
//                                      .unwrap_or(0))
//                             .map(|s| chrono::DateTime::from_timestamp(s as i64, 0)
//                                      .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
//                                      .unwrap_or_else(|| "unknown".to_string()))
//                             .unwrap_or_else(|_| "unknown".to_string());
                            
//                         println!("  {} ({}, {})", backup.display(), metadata, modified);
//                     }
//                 }
//                 return 0;
//             }
//             Err(e) => {
//                 eprintln!("backup: Failed to list backups: {}", e);
//                 return 1;
//             }
//         }
//     }
    
//     // Handle restore operation
//     if let Some(restore_path) = restore_file {
//         if !quiet {
//             println!("Restoring from backup: {}", restore_path.display());
//         }
        
//         // SP-4: Map cursor_name to database for backward compatibility
//         let effective_database = if let Some(cursor) = &cursor_name {
//             cursor  // Use cursor name as database name for backward compatibility
//         } else {
//             &database
//         };
        
//         let config = BackupConfig {
//             output_path: output_dir.as_deref(),
//             database: effective_database,
//         };
        
//         match restore_backup(&restore_path, config) {
//             Ok(_) => {
//                 if !quiet {
//                     println!("Backup restored successfully!");
//                 }
//                 return 0;
//             }
//             Err(e) => {
//                 eprintln!("backup: Failed to restore backup: {}", e);
//                 return 1;
//             }
//         }
//     }
    
//     // Default: create backup
//     if !quiet {
//         println!("Creating backup...");
//     }
    
//     // SP-4: Map cursor_name to database for backward compatibility
//     let effective_database = if let Some(cursor) = &cursor_name {
//         cursor  // Use cursor name as database name for backward compatibility
//     } else {
//         &database
//     };
    
//     let config = BackupConfig {
//         output_path: output_dir.as_deref(),
//         database: effective_database,
//     };
    
//     match backup_database(config, effective_database) {
//         Ok(result) => {
//             if !quiet {
//                 println!("Backup created successfully!");
//                 println!("File: {}", result.file_path.display());
//                 println!("Database: {}", result.db_name);
//                 let paths = XdgPaths::new();
//                 println!("Database directory: {}", paths.get_database_dir(effective_database).display());
//                 println!("Date: {}", result.date_str);
//                 if let Some(increment) = result.increment {
//                     println!("Increment: n{:02}", increment);
//                 } else {
//                     println!("Increment: none (first backup today)");
//                 }
//                 println!("Size: {} bytes", result.size_bytes);
//                 println!("Files included: {}", result.files_included.len());
//                 for file in &result.files_included {
//                     println!("  - {}", file);
//                 }
//             }
//             0
//         }
//         Err(e) => {
//             eprintln!("backup: Failed to create backup: {}", e);
//             1
//         }
//     }
// }
