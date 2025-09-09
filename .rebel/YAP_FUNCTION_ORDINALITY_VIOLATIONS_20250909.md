# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/commands/backup.rs
**Violation Type**: Function ordinality pattern violations

## VIOLATION DETECTED 🚨

**Mixed ordinality naming in backup.rs**: The backup module mixes public API functions, helpers, and low-level utilities without proper RSB ordinality naming conventions:

```rust
// Lines 54, 120, 146, 182, 212 - Missing RSB function ordinality prefixes
pub fn backup_database(config: BackupConfig) -> Result<BackupResult, BackupError>
fn generate_unique_backup_name(backup_dir: &Path, db_name: &str, date_str: &str) -> Result<(String, Option<u32>), BackupError>
pub fn list_backups(search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError>
pub fn restore_backup(backup_path: &Path, config: BackupConfig) -> Result<(), BackupError>
pub fn handle_backup_command(args: rsb::args::Args) -> i32
```

## CANONICAL RSB PATTERN 📚

From rsb-architecture.md Section 1.5 "BashFX Function Ordinality in Rust":
> "RSB Function Ordinality Rules (adapted from BashFX):
> - `pub fn api_function`: User-facing, full input validation, user fault errors  
> - `fn _helper_function`: Business logic, app fault errors, assumes valid inputs
> - `fn __blind_faith_function`: System operations, system fault errors only"

## CORRECTIVE ACTION ⚡

Apply proper RSB function ordinality naming:

```rust
// PUBLIC API FUNCTIONS (User fault error handling)  
pub fn do_backup(args: rsb::args::Args) -> i32 {
    // Handle user errors with helpful messages
    // Delegate to mid-level helpers
    let result = _create_backup(&config);
    // Format user-friendly output
    0
}

// CRATE-INTERNAL FUNCTIONS (App fault error handling)
fn _create_backup(config: &BackupConfig) -> Result<BackupResult, BackupError> {
    // Business logic, assume valid inputs from public layer
    // Handle app logic errors
    let backup_path = __copy_database_file(&source, &target)?;
    Ok(result)
}

fn _list_backup_files(search_dir: Option<&Path>) -> Result<Vec<PathBuf>, BackupError> {
    // Business logic for listing backups
    let files = __scan_directory(search_dir)?;
    Ok(filtered_files)
}

fn _restore_from_backup(backup_path: &Path, config: &BackupConfig) -> Result<(), BackupError> {
    // Business logic for restore operation
    __copy_file_with_verification(backup_path, target_path)?;
    Ok(())
}

// LOW-LEVEL UTILITY FUNCTIONS (System fault error handling)
fn __copy_database_file(source: &Path, target: &Path) -> Result<PathBuf, BackupError> {
    // "Blind faith" function - trust caller provided valid paths
    // Handle only system-level errors (permissions, disk space, etc.)
    std::fs::copy(source, target)?;
    Ok(target.to_path_buf())
}

fn __scan_directory(dir: &Path) -> Result<Vec<PathBuf>, BackupError> {
    // Low-level directory scanning
    std::fs::read_dir(dir)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.extension().map_or(false, |ext| ext == "db"))
        .collect()
}

fn __generate_backup_filename(base: &str, date: &str) -> Result<(String, Option<u32>), BackupError> {
    // Low-level filename generation logic
    // System-level error handling only
}
```

## REFERENCE 📖
- RSB Architecture Framework Section 1.5: "BashFX Function Ordinality in Rust"  
- RSB Function Ordinality Rules for responsibility separation
- BashFX Origins: Function ordinality prevents "everything in one function" problem