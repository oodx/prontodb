//! store.rs — SQLite + schema + CRUD + TTL + admin

use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::io::Write;
use rsb::preamble::*;
use rusqlite::{Connection, params, OpenFlags, OptionalExtension};

use crate::Cfg;

fn xdg_paths() -> (PathBuf, PathBuf, PathBuf) {
    let home = home_dir();
    let etc = home.join(".local/etc/odx/prontodb");
    let data = home.join(".local/data/odx/prontodb");
    let lib  = home.join(".local/lib/odx/prontodb");
    (etc, data, lib)
}

fn db_path(cfg: &Cfg) -> PathBuf {
    if let Some(p) = &cfg.db { return p.clone(); }
    let (_, data, _) = xdg_paths();
    data.join("prontodb.sqlite")
}

fn open_db(cfg: &Cfg) -> Result<Connection> {
    let path = db_path(cfg);
    if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
    let conn = Connection::open_with_flags(&path,
        OpenFlags::SQLITE_OPEN_READ_WRITE |
        OpenFlags::SQLITE_OPEN_CREATE |
        OpenFlags::SQLITE_OPEN_URI
    )?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;
    conn.pragma_update(None, "busy_timeout", &5000i64)?;
    Ok(conn)
}

pub fn cmd_install(cfg: &Cfg) -> Result<()> {
    let (etc, data, lib) = xdg_paths();
    std::fs::create_dir_all(&etc)?;
    std::fs::create_dir_all(&data)?;
    std::fs::create_dir_all(&lib)?;
    // write default pronto.conf if missing
    let conf = etc.join("pronto.conf");
    if !conf.exists() {
        let c = "ns_delim=\":\"\nsecurity.required=true\nbusy_timeout_ms=5000\n";
        std::fs::write(&conf, c)?;
        stderr!("wrote {}", conf.display());
    }
    // bootstrap DB + schema + default admin
    cmd_init(cfg)?;
    seed_default_admin(cfg)?;
    stderr!("install complete: etc={}, data={}, lib={}", etc.display(), data.display(), lib.display());
    Ok(())
}

pub fn cmd_uninstall(purge: bool) -> Result<()> {
    let (etc, data, lib) = xdg_paths();
    if purge {
        if data.exists() { std::fs::remove_dir_all(&data)?; }
    }
    if etc.exists() { std::fs::remove_dir_all(&etc)?; }
    if lib.exists() { std::fs::remove_dir_all(&lib)?; }
    stderr!("uninstall complete (purge={})", purge);
    Ok(())
}

pub fn cmd_backup(out: Option<String>, age_rec: Option<String>, _age_id: Option<String>) -> Result<()> {
    let (etc, data, _) = xdg_paths();
    let db = data.join("prontodb.sqlite");
    let conf = etc.join("pronto.conf");
    let out_file = out.unwrap_or_else(|| format!("{}/backup.tar.gz", data.display()));
    // assemble tar.gz
    let tmpdir = tmp!("prontodb_backup"); 
    let tar = tmpdir.path().join("bk.tar"); 
    let gz = tmpdir.path().join("bk.tar.gz");
    tar::create(&tar, &[db.to_string_lossy().to_string(), conf.to_string_lossy().to_string()])?;
    tar::gzip(&tar, &gz)?;
    if let Some(rec) = age_rec {
        run!("age -r {} -o {} {}", rec, out_file, gz.display())?;
    } else {
        std::fs::copy(&gz, &out_file)?;
    }
    stderr!("backup written: {}", out_file);
    Ok(())
}

