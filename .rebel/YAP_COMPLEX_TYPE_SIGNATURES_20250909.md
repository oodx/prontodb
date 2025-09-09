# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/commands/backup.rs
**Violation Type**: Complex type signatures violating string-biased philosophy

## VIOLATION DETECTED 🚨

**Complex public API signatures**: Several public functions use complex Rust types instead of RSB's string-first interfaces:

```rust
// Lines 14-20: Complex struct in public API
pub struct BackupResult {
    pub file_path: PathBuf,          // Should be String
    pub size_bytes: u64,             // Should be String  
    pub db_name: String,             // ✅ Already string
    pub date_str: String,            // ✅ Already string
    pub increment: Option<u32>,      // Should be String
}

// Lines 47-51: Complex struct with lifetime and Option types
pub struct BackupConfig<'a> {
    pub output_path: Option<&'a str>,  // Complex Option type
    pub cursor_name: Option<&'a str>,  // Complex Option type  
    pub user: &'a str,                 // Lifetime complexity
}

// Line 54: Complex Result type with custom error enum
pub fn backup_database(config: BackupConfig) -> Result<BackupResult, BackupError>

// Line 146: Complex Result with Vec and custom error
pub fn list_backups(search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError>
```

## CANONICAL RSB PATTERN 📚

From rsb-architecture.md Section 1.2:
> "RSB is opinionated about using strings as the primary interface type, hiding Rust's type complexity behind familiar operations"

> "✅ RSB Pattern: String-biased signatures
> pub fn read_config(path: &str) -> String;
> pub fn process_logs(input: &str, pattern: &str) -> String;
> pub fn send_alert(message: &str, recipient: &str) -> i32;"

> "❌ Anti-Pattern: Complex type signatures  
> pub fn process<T, E>(input: Result<Option<T>, E>) -> Result<Vec<Config>, ProcessError>"

## CORRECTIVE ACTION ⚡

Replace complex types with string-biased RSB interfaces:

```rust
// ✅ RSB-compliant string-first backup API:

// Replace complex BackupResult with string output
pub fn do_backup(args: rsb::args::Args) -> i32 {
    // String-based configuration via RSB param! macro
    let output_path = param!("BACKUP_OUTPUT", default: "");
    let cursor_name = param!("BACKUP_CURSOR", default: "");
    let user = param!("BACKUP_USER", default: "default");
    
    // Business logic in helper function
    let backup_info = _create_backup(&output_path, &cursor_name, &user);
    
    if backup_info.is_empty() {
        error!("Backup failed");
        1
    } else {
        // String-based output for user
        echo!("{}", backup_info);  // RSB echo to stdout
        0
    }
}

// Helper function with string-first interface
fn _create_backup(output_path: &str, cursor_name: &str, user: &str) -> String {
    // Return string description instead of complex struct
    let backup_path = __perform_backup_copy(output_path, cursor_name, user);
    
    if backup_path.is_empty() {
        return String::new();  // Error case
    }
    
    // Simple string format for backup info
    format!("BACKUP_SUCCESS: path={} user={} timestamp={}", 
            backup_path, user, date!())
}

// List backups with string output
pub fn do_list_backups(args: rsb::args::Args) -> i32 {
    let search_dir = args.get_or(1, "");
    let backup_list = _find_backup_files(&search_dir);
    
    if backup_list.is_empty() {
        echo!("No backups found");
    } else {
        echo!("{}", backup_list);  // String output, one per line
    }
    0
}

fn _find_backup_files(search_dir: &str) -> String {
    // Return newline-separated string instead of Vec<PathBuf>
    let backup_dir = if search_dir.is_empty() {
        format!("{}/repos/zindex/cache/backup", home_dir!())
    } else {
        search_dir.to_string()
    };
    
    // Use RSB cat! or similar for directory listing
    run!(format!("find {} -name 'prontodb_*.db' | sort", backup_dir))
}

// Restore with string interface
pub fn do_restore_backup(args: rsb::args::Args) -> i32 {
    let backup_file = args.get_or(1, "");
    validate!(!backup_file.is_empty(), "Backup file path required");
    
    let cursor_name = param!("RESTORE_CURSOR", default: "");
    let user = param!("RESTORE_USER", default: "default");
    
    let result = _restore_from_file(&backup_file, &cursor_name, &user);
    if result {
        okay!("Backup restored successfully");
        0
    } else {
        error!("Restore failed");
        1
    }
}
```

## REFERENCE 📖
- RSB Architecture Section 1.2: String-Biased Philosophy
- RSB Architecture Section 1.3: "Why Strings/Streams Over Types"
- RSB patterns: String-first interfaces hide Rust complexity