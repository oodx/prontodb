use crate::addressing::Address;
use crate::storage::Storage;
use crate::xdg::XdgPaths;

// Centralized helpers for CLI handlers and future RSB adapters

fn open_storage() -> Result<Storage, String> {
    let paths = XdgPaths::new();
    paths.ensure_dirs().map_err(|e| e.to_string())?;
    Storage::open(&paths.get_db_path()).map_err(|e| e.to_string())
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

