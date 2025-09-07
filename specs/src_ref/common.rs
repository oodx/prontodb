
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

pub fn now_epoch() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() as i64
}

#[derive(Debug, Clone)]
pub struct Config {
    pub ns_delim: char,
    pub security_required: bool,
    pub busy_timeout_ms: i64,
    pub db_path: PathBuf,
}

fn xdg_dir(home: &Path, env_key: &str, fallback: &str) -> PathBuf {
    if let Ok(p) = std::env::var(env_key) {
        if !p.is_empty() { return PathBuf::from(p); }
    }
    home.join(fallback)
}

pub fn load_config(home: &Path) -> io::Result<Config> {
    let etc = xdg_dir(home, "XDG_CONFIG_HOME", ".local/etc").join("odx/prontodb");
    let conf = etc.join("pronto.conf");

    let mut cfg = Config {
        ns_delim: '.',
        security_required: true,
        busy_timeout_ms: 5000,
        db_path: xdg_dir(home, "XDG_DATA_HOME", ".local/data").join("odx/prontodb/prontodb.sqlite3"),
    };

    if let Ok(db) = std::env::var("PRONTO_DB") {
        if !db.is_empty() { cfg.db_path = PathBuf::from(db); }
    }
    if let Ok(sec) = std::env::var("PRONTO_SECURITY") {
        let s = sec.to_lowercase();
        if s == "0" || s == "false" || s == "off" { cfg.security_required = false; }
    }

    if let Ok(mut f) = fs::File::open(&conf) {
        let mut s = String::new(); f.read_to_string(&mut s)?;
        for line in s.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') { continue; }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 { continue; }
            let k = parts[0].trim();
            let v = parts[1].trim().trim_matches('"');
            match k {
                "ns_delim" => if let Some(c) = v.chars().next() { cfg.ns_delim = c; },
                "security.required" => {
                    let vl = v.to_lowercase();
                    cfg.security_required = !(vl == "0" || vl == "false" || vl == "off");
                }
                "busy_timeout_ms" => cfg.busy_timeout_ms = v.parse().unwrap_or(5000),
                "db_path" => cfg.db_path = PathBuf::from(v),
                _ => {}
            }
        }
    }
    Ok(cfg)
}

pub struct Paths {
    pub etc: PathBuf,
    pub data: PathBuf,
    pub lib: PathBuf,
    pub bin: PathBuf,
    pub conf_file: PathBuf,
    pub db_file: PathBuf,
}

pub fn derive_paths(home: &Path, cfg: &Config) -> Paths {
    let etc = xdg_dir(home, "XDG_CONFIG_HOME", ".local/etc").join("odx/prontodb");
    let data = xdg_dir(home, "XDG_DATA_HOME", ".local/data").join("odx/prontodb");
    let lib = xdg_dir(home, "XDG_LIB_HOME", ".local/lib").join("odx/prontodb");
    let bin = home.join(".local/bin");
    let conf_file = etc.join("pronto.conf");
    let db_file = cfg.db_path.clone();
    Paths { etc, data, lib, bin, conf_file, db_file }
}

#[derive(Debug, Clone)]
pub struct Addr { pub project: String, pub namespace: String, pub key: String, pub ctx: Option<String> }

pub fn parse_addr(input: &str, delim: char) -> Result<Addr, String> {
    if input.is_empty() { return Err("empty address".into()); }
    let (base, ctx) = if let Some(i) = input.rfind("__") {
        (&input[..i], Some(input[i+2..].to_string()))
    } else { (input, None) };
    let parts: Vec<&str> = base.split(delim).collect();
    if parts.len() < 3 { return Err("address must be project.namespace.key".into()); }
    let project = parts[0].to_string();
    let namespace = parts[1].to_string();
    let key = parts[2].to_string();
    if key.contains(delim) { return Err("key must not contain the namespace delimiter".into()); }
    Ok(Addr { project, namespace, key, ctx })
}
