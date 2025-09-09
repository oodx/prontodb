# 🦊 RSB VIOLATION YAP - LIFECYCLE MODULES
**Date**: 2025-09-09
**Target**: ProntoDB Lifecycle Modules (backup.rs, main.rs lifecycle commands, cursor.rs)
**Violation Type**: Multiple Critical RSB Architecture Violations

## VIOLATIONS DETECTED 🚨

### 1. CRITICAL: Missing RSB Macros for System Operations

**File**: src/backup.rs, src/main.rs
**Violation**: Direct `std::process::Command` usage instead of RSB `run!()` or `shell!()` macros

```rust
// ❌ VIOLATION: Direct Command usage in backup.rs
use std::process::Command;

let output = Command::new("tar")
    .args(["-czf", output_path.to_str()?, "-C", source_dir.to_str()?, "."])
    .output()
    .map_err(|e| BackupError::CompressionError(format!("Failed to run tar: {}", e)))?;

// ❌ VIOLATION: Direct current_exe() usage in main.rs  
let current_exe = match std::env::current_exe() {
    Ok(path) => path,
    Err(e) => {
        eprintln!("install: Failed to get current executable path: {}", e);
        return 1;
    }
};
```

### 2. CRITICAL: Manual std::env Usage Instead of RSB Patterns

**File**: src/main.rs, src/xdg.rs, src/storage.rs
**Violation**: Direct `std::env::var()` calls instead of RSB `param!()` macro

```rust
// ❌ VIOLATION: Manual env var access in main.rs
let raw_args: Vec<String> = std::env::args().collect();

// ❌ VIOLATION: Manual env var checking in xdg.rs
if let Ok(runtime_db) = std::env::var("PRONTO_DB") {
    if !runtime_db.is_empty() {
        return PathBuf::from(runtime_db);
    }
}

// ❌ VIOLATION: Manual env access in storage.rs
if let Ok(path) = std::env::var("PRONTO_DB") {
    PathBuf::from(path)
} else if let Ok(home) = std::env::var("HOME") {
    PathBuf::from(home)
```

### 3. CRITICAL: Direct std::fs Usage Instead of RSB File Operations

**File**: src/backup.rs, src/main.rs, src/cursor.rs
**Violation**: Manual filesystem operations instead of RSB file handling patterns

```rust
// ❌ VIOLATION: Direct filesystem operations in backup.rs
use std::fs;
fs::copy(&db_path, &dest_db).map_err(BackupError::IoError)?;
fs::create_dir_all(&staging_path).map_err(BackupError::IoError)?;

// ❌ VIOLATION: Direct filesystem operations in main.rs
use std::fs;
if let Err(e) = fs::create_dir_all(&install_dir) {
    eprintln!("install: Failed to create directory {}: {}", install_dir.display(), e);
    return 1;
}

// ❌ VIOLATION: Direct filesystem operations in cursor.rs
fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory");
fs::write(&cursor_file, json_content).expect("Failed to write cursor file");
```

### 4. CRITICAL: expect() and unwrap() Usage Instead of RSB Error Handling

**File**: src/cursor.rs, src/backup.rs
**Violation**: Non-RSB error handling patterns violating graceful degradation

```rust
// ❌ VIOLATION: expect() usage in cursor.rs
fs::create_dir_all(&cursor_dir).expect("Failed to create cursor directory");
let json_content = serde_json::to_string_pretty(&cursor_data)
    .expect("Failed to serialize cursor data");
fs::write(&cursor_file, json_content)
    .expect("Failed to write cursor file");

// ❌ VIOLATION: unwrap() in time operations
let created_at = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .unwrap()
    .as_secs();
```

### 5. MAJOR: Manual Error Types Instead of RSB Error Patterns

**File**: src/backup.rs
**Violation**: Custom error enums instead of RSB string-biased error handling

```rust
// ❌ VIOLATION: Complex error types instead of string-biased patterns
#[derive(Debug)]
pub enum BackupError {
    IoError(io::Error),
    CompressionError(String),
    ValidationError(String),
    PermissionError(String),
}

impl std::fmt::Display for BackupError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Complex error handling instead of RSB simplicity
    }
}
```

### 6. MAJOR: Complex Type Signatures Instead of String-Biased Patterns

**File**: src/backup.rs
**Violation**: Non-string-first function signatures violating RSB philosophy

```rust
// ❌ VIOLATION: Complex return types instead of string-biased
pub fn create_backup(&self, output_dir: Option<&Path>) -> Result<BackupResult, BackupError>
pub fn restore_backup(&self, backup_path: &Path) -> Result<(), BackupError>
pub fn list_backups(&self, search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError>

// ❌ VIOLATION: Complex struct instead of string operations
#[derive(Debug)]
pub struct BackupResult {
    pub file_path: PathBuf,
    pub size_bytes: u64,
    pub timestamp: String,
    pub contents: Vec<String>,
}
```

## CANONICAL RSB PATTERNS 📚

### Command Execution (RSB Quick Reference v2.0)
```rust
// ✅ CORRECT: RSB command execution patterns
run!("tar -czf backup.tar.gz -C /source .")     // Returns String output
let result = shell!("tar -xzf backup.tar.gz");  // Returns CmdResult for error handling
if result.status != 0 {
    error!("Archive extraction failed: {}", result.error);
    return 1;
}
```

