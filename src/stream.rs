
use crate::store::{Store, NsKind};
use crate::common::{Config, Addr, parse_addr, now_epoch};

pub fn handle_stream(store: &Store, cfg: &Config, input: &str) -> i32 {
    let mut pass: Option<String> = None;
    let mut user: Option<String> = None;
    let mut apikey: Option<String> = None;
    let mut delim = cfg.ns_delim;
    let mut ns_path: Option<(String,String)> = None;
    let mut ttl_override: Option<i64> = None;
    let mut order: Vec<&str> = Vec::new();

    for raw in input.split(';') {
        let t = raw.trim();
        if t.is_empty() { continue; }
        if t.starts_with("meta:") {
            if let Some(v)=t.strip_prefix("meta:sec:pass=") { pass=Some(v.to_string()); order.push("pass"); continue; }
            if let Some(v)=t.strip_prefix("meta:sec:user=") { user=Some(v.to_string()); order.push("user"); continue; }
            if let Some(v)=t.strip_prefix("meta:sec:apikey=") { apikey=Some(v.to_string()); order.push("apikey"); continue; }
            if let Some(v)=t.strip_prefix("meta:path=") { ns_path=split_ns(v,delim); continue; }
            if let Some(v)=t.strip_prefix("meta:ns=") { ns_path=split_ns(v,delim); continue; }
            if let Some(v)=t.strip_prefix("meta:project=") { ns_path.get_or_insert((v.to_string(), String::new())); continue; }
            if let Some(v)=t.strip_prefix("meta:namespace=") {
                if let Some((ref mut p, ref mut n)) = ns_path { *n=v.to_string(); } else { ns_path=Some(("default".into(), v.to_string())); }
                continue;
            }
            if let Some(v)=t.strip_prefix("meta:delim=") { if let Some(c)=v.chars().next(){ delim=c; } continue; }
            if let Some(v)=t.strip_prefix("meta:ttl=") { ttl_override=v.parse::<i64>().ok(); continue; }
        }
    }

    if cfg.security_required {
        if apikey.is_none() {
            let pi = order.iter().position(|&x| x=="pass");
            let ui = order.iter().position(|&x| x=="user");
            if pi.is_none() || ui.is_none() || pi.unwrap() > ui.unwrap() {
                eprintln!("auth preamble required: meta:sec:pass=...; meta:sec:user=...;");
                return 1;
            }
            let u = user.unwrap();
            let p = pass.unwrap();
            let ok: Option<i64> = store.conn.query_row(
                "SELECT 1 FROM sec_users WHERE user=?1 AND pass=?2 LIMIT 1",
                rusqlite::params![u, p], |r| r.get(0)).optional().unwrap_or(None);
            if ok.is_none() { eprintln!("invalid credentials"); return 1; }
        }
    }

    let (project, namespace) = ns_path.unwrap_or_else(|| ("default".into(), "default".into()));
    if let Err(e) = store.ensure_ns(&project, &namespace, delim) { eprintln!("{}", e); return 1; }
    let kind = store.ns_kind(&project, &namespace).unwrap_or(NsKind::Std);
    let tn = match kind { NsKind::Std => format!("ns_{}_{}", project, namespace), NsKind::Ttl => format!("ns_{}_{}__ttl", project, namespace) };

    for raw in input.split(';') {
        let t = raw.trim();
        if t.is_empty() || t.starts_with("meta:") { continue; }
        if let Some((k,v)) = t.split_once('=') {
            if k.contains(delim) { eprintln!("key cannot contain delimiter"); return 1; }
            let key = if let Some(i)=k.rfind("__") { (&k[..i], Some(&k[i+2..])) } else { (k, None) };
            let fullkey = if let Some(ctx)=key.1 { format!("{}__{}", key.0, ctx) } else { key.0.to_string() };
            match kind {
                NsKind::Std => {
                    if ttl_override.is_some() { eprintln!("meta:ttl only valid in TTL namespaces"); return 1; }
                    if let Err(e)=store.conn.execute(&format!("INSERT OR REPLACE INTO {}(k,v) VALUES (?1,?2)", tn),
                        rusqlite::params![fullkey, v.as_bytes()]) { eprintln!("{}", e); return 1; }
                }
                NsKind::Ttl => {
                    let def_ttl: i64 = store.conn.query_row(
                        "SELECT timeout_sec FROM sys_caches WHERE project=?1 AND namespace=?2",
                        rusqlite::params![&project, &namespace], |r| r.get(0)).unwrap_or(60);
                    let ttl = ttl_override.unwrap_or(def_ttl);
                    let now = now_epoch();
                    if let Err(e)=store.conn.execute(&format!("INSERT OR REPLACE INTO {}(k,v,created_at,ttl_sec) VALUES (?1,?2,?3,?4)", tn),
                        rusqlite::params![fullkey, v.as_bytes(), now, ttl]) { eprintln!("{}", e); return 1; }
                }
            }
        }
    }
    0
}

fn split_ns(path: &str, delim: char) -> Option<(String,String)> {
    let mut it = path.split(delim);
    let p = it.next()?.to_string();
    let n = it.next()?.to_string();
    Some((p,n))
}
