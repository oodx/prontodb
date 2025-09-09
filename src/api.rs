#![allow(dead_code)]  // Functions are exported for library use

use crate::addressing::Address;
use crate::cursor::CursorManager;
use crate::storage::Storage;
use crate::xdg::XdgPaths;

// Centralized helpers for CLI handlers and future RSB adapters

fn open_storage() -> Result<Storage, String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    Storage::open(&paths.get_db_path()).map_err(|e| e.to_string())
}

fn open_storage_with_cursor(cursor_name: Option<&str>, user: &str) -> Result<Storage, String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    
    let db_path = if let Some(cursor) = cursor_name {
        let cursor_manager = CursorManager::new();
        // Ensure default cursor exists for this user
        cursor_manager.ensure_default_cursor(user).map_err(|e| e.to_string())?;
        
        match cursor_manager.resolve_database_path(Some(cursor), user) {
            Ok(Some(path)) => path,
            Ok(None) => {
                // Cursor not found, fall back to default XDG path
                paths.get_db_path()
            },
            Err(e) => return Err(e.to_string()),
        }
    } else {
        // No cursor specified, use default cursor for user if it exists
        let cursor_manager = CursorManager::new();
        cursor_manager.ensure_default_cursor(user).map_err(|e| e.to_string())?;
        match cursor_manager.resolve_database_path(None, user) {
            Ok(Some(path)) => path,
            Ok(None) => paths.get_db_path(),
            Err(_) => paths.get_db_path(), // Fallback on any cursor error
        }
    };
    
    Storage::open(&db_path).map_err(|e| e.to_string())
}

fn parse_address_from_parts(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
) -> Result<Address, String> {
    if key_or_path.contains(ns_delim) {
        Address::parse(key_or_path, ns_delim)
    } else {
        let (key, context) = if let Some(idx) = key_or_path.rfind("__") {
            let k = &key_or_path[..idx];
            let ctxs = &key_or_path[idx + 2..];
            (
                k.to_string(),
                if ctxs.is_empty() { None } else { Some(ctxs.to_string()) },
            )
        } else {
            (key_or_path.to_string(), None)
        };
        Ok(Address::from_parts(
            project.map(|s| s.to_string()),
            namespace.map(|s| s.to_string()),
            key,
            context,
        ))
    }
}

pub fn set_value(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    value: &str,
    ns_delim: &str,
    ttl_flag: Option<u64>,
) -> Result<(), String> {
    let storage = open_storage()?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    addr.validate_key(ns_delim)?;

    let ns_default_ttl = storage
        .get_namespace_ttl(&addr.project, &addr.namespace)
        .map_err(|e| e.to_string())?;
    let effective_ttl = match (ns_default_ttl, ttl_flag) {
        (Some(_), Some(ttl)) => Some(ttl),
        (Some(default), None) => Some(default),
        (None, Some(_)) => return Err("TTL not allowed: namespace is not TTL-enabled".into()),
        (None, None) => None,
    };

    storage
        .set(&addr, value, effective_ttl)
        .map_err(|e| e.to_string())
}

pub fn get_value(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
) -> Result<Option<String>, String> {
    let storage = open_storage()?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    storage.get(&addr).map_err(|e| e.to_string())
}

pub fn delete_value(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
) -> Result<(), String> {
    let storage = open_storage()?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    storage.delete(&addr).map_err(|e| e.to_string())
}

pub fn list_keys(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
) -> Result<Vec<String>, String> {
    let storage = open_storage()?;
    storage
        .list_keys(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn scan_pairs(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    let storage = open_storage()?;
    storage
        .scan(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn create_ttl_namespace(project: &str, namespace: &str, default_ttl: u64) -> Result<(), String> {
    let storage = open_storage()?;
    storage
        .create_ttl_namespace(project, namespace, default_ttl)
        .map_err(|e| e.to_string())
}

pub fn projects() -> Result<Vec<String>, String> {
    let storage = open_storage()?;
    storage.list_projects().map_err(|e| e.to_string())
}

pub fn namespaces(project: &str) -> Result<Vec<String>, String> {
    let storage = open_storage()?;
    storage
        .list_namespaces(project)
        .map_err(|e| e.to_string())
}

// Cursor-aware API functions for database context support

pub fn set_value_with_cursor(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    value: &str,
    ns_delim: &str,
    ttl_flag: Option<u64>,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<(), String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    addr.validate_key(ns_delim)?;

    let ns_default_ttl = storage
        .get_namespace_ttl(&addr.project, &addr.namespace)
        .map_err(|e| e.to_string())?;
    let effective_ttl = match (ns_default_ttl, ttl_flag) {
        (Some(_), Some(ttl)) => Some(ttl),
        (Some(default), None) => Some(default),
        (None, Some(_)) => return Err("TTL not allowed: namespace is not TTL-enabled".into()),
        (None, None) => None,
    };

    storage
        .set(&addr, value, effective_ttl)
        .map_err(|e| e.to_string())
}

pub fn get_value_with_cursor(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Option<String>, String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    storage.get(&addr).map_err(|e| e.to_string())
}

pub fn delete_value_with_cursor(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<(), String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim)?;
    storage.delete(&addr).map_err(|e| e.to_string())
}

pub fn list_keys_with_cursor(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<String>, String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    storage
        .list_keys(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn scan_pairs_with_cursor(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<(String, String)>, String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    storage
        .scan(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn create_ttl_namespace_with_cursor(
    project: &str, 
    namespace: &str, 
    default_ttl: u64,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<(), String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    storage
        .create_ttl_namespace(project, namespace, default_ttl)
        .map_err(|e| e.to_string())
}

pub fn projects_with_cursor(cursor_name: Option<&str>, user: &str) -> Result<Vec<String>, String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    storage.list_projects().map_err(|e| e.to_string())
}

pub fn namespaces_with_cursor(
    project: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<String>, String> {
    let storage = open_storage_with_cursor(cursor_name, user)?;
    storage
        .list_namespaces(project)
        .map_err(|e| e.to_string())
}

