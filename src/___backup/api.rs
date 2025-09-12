#![allow(dead_code)]  // Functions are exported for library use

use crate::addressing::{Address, AddressContext};
use crate::cursor::{CursorManager, CursorData};
use crate::storage::Storage;
use crate::xdg::XdgPaths;

// Centralized helpers for CLI handlers and future RSB adapters

// Meta namespace key transformation utilities
fn transform_address_for_storage(user_addr: &Address, meta_context: &Option<String>) -> Address {
    match meta_context {
        Some(meta) => Address {
            project: format!("{}.{}", meta, user_addr.project),
            namespace: user_addr.namespace.clone(),
            key: user_addr.key.clone(),
            context: user_addr.context.clone(),
        },
        None => user_addr.clone(),
    }
}

fn transform_address_for_display(storage_addr: &Address, meta_context: &Option<String>) -> Address {
    match meta_context {
        Some(meta) => {
            let prefix = format!("{}.", meta);
            let user_project = storage_addr.project.strip_prefix(&prefix)
                .unwrap_or(&storage_addr.project)
                .to_string();
            Address {
                project: user_project,
                namespace: storage_addr.namespace.clone(),
                key: storage_addr.key.clone(),
                context: storage_addr.context.clone(),
            }
        }
        None => storage_addr.clone(),
    }
}

// Enhanced storage opening that returns cursor context for key transformation
fn open_storage_with_cursor_context(
    cursor_name: Option<&str>, 
    user: &str
) -> Result<(Storage, Option<CursorData>), String> {
    open_storage_with_cursor_context_and_database(cursor_name, user, "main")
}

// Enhanced storage opening with explicit CursorManager for dependency injection
fn open_storage_with_cursor_context_and_database_with_manager(
    cursor_name: Option<&str>, 
    user: &str, 
    db_name: &str,
    cursor_manager: &CursorManager
) -> Result<(Storage, Option<CursorData>), String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    
    let (db_path, cursor_data) = if let Some(cursor) = cursor_name {
        cursor_manager.ensure_default_cursor(user).map_err(|e| e.to_string())?;
        
        match cursor_manager.get_cursor(cursor, user) {
            Ok(cursor_data) => (cursor_data.database_path.clone(), Some(cursor_data)),
            Err(_) => {
                // Cursor not found, fall back to database-scoped path
                (paths.get_db_path_with_name(db_name), None)
            }
        }
    } else {
        // No cursor specified, use database-scoped path directly
        (paths.get_db_path_with_name(db_name), None)
    };
    
    // Ensure database-specific directory exists
    let db_dir = db_path.parent().unwrap();
    std::fs::create_dir_all(db_dir).map_err(|e| e.to_string())?;
    
    let storage = Storage::open(&db_path).map_err(|e| e.to_string())?;
    Ok((storage, cursor_data))
}

fn open_storage_with_cursor_context_and_database(
    cursor_name: Option<&str>, 
    user: &str, 
    db_name: &str
) -> Result<(Storage, Option<CursorData>), String> {
    // Use CursorManager::from_xdg with default paths for CLI consistency
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    let cursor_manager = CursorManager::from_xdg(paths.clone());
    
    let (db_path, cursor_data) = if let Some(cursor) = cursor_name {
        cursor_manager.ensure_default_cursor(user).map_err(|e| e.to_string())?;
        
        match cursor_manager.get_cursor(cursor, user) {
            Ok(cursor_data) => (cursor_data.database_path.clone(), Some(cursor_data)),
            Err(_) => {
                // Cursor not found, fall back to database-scoped path
                (paths.get_db_path_with_name(db_name), None)
            }
        }
    } else {
        // No cursor specified, use database-scoped path directly
        (paths.get_db_path_with_name(db_name), None)
    };
    
    // Ensure database-specific directory exists
    let db_dir = db_path.parent().unwrap();
    std::fs::create_dir_all(db_dir).map_err(|e| e.to_string())?;
    
    let storage = Storage::open(&db_path).map_err(|e| e.to_string())?;
    Ok((storage, cursor_data))
}

fn open_storage() -> Result<Storage, String> {
    open_storage_with_database("main")
}

fn open_storage_with_database(db_name: &str) -> Result<Storage, String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    
    // Ensure database-specific directory exists
    let db_dir = paths.get_database_dir(db_name);
    std::fs::create_dir_all(&db_dir).map_err(|e| e.to_string())?;
    
    Storage::open(&paths.get_db_path_with_name(db_name)).map_err(|e| e.to_string())
}

