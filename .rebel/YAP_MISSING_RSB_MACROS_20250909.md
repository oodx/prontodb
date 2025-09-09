# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/commands/backup.rs
**Violation Type**: Missing RSB validation and error handling macros

## VIOLATION DETECTED 🚨

**Manual error handling instead of RSB macros**: The backup module uses manual Result handling patterns instead of RSB's validate!(), require_file!(), and fatal!() macros.

```rust
// Lines 74-76: Manual file existence check
if !db_path.exists() {
    return Err(BackupError::DatabaseNotFound(format!("Database file not found: {}", db_path.display())));
}

// Lines 183-185: Manual file existence check  
if !backup_path.exists() {
    return Err(BackupError::DatabaseNotFound(format!("Backup file not found: {}", backup_path.display())));
}

// Lines 160-162: Manual directory existence check
if !search_path.exists() {
    return Ok(Vec::new());
}
```

## CANONICAL RSB PATTERN 📚

From rsb-architecture.md Section 2.2:
> "validate!(!processed.is_empty(), 'No errors found to process');"

From rsb-quick-reference-v2.md:
> "require_file!(path);" - Validates file existence and exits on failure

From rsb-architecture.md error handling patterns:
> "require_file!(path); let content = cat!(path); validate!(!content.is_empty(), 'Config file is empty');"

## CORRECTIVE ACTION ⚡

Replace manual error handling with RSB macros:

```rust
// Replace manual file existence checks:
// OLD:
if !db_path.exists() {
    return Err(BackupError::DatabaseNotFound(format!("Database file not found: {}", db_path.display())));
}

// NEW (RSB compliant):
require_file!(&db_path.to_string_lossy());

// Replace manual directory checks:
// OLD: 
if !search_path.exists() {
    return Ok(Vec::new());
}

// NEW (RSB compliant):
if !test!(-d search_path.to_string_lossy()) {
    return Ok(Vec::new());  // Empty result for missing directory
}

// Replace manual validation with RSB patterns:
// OLD:
if backup_path.exists() {
    // process...
} else {
    return Err(...);
}

// NEW (RSB compliant):
require_file!(&backup_path.to_string_lossy());
// continue processing...
```

**Complete function refactor example**:
```rust
pub fn _create_backup(config: &BackupConfig) -> i32 {
    // RSB validation pattern
    let db_path = _resolve_database_path(config);
    require_file!(&db_path);  // RSB macro handles error and exit
    
    let backup_dir = _determine_backup_directory(config);
    validate!(!backup_dir.is_empty(), "Invalid backup directory");
    
    // Continue with business logic knowing inputs are valid
    let result = __copy_database_file(&db_path, &backup_dir);
    0
}
```

## REFERENCE 📖
- RSB Architecture Framework: Error handling patterns Section 2.2
- RSB Quick Reference v2: require_file!(), validate!(), test!() macros  
- RSB prelude: Validation and file system macros