pub fn cmd_init(cfg: &Cfg) -> Result<()> {
    let conn = open_db(cfg)?;
    conn.execute_batch(r#"
        CREATE TABLE IF NOT EXISTS sys_namespaces(
            ns TEXT PRIMARY KEY, kind TEXT CHECK(kind IN ('std','ttl')) NOT NULL,
            table_name TEXT NOT NULL, delim TEXT DEFAULT ':'
        );
        CREATE TABLE IF NOT EXISTS sys_caches(
            ns TEXT PRIMARY KEY, timeout_sec INTEGER NOT NULL, evict_on_read INTEGER DEFAULT 1, max_items INTEGER NULL
        );
        CREATE TABLE IF NOT EXISTS sec_users(
            user TEXT PRIMARY KEY, pass_hash TEXT NOT NULL, prefs TEXT NULL, created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS sec_api_keys(
            key TEXT PRIMARY KEY, user TEXT NOT NULL, created_at INTEGER NOT NULL, last_used INTEGER
        );
        CREATE TABLE IF NOT EXISTS sec_sessions(
            sess TEXT PRIMARY KEY, user TEXT NOT NULL, expires_at INTEGER NOT NULL, created_at INTEGER NOT NULL
        );
    "#)?;
    stderr!("init complete");
    Ok(())
}

fn seed_default_admin(cfg: &Cfg) -> Result<()> {
    let conn = open_db(cfg)?;
    let exists: Option<String> = conn.query_row("SELECT user FROM sec_users LIMIT 1", [], |r| r.get(0)).optional()?;
    if exists.is_none() {
        let pass = get_env("PRONTO_ADMIN_PASS").unwrap_or_else(|| "pronto!".into());
        // try external hashers; fallback to plain:
        let hashed = if process_exists("openssl") {
            let out = run!("bash -lc", format!("printf '%s' '{}' | openssl dgst -sha256", pass))?;
            format!("sha256:{}", str_trim(&out))
        } else if process_exists("sha256sum") {
            let out = run!("bash -lc", format!("printf '%s' '{}' | sha256sum | awk '{print $1}'", pass))?;
            format!("sha256:{}", str_trim(&out))
        } else {
            format!("plain:{}", pass)
        };
        let now = date!("+%s").parse::<i64>().unwrap_or(0);
        conn.execute("INSERT INTO sec_users(user, pass_hash, created_at) VALUES(?1, ?2, ?3)",
            params!["admin", hashed, now])?;
        stderr!("seeded default admin 'admin' with password 'pronto!' (override with PRONTO_ADMIN_PASS)");
    }
    Ok(())
}

// ---------- Namespace table helpers ----------

fn slug_from_ns(ns: &str, delim: char) -> String {
    ns.split(delim).map(|s| s.replace(|c: char| !c.is_ascii_alphanumeric(), "_")).collect::<Vec<_>>().join("__")
}

fn table_std(ns: &str, delim: char) -> String { format!("ns_{}", slug_from_ns(ns, delim)) }
fn table_ttl(ns: &str, delim: char) -> String { format!("ns_{}__ttl", slug_from_ns(ns, delim)) }

fn ensure_ns(conn: &Connection, ns: &str, delim: char) -> Result<(String, bool)> {
    // returns (table_name, is_ttl)
    let found: Option<(String, String)> =
        conn.query_row("SELECT kind, table_name FROM sys_namespaces WHERE ns=?1", params![ns], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
    if let Some((kind, table)) = found { return Ok((table, kind == "ttl")); }
    // default: standard namespace
    let table = table_std(ns, delim);
    conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}(k TEXT PRIMARY KEY, v BLOB NOT NULL)", table), [])?;
    conn.execute("INSERT OR REPLACE INTO sys_namespaces(ns, kind, table_name, delim) VALUES(?1,'std',?2,?3)",
        params![ns, table, delim.to_string()])?;
    Ok((table_std(ns, delim), false))
}

fn ensure_cache(conn: &Connection, ns: &str, delim: char, timeout: i64) -> Result<String> {
    let table = table_ttl(ns, delim);
    conn.execute(&format!("CREATE TABLE IF NOT EXISTS {}(k TEXT PRIMARY KEY, v BLOB NOT NULL, created_at INTEGER NOT NULL, ttl_sec INTEGER NOT NULL)", table), [])?;
    conn.execute("INSERT OR REPLACE INTO sys_namespaces(ns, kind, table_name, delim) VALUES(?1,'ttl',?2,?3)",
        params![ns, table, delim.to_string()])?;
    conn.execute("INSERT OR REPLACE INTO sys_caches(ns, timeout_sec) VALUES(?1,?2)", params![ns, timeout])?;
    Ok(table)
}

// ---------- Commands ----------

pub fn cmd_set(cfg: &Cfg, key: &str, val: &[u8], ttl: Option<u64>, json_flag: bool) -> Result<()> {
    let conn = open_db(cfg)?;
    let (table, is_ttl) = ensure_ns(&conn, &cfg.ns, cfg.ns_delim)?;
    let mut payload = val.to_vec();
    if json_flag {
        if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&payload) {
            payload = serde_json::to_vec(&v)?;
        }
    }
    if is_ttl {
        let ttl_sec: i64 = ttl.map(|x| x as i64).or_else(|| {
            conn.query_row::<i64,_>("SELECT timeout_sec FROM sys_caches WHERE ns=?1", params![cfg.ns.as_str()], |r| r.get(0)).ok()
        }).unwrap_or(3600);
        let now = date!("+%s").parse::<i64>().unwrap_or(0);
        conn.execute(&format!("INSERT OR REPLACE INTO {}(k,v,created_at,ttl_sec) VALUES(?1,?2,?3,?4)", table),
            params![key, payload, now, ttl_sec])?;
    } else {
        conn.execute(&format!("INSERT OR REPLACE INTO {}(k,v) VALUES(?1,?2)", table), params![key, payload])?;
    }
    Ok(())
}

pub fn cmd_get(cfg: &Cfg, key: &str, include_expired: bool, json_out: bool, b64: bool) -> Result<()> {
    let conn = open_db(cfg)?;
    let (table, is_ttl) = ensure_ns(&conn, &cfg.ns, cfg.ns_delim)?;
    let out: Option<Vec<u8>> = if is_ttl && !include_expired {
        let now = date!("+%s").parse::<i64>().unwrap_or(0);
        conn.query_row(&format!("SELECT v FROM {} WHERE k=?1 AND (created_at + ttl_sec) > ?2", table), params![key, now], |r| r.get(0)).optional()?
    } else {
        conn.query_row(&format!("SELECT v FROM {} WHERE k=?1", table), params![key], |r| r.get(0)).optional()?
    };
    if let Some(v) = out {
        if json_out {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                println!("{}", serde_json::to_string_pretty(&val)?);
                return Ok(());
            }
        }
        if b64 { println!("{}", base64::engine::general_purpose::STANDARD.encode(&v)); }
        else { std::io::stdout().write_all(&v)?; }
    }
    Ok(())
}