fn open_storage_with_cursor(cursor_name: Option<&str>, user: &str) -> Result<Storage, String> {
    open_storage_with_cursor_and_database(cursor_name, user, "main")
}

fn open_storage_with_cursor_and_database(cursor_name: Option<&str>, user: &str, db_name: &str) -> Result<Storage, String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    
    let db_path = if let Some(cursor) = cursor_name {
        let cursor_manager = CursorManager::new();
        // Ensure default cursor exists for this user
        cursor_manager.ensure_default_cursor(user).map_err(|e| e.to_string())?;
        
        match cursor_manager.resolve_database_path(Some(cursor), user) {
            Ok(Some(path)) => path,
            Ok(None) => {
                // Cursor not found, fall back to database-scoped path
                paths.get_db_path_with_name(db_name)
            },
            Err(e) => return Err(e.to_string()),
        }
    } else {
        // No cursor specified, use database-scoped path directly
        paths.get_db_path_with_name(db_name)
    };
    
    // Ensure database-specific directory exists
    let db_dir = db_path.parent().unwrap();
    std::fs::create_dir_all(db_dir).map_err(|e| e.to_string())?;
    
    Storage::open(&db_path).map_err(|e| e.to_string())
}

fn parse_address_from_parts(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    context: AddressContext,
) -> Result<Address, String> {
    if key_or_path.contains(ns_delim) {
        Address::parse_with_context(key_or_path, ns_delim, context)
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
    set_value_with_database(project, namespace, key_or_path, value, ns_delim, ttl_flag, "main")
}

pub fn set_value_with_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    value: &str,
    ns_delim: &str,
    ttl_flag: Option<u64>,
    db_name: &str,
) -> Result<(), String> {
    let storage = open_storage_with_database(db_name)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
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
    get_value_with_database(project, namespace, key_or_path, ns_delim, "main")
}

pub fn get_value_with_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    db_name: &str,
) -> Result<Option<String>, String> {
    let storage = open_storage_with_database(db_name)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    storage.get(&addr).map_err(|e| e.to_string())
}

pub fn delete_value(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
) -> Result<(), String> {
    delete_value_with_database(project, namespace, key_or_path, ns_delim, "main")
}

pub fn delete_value_with_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    db_name: &str,
) -> Result<(), String> {
    let storage = open_storage_with_database(db_name)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    storage.delete(&addr).map_err(|e| e.to_string())
}

pub fn list_keys(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
) -> Result<Vec<String>, String> {
    list_keys_with_database(project, namespace, prefix, "main")
}

