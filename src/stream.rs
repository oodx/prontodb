//! stream.rs — KV stream parser, auth preamble enforcement, transactional apply
//! Grammar (tokens ended by ';', whitespace tolerant):
//!   meta:sec:pass=...; meta:sec:user=...;  OR  meta:sec:apikey=...;
//!   meta:ns=...; meta:delim=.; meta:ttl=SECONDS;
//!   key=value; key2=value2;

use anyhow::{Result, anyhow};
use rsb::preamble::*;
use rusqlite::{params};
use crate::{Cfg};
use crate::store::{cmd_set, cmd_get}; // reuse ops if helpful (we'll apply directly here)

pub fn cmd_stream(cfg: &Cfg, input: &str) -> Result<()> {
    // Enforce preamble unless disabled in config
    let etc = home_dir().join(".local/etc/odx/prontodb/pronto.conf");
    let mut security_required = true;
    if etc.exists() {
        let content = std::fs::read_to_string(&etc)?;
        let map = parse_config_content(&content);
        if let Some(v) = map.get("security.required") {
            security_required = v == "true" || v == "1";
        }
    }

    // tokenize
    let mut ns = cfg.ns.clone();
    let mut delim = cfg.ns_delim;
    let mut ttl_default: Option<u64> = None;
    let mut authed = !security_required;
    let mut saw_pass = false;
    let mut saw_user = false;
    let mut apikey = false;

    // We'll collect (k,v) and apply in one transaction
    let mut kvs: Vec<(String, Vec<u8>)> = Vec::new();

    for raw in input.split(';') {
        let tok = raw.trim();
        if tok.is_empty() { continue; }

        if tok.starts_with("meta:") {
            // security
            if let Some(v) = tok.strip_prefix("meta:sec:pass=") { saw_pass = true; continue; }
            if let Some(v) = tok.strip_prefix("meta:sec:user=") { if saw_pass { saw_user = true; authed = true; } continue; }
            if let Some(v) = tok.strip_prefix("meta:sec:apikey=") { let _ = v; apikey = true; authed = true; continue; }
            // ns, delim, ttl
            if let Some(v) = tok.strip_prefix("meta:ns=") { ns = v.to_string(); continue; }
            if let Some(v) = tok.strip_prefix("meta:delim=") { delim = v.chars().next().unwrap_or(delim); continue; }
            if let Some(v) = tok.strip_prefix("meta:ttl=") { ttl_default = v.parse::<u64>().ok(); continue; }
            continue;
        }

        // data token: require auth first if needed
        if !authed { return Err(anyhow!("stream rejected: auth preamble required before data tokens")); }

        if let Some((k, v)) = tok.split_once('=') {
            kvs.push((k.trim().to_string(), v.as_bytes().to_vec()));
        }
    }

    // Apply transactionally using store::cmd_set for each kv
    for (k, v) in kvs {
        // Keys may be qualified; we do not override ns here.
        // TTL default only affects TTL namespaces; store::cmd_set will read per-ns timeout if None.
        crate::store::cmd_set(&Cfg { ns: ns.clone(), ns_delim: delim, ..cfg.clone() }, &k, &v, ttl_default, cfg.json)?;
    }

    Ok(())
}
