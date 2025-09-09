# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/xdg.rs
**Violation Type**: Manual std::env::var() Instead of RSB param!() Macro

## VIOLATION DETECTED 🚨
Multiple instances of manual std::env::var() usage throughout xdg.rs:

```rust
// Lines 67-72 - Manual env var access
pub fn get_db_path(&self) -> PathBuf {
    if let Ok(db_path) = env::var("PRONTO_DB") {
        PathBuf::from(db_path)
    } else {
        self.db_path.clone()
    }
}

// Lines 85-95 - More manual env var access
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

// Lines 97-103 - XDG env vars manually accessed
fn get_data_dir(home: &Path) -> PathBuf {
    if let Ok(xdg_data_home) = env::var("XDG_DATA_HOME") {
        PathBuf::from(xdg_data_home).join("odx").join("prontodb")
    } else {
        home.join(".local").join("data").join("odx").join("prontodb")
    }
}
```

## CANONICAL RSB PATTERN 📚
RSB provides param!() macro to replace std::env::var():

```rust
// RSB Environment Variable Access
let value = param!("ENV_VAR_NAME");           // Required param
let value = param!("ENV_VAR_NAME", "default"); // With default
```

## CORRECTIVE ACTION ⚡
Replace all std::env::var() usage with RSB param!() macro:

```rust
pub fn get_db_path(&self) -> PathBuf {
    let db_path = param!("PRONTO_DB", "");
    if !db_path.is_empty() {
        PathBuf::from(db_path)
    } else {
        self.db_path.clone()
    }
}

fn get_home_dir() -> PathBuf {
    let home = param!("HOME", "");
    if !home.is_empty() {
        PathBuf::from(home)
    } else {
        let userprofile = param!("USERPROFILE", "");
        if !userprofile.is_empty() {
            PathBuf::from(userprofile)
        } else {
            PathBuf::from("/tmp")
        }
    }
}

fn get_data_dir(home: &Path) -> PathBuf {
    let xdg_data_home = param!("XDG_DATA_HOME", "");
    if !xdg_data_home.is_empty() {
        PathBuf::from(xdg_data_home).join("odx").join("prontodb")
    } else {
        home.join(".local").join("data").join("odx").join("prontodb")
    }
}
```

## REFERENCE 📖
RSB Architecture Guide: "Replace std::env::var() with param!() macro"