pub fn list_keys_with_database(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    db_name: &str,
) -> Result<Vec<String>, String> {
    let storage = open_storage_with_database(db_name)?;
    storage
        .list_keys(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn scan_pairs(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
) -> Result<Vec<(String, String)>, String> {
    scan_pairs_with_database(project, namespace, prefix, "main")
}

pub fn scan_pairs_with_database(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    db_name: &str,
) -> Result<Vec<(String, String)>, String> {
    let storage = open_storage_with_database(db_name)?;
    storage
        .scan(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn create_ttl_namespace(project: &str, namespace: &str, default_ttl: u64) -> Result<(), String> {
    create_ttl_namespace_with_database(project, namespace, default_ttl, "main")
}

pub fn create_ttl_namespace_with_database(project: &str, namespace: &str, default_ttl: u64, db_name: &str) -> Result<(), String> {
    let storage = open_storage_with_database(db_name)?;
    storage
        .create_ttl_namespace(project, namespace, default_ttl)
        .map_err(|e| e.to_string())
}

pub fn projects() -> Result<Vec<String>, String> {
    projects_with_database("main")
}

pub fn projects_with_database(db_name: &str) -> Result<Vec<String>, String> {
    let storage = open_storage_with_database(db_name)?;
    storage.list_projects().map_err(|e| e.to_string())
}

pub fn namespaces(project: &str) -> Result<Vec<String>, String> {
    namespaces_with_database(project, "main")
}

pub fn namespaces_with_database(project: &str, db_name: &str) -> Result<Vec<String>, String> {
    let storage = open_storage_with_database(db_name)?;
    storage
        .list_namespaces(project)
        .map_err(|e| e.to_string())
}

// Cursor-aware API functions for database context support

pub struct SetValueConfig<'a> {
    pub project: Option<&'a str>,
    pub namespace: Option<&'a str>,
    pub key_or_path: &'a str,
    pub value: &'a str,
    pub ns_delim: &'a str,
    pub ttl_flag: Option<u64>,
    pub cursor_name: Option<&'a str>,
    pub user: &'a str,
    pub database: &'a str,
    pub meta_context_override: Option<&'a str>,  // Per-command meta context override
}

pub fn set_value_with_cursor(config: SetValueConfig) -> Result<(), String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(config.cursor_name, config.user, config.database)?;
    
    // Parse user's input key (3-layer format)
    let user_addr = parse_address_from_parts(config.project, config.namespace, config.key_or_path, config.ns_delim, AddressContext::KeyAccess)?;
    user_addr.validate_key(config.ns_delim)?;
    
    // Apply meta context transformation for storage (enhanced project addressing)
    // Use override if provided, otherwise use cursor's stored meta context
    let meta_context = config.meta_context_override
        .map(|s| s.to_string())
        .or_else(|| cursor_data.as_ref().and_then(|c| c.meta_context.clone()));
    let storage_addr = transform_address_for_storage(&user_addr, &meta_context);

    let ns_default_ttl = storage
        .get_namespace_ttl(&storage_addr.project, &storage_addr.namespace)
        .map_err(|e| e.to_string())?;
    let effective_ttl = match (ns_default_ttl, config.ttl_flag) {
        (Some(_), Some(ttl)) => Some(ttl),
        (Some(default), None) => Some(default),
        (None, Some(_)) => return Err("TTL not allowed: namespace is not TTL-enabled".into()),
        (None, None) => None,
    };

    storage
        .set(&storage_addr, config.value, effective_ttl)
        .map_err(|e| e.to_string())
}

// Enhanced version with explicit CursorManager for testing and dependency injection
pub fn set_value_with_cursor_and_manager(
    config: SetValueConfig, 
    cursor_manager: &CursorManager
) -> Result<(), String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database_with_manager(
        config.cursor_name, 
        config.user, 
        config.database, 
        cursor_manager
    )?;
    
    // Parse user's input key (3-layer format)
    let user_addr = parse_address_from_parts(config.project, config.namespace, config.key_or_path, config.ns_delim, AddressContext::KeyAccess)?;
    user_addr.validate_key(config.ns_delim)?;
    
    // Apply meta context transformation for storage (enhanced project addressing)
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    let storage_addr = transform_address_for_storage(&user_addr, &meta_context);

    let ns_default_ttl = storage
        .get_namespace_ttl(&storage_addr.project, &storage_addr.namespace)
        .map_err(|e| e.to_string())?;
    let effective_ttl = match (ns_default_ttl, config.ttl_flag) {
        (Some(_), Some(ttl)) => Some(ttl),
        (Some(default), None) => Some(default),
        (None, Some(_)) => return Err("TTL not allowed: namespace is not TTL-enabled".into()),
        (None, None) => None,
    };

    storage
        .set(&storage_addr, config.value, effective_ttl)
        .map_err(|e| e.to_string())
}

pub fn get_value_with_cursor(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    meta_context_override: Option<&str>,
) -> Result<Option<String>, String> {
    get_value_with_cursor_and_database(project, namespace, key_or_path, ns_delim, cursor_name, user, "main", meta_context_override)
}

pub fn get_value_with_cursor_and_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
    meta_context_override: Option<&str>,
) -> Result<Option<String>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    
    // Parse user's input key (3-layer format)
    let user_addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    
    // Apply meta context transformation
    // Use override if provided, otherwise use cursor's stored meta context
    let meta_context = meta_context_override
        .map(|s| s.to_string())
        .or_else(|| cursor_data.as_ref().and_then(|c| c.meta_context.clone()));
    
    if meta_context.is_some() {
        // With meta context: ONLY use meta-prefixed key (no fallback for pure isolation)
        let meta_addr = transform_address_for_storage(&user_addr, &meta_context);
        storage.get(&meta_addr).map_err(|e| e.to_string())
    } else {
        // No meta context, use direct lookup
        storage.get(&user_addr).map_err(|e| e.to_string())
    }
}