pub fn cmd_del(cfg: &Cfg, key: &str) -> Result<()> {
    let conn = open_db(cfg)?;
    let (table, _) = ensure_ns(&conn, &cfg.ns, cfg.ns_delim)?;
    conn.execute(&format!("DELETE FROM {} WHERE k=?1", table), params![key])?;
    Ok(())
}

pub fn cmd_keys(cfg: &Cfg, prefix: Option<&str>, as_stream: bool) -> Result<()> {
    let conn = open_db(cfg)?;
    let (table, _) = ensure_ns(&conn, &cfg.ns, cfg.ns_delim)?;
    let rows = if let Some(p) = prefix {
        let like = format!("{}%", p);
        conn.prepare(&format!("SELECT k FROM {} WHERE k LIKE ?1 ORDER BY k", table))?
            .query_map(params![like], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?
    } else {
        conn.prepare(&format!("SELECT k FROM {} ORDER BY k", table))?
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<rusqlite::Result<Vec<String>>>()?
    };
    if as_stream {
        println!("meta:ns={};", cfg.ns);
        for k in rows { println!("{}=;", k); }
    } else {
        for k in rows { println!("{}", k); }
    }
    Ok(())
}

pub fn cmd_scan(cfg: &Cfg, prefix: Option<&str>, include_expired: bool, json_out: bool, as_stream: bool, b64: bool) -> Result<()> {
    let conn = open_db(cfg)?;
    let (table, is_ttl) = ensure_ns(&conn, &cfg.ns, cfg.ns_delim)?;
    let mut sql = format!("SELECT k, v FROM {}", table);
    let mut where_clauses: Vec<String> = Vec::new();
    let now = date!("+%s").parse::<i64>().unwrap_or(0);
    if is_ttl && !include_expired { where_clauses.push(format!("(created_at + ttl_sec) > {}", now)); }
    if let Some(p) = prefix { where_clauses.push(format!("k LIKE '{}%'", p.replace(\"'\", \"''\"))); }
    if !where_clauses.is_empty() { sql.push_str(" WHERE "); sql.push_str(&where_clauses.join(" AND ")); }
    sql.push_str(" ORDER BY k");
    let mut stmt = conn.prepare(&sql)?;
    let mut rows = stmt.query([])?;
    if as_stream { println!("meta:ns={};", cfg.ns); }
    while let Some(r) = rows.next()? {
        let k: String = r.get(0)?; let v: Vec<u8> = r.get(1)?;
        if as_stream {
            if json_out {
                if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                    println!("{}={};", k, serde_json::to_string(&val)?);
                    continue;
                }
            }
            if b64 { println!("{}={};", k, base64::engine::general_purpose::STANDARD.encode(&v)); }
            else { print!("{}=", k); std::io::stdout().write_all(&v)?; println!(";"); }
        } else if json_out {
            if let Ok(val) = serde_json::from_slice::<serde_json::Value>(&v) {
                println!("{}\t{}", k, serde_json::to_string(&val)?);
            } else if b64 {
                println!("{}\t{}", k, base64::engine::general_purpose::STANDARD.encode(&v));
            } else {
                print!("{}\t", k); std::io::stdout().write_all(&v)?; println!();
            }
        } else {
            if b64 { println!("{}\t{}", k, base64::engine::general_purpose::STANDARD.encode(&v)); }
            else { print!("{}\t", k); std::io::stdout().write_all(&v)?; println!(); }
        }
    }
    Ok(())
}

pub fn cmd_nss(cfg: &Cfg) -> Result<()> {
    let conn = open_db(cfg)?;
    let mut stmt = conn.prepare("SELECT ns, kind, table_name, delim FROM sys_namespaces ORDER BY ns")?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let ns: String = r.get(0)?; let kind: String = r.get(1)?; let table: String = r.get(2)?; let delim: String = r.get(3)?;
        if kind == "ttl" {
            if let Ok(timeout): Result<i64, _> = conn.query_row("SELECT timeout_sec FROM sys_caches WHERE ns=?1", params![ns.as_str()], |rr| rr.get(0)).optional().map(|o| o.unwrap_or(0)) {
                println!("{}\t{}\t{}\tdelim={}\ttimeout_sec={}", ns, kind, table, delim, timeout);
            } else {
                println!("{}\t{}\t{}\tdelim={}", ns, kind, table, delim);
            }
        } else {
            println!("{}\t{}\t{}\tdelim={}", ns, kind, table, delim);
        }
    }
    Ok(())
}

pub fn cmd_export(cfg: &Cfg, ns: Option<&str>) -> Result<()> {
    let conn = open_db(cfg)?;
    let which = ns.unwrap_or(&cfg.ns);
    let (table, _ttl) = ensure_ns(&conn, which, cfg.ns_delim)?;
    let mut stmt = conn.prepare(&format!("SELECT k, v FROM {} ORDER BY k", table))?;
    let mut rows = stmt.query([])?;
    while let Some(r) = rows.next()? {
        let k: String = r.get(0)?; let v: Vec<u8> = r.get(1)?;
        println!("{}\t{}", k, base64::engine::general_purpose::STANDARD.encode(&v));
    }
    Ok(())
}

pub fn cmd_import(cfg: &Cfg, ns: Option<&str>, buf: &[u8]) -> Result<()> {
    let conn = open_db(cfg)?;
    let which = ns.unwrap_or(&cfg.ns);
    let (table, _ttl) = ensure_ns(&conn, which, cfg.ns_delim)?;
    let s = String::from_utf8_lossy(buf);
    let tx = conn.unchecked_transaction()?;
    for (i, line) in s.lines().enumerate() {
        if line.trim().is_empty() { continue; }
        if let Some((k, b64)) = line.split_once('\t') {
            let v = base64::engine::general_purpose::STANDARD.decode(b64.trim()).map_err(|e| anyhow!("line {} base64: {}", i+1, e))?;
            tx.execute(&format!("INSERT OR REPLACE INTO {}(k,v) VALUES(?1,?2)", table), params![k.trim(), v])?;
        }
    }
    tx.commit()?;
    Ok(())
}

// Admin commands

pub fn cmd_admin_create_cache(cfg: &Cfg, ns: &str, spec: &str) -> Result<()> {
    let conn = open_db(cfg)?;
    let mut timeout = 3600i64; // default 1h
    for tok in spec.split(|c| c==' ' || c==';') {
        let t = tok.trim(); if t.is_empty() { continue; }
        if let Some((k,v)) = t.split_once('=') {
            if k == "timeout" || k == "timeout_sec" { timeout = v.parse::<i64>().unwrap_or(3600); }
        }
    }
    let table = ensure_cache(&conn, ns, cfg.ns_delim, timeout)?;
    stderr!("cache created: ns={} table={} timeout={}", ns, table, timeout);
    Ok(())
}

pub fn cmd_admin_set_cache(cfg: &Cfg, ns: &str, spec: &str) -> Result<()> {
    let conn = open_db(cfg)?;
    for tok in spec.split(|c| c==' ' || c==';') {
        let t = tok.trim(); if t.is_empty() { continue; }
        if let Some((k,v)) = t.split_once('=') {
            match k {
                "timeout" | "timeout_sec" => { conn.execute("UPDATE sys_caches SET timeout_sec=?1 WHERE ns=?2", params![v.parse::<i64>().unwrap_or(3600), ns])?; }
                "evict_on_read" => { conn.execute("UPDATE sys_caches SET evict_on_read=?1 WHERE ns=?2", params![v.parse::<i64>().unwrap_or(1), ns])?; }
                "max_items" => { conn.execute("UPDATE sys_caches SET max_items=?1 WHERE ns=?2", params![v.parse::<i64>().ok()], ns)?; }
                _ => {}
            }
        }
    }
    Ok(())
}

pub fn cmd_admin_drop_cache(cfg: &Cfg, ns: &str) -> Result<()> {
    let conn = open_db(cfg)?;
    let found: Option<(String, String)> =
        conn.query_row("SELECT kind, table_name FROM sys_namespaces WHERE ns=?1", params![ns], |r| Ok((r.get(0)?, r.get(1)?))).optional()?;
    if let Some((kind, table)) = found {
        if kind != "ttl" { return Err(anyhow!("namespace is not ttl: {}", ns)); }
        conn.execute(&format!("DROP TABLE IF EXISTS {}", table), [])?;
        conn.execute("DELETE FROM sys_caches WHERE ns=?1", params![ns])?;
        conn.execute("DELETE FROM sys_namespaces WHERE ns=?1", params![ns])?;
        stderr!("cache dropped: {}", ns);
    }
    Ok(())
}
