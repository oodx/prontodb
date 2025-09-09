# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/cursor.rs
**Violation Type**: Missing Three-Tier Function Ordinality

## VIOLATION DETECTED 🚨
Cursor module functions don't follow RSB three-tier naming convention:

```rust
// Lines 65-235 - All functions at same level, no ordinality
impl CursorManager {
    pub fn new() -> Result<Self, std::io::Error>
    pub fn from_xdg(xdg: XdgPaths) -> Result<Self, std::io::Error>
    pub fn set_cursor(&self, name: &str, database_path: PathBuf, user: &str)
    pub fn set_cursor_with_defaults(&self, name: &str, ...)
    pub fn get_cursor(&self, name: &str, user: &str)
    pub fn get_active_cursor(&self, user: &str)
    pub fn list_cursors(&self, user: &str)
    pub fn delete_cursor(&self, name: &str, user: &str)
    // No helper or blind faith functions
}
```

## CANONICAL RSB PATTERN 📚
RSB enforces three-tier function ordinality:
1. **do_*** - Public command handlers
2. **_helper_*** - Internal logic functions  
3. **__blind_faith_*** - Lowest level operations

```rust
// RSB Three-Tier Function Ordinality
pub fn do_cursor_operation() -> i32 { ... }     // Tier 1: Public interface
fn _helper_validate_cursor() -> bool { ... }    // Tier 2: Internal logic
fn __blind_faith_write_file() { ... }          // Tier 3: Raw operations
```

## CORRECTIVE ACTION ⚡
Restructure cursor functions following RSB ordinality:

```rust
impl CursorManager {
    // Tier 1: Public do_* functions (command interface)
    pub fn do_create_cursor(&self, name: &str, database_path: &str, user: &str) {
        validate!(!name.is_empty(), "Cursor name required");
        validate!(!database_path.is_empty(), "Database path required");
        
        let path_buf = _helper_validate_path(database_path);
        __blind_faith_write_cursor_file(&self.get_cursor_file_path(name, user), &path_buf, user);
    }
    
    pub fn do_get_cursor(&self, name: &str, user: &str) -> String {
        let cursor_file = self.get_cursor_file_path(name, user);
        _helper_load_cursor_data(&cursor_file)
    }
    
    // Tier 2: _helper_* functions (validation and processing)
    fn _helper_validate_path(path: &str) -> PathBuf {
        let path_buf = PathBuf::from(path);
        validate!(path_buf.is_absolute(), "Database path must be absolute");
        path_buf
    }
    
    fn _helper_load_cursor_data(cursor_file: &Path) -> String {
        require_file!(cursor_file.to_str().unwrap());
        __blind_faith_read_cursor_file(cursor_file)
    }
    
    // Tier 3: __blind_faith_* functions (raw operations)
    fn __blind_faith_write_cursor_file(file_path: &Path, db_path: &PathBuf, user: &str) {
        let cursor_data = CursorData::new(db_path.clone(), user.to_string());
        let json_content = cat!(cursor_data); // RSB serialization
        std::fs::write(file_path, json_content).expect("Write cursor file");
    }
    
    fn __blind_faith_read_cursor_file(file_path: &Path) -> String {
        std::fs::read_to_string(file_path).expect("Read cursor file")
    }
}
```

## REFERENCE 📖
RSB Architecture Guide: "Three-tier function ordinality: do_* → _helper_* → __blind_faith_*"