pub fn delete_value_with_cursor(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<(), String> {
    delete_value_with_cursor_and_database(project, namespace, key_or_path, ns_delim, cursor_name, user, "main")
}

pub fn delete_value_with_cursor_and_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<(), String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    
    // Parse user's input key (3-layer format)
    let user_addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    
    // Apply meta context transformation
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    
    if let Some(_) = &meta_context {
        // When using meta context, ONLY delete meta-prefixed key to maintain isolation
        let storage_addr = transform_address_for_storage(&user_addr, &meta_context);
        storage.delete(&storage_addr).map_err(|e| e.to_string())
    } else {
        // No meta context, use direct deletion
        storage.delete(&user_addr).map_err(|e| e.to_string())
    }
}

pub fn list_keys_with_cursor(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<String>, String> {
    list_keys_with_cursor_and_database(project, namespace, prefix, cursor_name, user, "main")
}

pub fn list_keys_with_cursor_and_database(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<Vec<String>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    
    if let Some(meta) = &meta_context {
        // List from meta-prefixed project
        let meta_project = format!("{}.{}", meta, project);
        let keys = storage.list_keys(&meta_project, namespace, prefix).map_err(|e| e.to_string())?;
        
        // Keys are already in the right format (just the key part), no transformation needed
        Ok(keys)
    } else {
        // No meta context, use direct listing
        storage.list_keys(project, namespace, prefix).map_err(|e| e.to_string())
    }
}

pub fn scan_pairs_with_cursor(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<(String, String)>, String> {
    scan_pairs_with_cursor_and_database(project, namespace, prefix, cursor_name, user, "main")
}

pub fn scan_pairs_with_cursor_and_database(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<Vec<(String, String)>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    
    if let Some(meta) = &meta_context {
        // Scan from meta-prefixed project
        let meta_project = format!("{}.{}", meta, project);
        let pairs = storage.scan(&meta_project, namespace, prefix).map_err(|e| e.to_string())?;
        
        // Pairs are already in the right format (key, value), no transformation needed
        Ok(pairs)
    } else {
        // No meta context, use direct scanning
        storage.scan(project, namespace, prefix).map_err(|e| e.to_string())
    }
}

// Flexible addressing versions that support both flags and dot notation
pub fn list_keys_flexible(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<String>, String> {
    list_keys_flexible_with_database(project, namespace, key_or_path, ns_delim, cursor_name, user, "main")
}

pub fn list_keys_flexible_with_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<Vec<String>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::Discovery)?;
    
    // For keys command, the "key" part becomes the prefix
    let prefix = if addr.key.is_empty() { None } else { Some(addr.key.as_str()) };
    
    // Apply meta namespace transformation if available
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    
    if let Some(meta) = &meta_context {
        // Use meta-prefixed project for listing
        let meta_project = format!("{}.{}", meta, addr.project);
        storage
            .list_keys(&meta_project, &addr.namespace, prefix)
            .map_err(|e| e.to_string())
    } else {
        // No meta context, use direct listing
        storage
            .list_keys(&addr.project, &addr.namespace, prefix)
            .map_err(|e| e.to_string())
    }
}

pub fn scan_pairs_flexible(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<(String, String)>, String> {
    scan_pairs_flexible_with_database(project, namespace, key_or_path, ns_delim, cursor_name, user, "main")
}

pub fn scan_pairs_flexible_with_database(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<Vec<(String, String)>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database(cursor_name, user, db_name)?;
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::Discovery)?;
    
    // For scan command, the "key" part becomes the prefix
    let prefix = if addr.key.is_empty() { None } else { Some(addr.key.as_str()) };
    
    // Apply meta namespace transformation if available
    let meta_context = cursor_data.as_ref().and_then(|c| c.meta_context.clone());
    
    if let Some(meta) = &meta_context {
        // Use meta-prefixed project for scanning
        let meta_project = format!("{}.{}", meta, addr.project);
        storage
            .scan(&meta_project, &addr.namespace, prefix)
            .map_err(|e| e.to_string())
    } else {
        // No meta context, use direct scanning
        storage
            .scan(&addr.project, &addr.namespace, prefix)
            .map_err(|e| e.to_string())
    }
}

pub fn create_ttl_namespace_with_cursor(
    project: &str, 
    namespace: &str, 
    default_ttl: u64,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<(), String> {
    create_ttl_namespace_with_cursor_and_database(project, namespace, default_ttl, cursor_name, user, "main")
}

pub fn create_ttl_namespace_with_cursor_and_database(
    project: &str, 
    namespace: &str, 
    default_ttl: u64,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<(), String> {
    let storage = open_storage_with_cursor_and_database(cursor_name, user, db_name)?;
    storage
        .create_ttl_namespace(project, namespace, default_ttl)
        .map_err(|e| e.to_string())
}

pub fn projects_with_cursor(cursor_name: Option<&str>, user: &str) -> Result<Vec<String>, String> {
    projects_with_cursor_and_database(cursor_name, user, "main")
}

pub fn projects_with_cursor_and_database(cursor_name: Option<&str>, user: &str, db_name: &str) -> Result<Vec<String>, String> {
    let storage = open_storage_with_cursor_and_database(cursor_name, user, db_name)?;
    storage.list_projects().map_err(|e| e.to_string())
}

pub fn namespaces_with_cursor(
    project: &str,
    cursor_name: Option<&str>,
    user: &str,
) -> Result<Vec<String>, String> {
    namespaces_with_cursor_and_database(project, cursor_name, user, "main")
}

pub fn namespaces_with_cursor_and_database(
    project: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
) -> Result<Vec<String>, String> {
    let storage = open_storage_with_cursor_and_database(cursor_name, user, db_name)?;
    storage
        .list_namespaces(project)
        .map_err(|e| e.to_string())
}

// Enhanced API functions with explicit CursorManager for Krex's fix
pub fn get_value_with_cursor_and_manager(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
    cursor_manager: &CursorManager,
) -> Result<Option<String>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database_with_manager(cursor_name, user, db_name, cursor_manager)?;
    
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    
    if let Some(cursor) = cursor_data {
        if let Some(meta_context) = &cursor.meta_context {
            // With meta context: ONLY use meta-prefixed key (no fallback for pure isolation)
            let meta_addr = transform_address_for_storage(&addr, &Some(meta_context.clone()));
            return storage.get(&meta_addr).map_err(|e| e.to_string());
        }
    }
    
    // No meta context, use direct lookup
    storage.get(&addr).map_err(|e| e.to_string())
}

pub fn delete_value_with_cursor_and_manager(
    project: Option<&str>,
    namespace: Option<&str>,
    key_or_path: &str,
    ns_delim: &str,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
    cursor_manager: &CursorManager,
) -> Result<(), String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database_with_manager(cursor_name, user, db_name, cursor_manager)?;
    
    let addr = parse_address_from_parts(project, namespace, key_or_path, ns_delim, AddressContext::KeyAccess)?;
    
    if let Some(cursor) = cursor_data {
        if let Some(meta_context) = &cursor.meta_context {
            // Delete meta-prefixed key
            let meta_addr = transform_address_for_storage(&addr, &Some(meta_context.clone()));
            return storage.delete(&meta_addr).map_err(|e| e.to_string());
        }
    }
    
    storage.delete(&addr).map_err(|e| e.to_string())
}

pub fn list_keys_with_cursor_and_manager(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
    cursor_manager: &CursorManager,
) -> Result<Vec<String>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database_with_manager(cursor_name, user, db_name, cursor_manager)?;
    
    if let Some(cursor) = cursor_data {
        if let Some(meta_context) = &cursor.meta_context {
            // List keys with meta-prefixed project
            let meta_project = format!("{}.{}", meta_context, project);
            return storage
                .list_keys(&meta_project, namespace, prefix)
                .map_err(|e| e.to_string());
        }
    }
    
    storage
        .list_keys(project, namespace, prefix)
        .map_err(|e| e.to_string())
}

pub fn scan_pairs_with_cursor_and_manager(
    project: &str,
    namespace: &str,
    prefix: Option<&str>,
    cursor_name: Option<&str>,
    user: &str,
    db_name: &str,
    cursor_manager: &CursorManager,
) -> Result<Vec<(String, String)>, String> {
    let (storage, cursor_data) = open_storage_with_cursor_context_and_database_with_manager(cursor_name, user, db_name, cursor_manager)?;
    
    if let Some(cursor) = cursor_data {
        if let Some(meta_context) = &cursor.meta_context {
            // Scan pairs with meta-prefixed project
            let meta_project = format!("{}.{}", meta_context, project);
            return storage
                .scan(&meta_project, namespace, prefix)
                .map_err(|e| e.to_string());
        }
    }
    
    storage
        .scan(project, namespace, prefix)
        .map_err(|e| e.to_string())
}