### Environment Variables (RSB Architecture)
```rust
// ✅ CORRECT: RSB environment patterns
let db_path = param!("PRONTO_DB", "/default/path/db.sqlite");
require_var!("BACKUP_DIR");  // Validates and exits if missing
```

### File Operations (RSB String-Biased)
```rust
// ✅ CORRECT: RSB file handling
let content = cat!("config.json");
write_file("backup-metadata.json", &json_content);
if !file_exists("required-config.conf") {
    error!("Configuration file missing");
    return 1;
}
```

### Error Handling (RSB Three-Tier Pattern)
```rust
// ✅ CORRECT: RSB error handling patterns
fn do_backup(args: Args) -> i32 {
    require_var!("BACKUP_DIR");
    require_command!("tar");
    
    let source = args.get_or(1, ".");
    let dest = param!("BACKUP_DIR");
    
    validate!(is_dir(&source), "Source directory not found: {}", source);
    
    _create_backup(&source, &dest)
}

fn _create_backup(source: &str, dest: &str) -> i32 {
    let backup_name = format!("backup-{}-{}.tar.gz", hostname!(), date!(epoch));
    let backup_path = format!("{}/{}", dest, backup_name);
    
    if __archive_directory(source, &backup_path) {
        okay!("Backup created: {}", backup_name);
        0
    } else {
        error!("Backup failed");
        1
    }
}

fn __archive_directory(source: &str, dest: &str) -> bool {
    let result = shell!(&format!("tar -czf {} -C {} .", dest, source));
    result.status == 0
}
```

### String-First Function Signatures (RSB Philosophy)
```rust
// ✅ CORRECT: RSB string-biased signatures
pub fn create_backup(source_dir: &str, output_dir: &str) -> String;  // Returns backup path
pub fn restore_backup(backup_path: &str, target_dir: &str) -> i32;   // Returns exit code
pub fn list_backups(search_dir: &str) -> String;                     // Returns newline-separated paths
```

## CORRECTIVE ACTION ⚡

### 1. Replace Command Execution with RSB Macros
```rust
// Fix backup.rs archive creation
fn create_archive(&self, source_dir: &str, output_path: &str) -> i32 {
    let cmd = format!("tar -czf {} -C {} .", output_path, source_dir);
    let result = shell!(&cmd);
    
    if result.status != 0 {
        error!("Archive creation failed: {}", result.error);
        return 1;
    }
    
    okay!("Archive created: {}", output_path);
    0
}
```

### 2. Replace Environment Access with RSB Patterns
```rust
// Fix main.rs global flag handling
fn main() {
    let args = bootstrap!();
    
    // Use RSB patterns for argument handling
    if args.has_flag("--cursor") || args.has_flag("--user") {
        handle_global_context(&args);
    }
    
    // Continue with RSB dispatch...
}
```

### 3. Implement RSB Three-Tier Function Architecture
```rust
// Fix backup module with proper RSB patterns
pub fn do_backup(args: Args) -> i32 {
    require_command!("tar");
    
    let output_dir = args.get_flag("--output").unwrap_or(".");
    let operation = args.get_or(1, "create");
    
    match operation {
        "create" => _create_backup(output_dir),
        "restore" => _restore_backup(&args.get_or(2, "")),
        "list" => _list_backups(output_dir),
        _ => {
            error!("Unknown backup operation: {}", operation);
            1
        }
    }
}

fn _create_backup(output_dir: &str) -> i32 {
    let timestamp = date!(human);
    let backup_file = format!("{}/prontodb-backup-{}.tar.gz", output_dir, timestamp);
    
    if __create_archive_files(&backup_file) {
        okay!("Backup created: {}", backup_file);
        0
    } else {
        error!("Backup creation failed");
        1
    }
}

fn __create_archive_files(backup_path: &str) -> bool {
    let xdg = XdgPaths::new();
    let data_dir = xdg.data_dir.to_string_lossy();
    let config_dir = xdg.config_dir.to_string_lossy();
    
    let cmd = format!("tar -czf {} -C {} . -C {} .", backup_path, data_dir, config_dir);
    let result = shell!(&cmd);
    
    result.status == 0
}
```

### 4. Convert to String-Biased Interfaces
```rust
// Simplified backup functions with string-first approach
pub fn create_backup_simple(output_dir: &str) -> String {
    // Returns backup file path or empty string on error
}

pub fn restore_backup_simple(backup_path: &str) -> i32 {
    // Returns exit code (0=success, 1=error)
}

pub fn list_backups_simple(search_dir: &str) -> String {
    // Returns newline-separated backup file paths
}
```

## REFERENCE 📖

- **RSB Quick Reference v2.0**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-quick-reference-v2.md`
- **RSB Architecture**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` (Lines 89-100: Function-Based Development)
- **RSB Error Handling**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-quick-reference-v2.md` (Lines 212-248: Error Handling Patterns)
- **String-Biased Philosophy**: `/home/xnull/repos/code/rust/oodx/rebel/docs/ref/rsb-architecture.md` (Lines 27-86: String-Biased Philosophy)

---

🦊 **FINAL HUNT ASSESSMENT**: The lifecycle modules contain significant RSB violations that must be addressed before production deployment. The violations primarily stem from bypassing RSB's string-biased abstraction layer and reverting to raw Rust stdlib patterns. All violations have clear RSB-compliant alternatives documented in the canonical sources.

**PRIORITY**: Critical - These violations break RSB architectural purity and should be resolved in the next development iteration.