# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/cursor.rs
**Violation Type**: Manual Error Handling Instead of RSB Macros

## VIOLATION DETECTED 🚨
Multiple instances of manual error handling throughout cursor.rs:

```rust
// Lines 65-73 - Manual Result handling
pub fn new() -> Result<Self, std::io::Error> {
    let xdg = XdgPaths::new();
    let cursor_dir = xdg.data_dir.join("cursors");
    
    // Ensure cursor directory exists
    fs::create_dir_all(&cursor_dir)?;
    
    Ok(Self { xdg, cursor_dir })
}

// Lines 83-91 - Manual error propagation
pub fn set_cursor(&self, name: &str, database_path: PathBuf, user: &str) -> Result<(), Box<dyn std::error::Error>> {
    let cursor_data = CursorData::new(database_path, user.to_string());
    let cursor_file = self.get_cursor_file_path(name, user);
    
    let json_content = serde_json::to_string_pretty(&cursor_data)?;
    fs::write(&cursor_file, json_content)?;
    
    Ok(())
}
```

## CANONICAL RSB PATTERN 📚
RSB provides validate!() and require_*!() macros for error handling:

```rust
// RSB Error Handling Macros
validate!(condition, "error message");
require_file!("path/to/file");  
require_var!("ENV_VAR_NAME");
```

## CORRECTIVE ACTION ⚡
Replace manual error handling with RSB macros:

```rust
pub fn new() -> Self {
    let xdg = XdgPaths::new();
    let cursor_dir = xdg.data_dir.join("cursors");
    
    // RSB directory validation
    validate!(fs::create_dir_all(&cursor_dir).is_ok(), 
             "Failed to create cursor directory");
    
    Self { xdg, cursor_dir }
}

pub fn set_cursor(&self, name: &str, database_path: PathBuf, user: &str) {
    let cursor_data = CursorData::new(database_path, user.to_string());
    let cursor_file = self.get_cursor_file_path(name, user);
    
    let json_content = validate!(serde_json::to_string_pretty(&cursor_data),
                                "Failed to serialize cursor data");
    validate!(fs::write(&cursor_file, json_content),
             "Failed to write cursor file");
}
```

## REFERENCE 📖
RSB Architecture Guide: "Replace manual error handling with validate!(), require_var!(), require_file!()"