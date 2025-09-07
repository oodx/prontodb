//! main.rs — RSB-style CLI (no clap).
//! stdout=data, stderr=status

use std::path::PathBuf;
use std::io::{self, Read};
use anyhow::{Result, anyhow};

use rsb::preamble::*;
mod store;
mod stream;

#[derive(Default)]
pub struct Cfg {
    pub db: Option<PathBuf>,
    pub ns: String,
    pub ns_delim: char,
    pub json: bool,
    pub b64: bool,
}

fn load_cfg() -> Result<Cfg> {
    // Minimal config loader via RSB helpers; fallback defaults
    let etc = home_dir().join(".local/etc/odx/prontodb/pronto.conf");
    let mut cfg = Cfg { db: None, ns: "default".into(), ns_delim: ':', json: false, b64: false };
    if etc.exists() {
        let content = std::fs::read_to_string(&etc)?;
        let map = parse_config_content(&content);
        if let Some(d) = map.get("ns_delim") { cfg.ns_delim = d.chars().next().unwrap_or(':'); }
    }
    Ok(cfg)
}

fn usage() {
    echo!(r#"prontodb [-d DB] [-n NS] [--ns-delim C] [--json] [--b64] <cmd> [args...]

Commands:
  install | uninstall [--purge] | backup [--out PATH] [--age-recipient X | --age-identity Y]
  init
  set <key> <val|-> [--ttl SECONDS]
  get <key> [--include-expired]
  del <key>
  keys [prefix] | ls [prefix] [--stream]
  scan [prefix] [--json] [--stream] [--include-expired]
  nss
  export [ns] | import [ns]
  stream
  admin create-cache <ns> timeout=... [evict_on_read=1; max_items=N; ...]
  admin set-cache <ns> key=val;... | admin drop-cache <ns>
"#);
}

fn main() -> Result<()> {
    rsb_bootstrap()?;

    let mut cfg = load_cfg()?;
    let mut args = args();
    args.pop(); // program name

    // Global flags (string-only parsing)
    loop {
        let peek = args.peek();
        if peek.is_none() { break; }
        match peek.unwrap().as_str() {
            "-d" => { args.pop(); cfg.db = Some(PathBuf::from(args.pop().ok_or_else(|| anyhow!("-d needs a path"))?)); }
            "-n" => { args.pop(); cfg.ns = args.pop().unwrap_or_else(|| "default".into()); }
            "--ns-delim" => { args.pop(); cfg.ns_delim = args.pop().unwrap_or_else(|| ":".into()).chars().next().unwrap_or(':'); }
            "--json" => { args.pop(); cfg.json = true; }
            "--b64" => { args.pop(); cfg.b64 = true; }
            "-h" | "--help" => { usage(); return Ok(()); }
            _ => break,
        }
    }

    let cmd = match args.pop_opt() { Some(s) => s, None => { usage(); return Ok(()); } };

    match cmd.as_str() {
        "install" => { store::cmd_install(&cfg)?; }
        "uninstall" => { let purge = args.has("--purge"); store::cmd_uninstall(purge)?; }
        "backup" => {
            let out = args.after("--out");
            let age_rec = args.after("--age-recipient");
            let age_id  = args.after("--age-identity");
            store::cmd_backup(out, age_rec, age_id)?;
        }
        "init" => { store::cmd_init(&cfg)?; }
        "set" => {
            let key = args.pop().ok_or_else(|| anyhow!("set needs <key> and <val|->"))?;
            let val_tok = args.pop().ok_or_else(|| anyhow!("set needs <val|->"))?;
            let ttl = args.opt_u64("--ttl");
            let mut data: Vec<u8> = Vec::new();
            if val_tok == "-" { io::stdin().read_to_end(&mut data)?; }
            else { data = val_tok.into_bytes(); }
            store::cmd_set(&cfg, &key, &data, ttl, cfg.json)?;
        }
        "get" => {
            let key = args.pop().ok_or_else(|| anyhow!("get needs <key>"))?;
            let include_expired = args.has("--include-expired");
            store::cmd_get(&cfg, &key, include_expired, cfg.json, cfg.b64)?;
        }
        "del" => { let key = args.pop().ok_or_else(|| anyhow!("del needs <key>"))?; store::cmd_del(&cfg, &key)?; }
        "keys" | "ls" => {
            let prefix = args.pop_opt();
            let as_stream = args.has("--stream");
            store::cmd_keys(&cfg, prefix.as_deref(), as_stream)?;
        }
        "scan" => {
            let prefix = args.pop_opt();
            let as_stream = args.has("--stream");
            let include_expired = args.has("--include-expired");
            let json_out = cfg.json || args.has("--json");
            store::cmd_scan(&cfg, prefix.as_deref(), include_expired, json_out, as_stream, cfg.b64)?;
        }
        "nss" => { store::cmd_nss(&cfg)?; }
        "export" => { let ns = args.pop_opt(); store::cmd_export(&cfg, ns.as_deref())?; }
        "import" => {
            let ns = args.pop_opt();
            let mut buf = Vec::new(); io::stdin().read_to_end(&mut buf)?;
            store::cmd_import(&cfg, ns.as_deref(), &buf)?;
        }
        "stream" => {
            let mut stdin = String::new(); io::stdin().read_to_string(&mut stdin)?;
            stream::cmd_stream(&cfg, &stdin)?;
        }
        "admin" => {
            let sub = args.pop().unwrap_or_default();
            match sub.as_str() {
                "create-cache" => {
                    let ns = args.pop().ok_or_else(|| anyhow!("admin create-cache <ns> ..."))?;
                    let spec = args.rest_join(" ");
                    store::cmd_admin_create_cache(&cfg, &ns, &spec)?;
                }
                "set-cache" => {
                    let ns = args.pop().ok_or_else(|| anyhow!("admin set-cache <ns> key=val;..."))?;
                    let spec = args.rest_join(" ");
                    store::cmd_admin_set_cache(&cfg, &ns, &spec)?;
                }
                "drop-cache" => {
                    let ns = args.pop().ok_or_else(|| anyhow!("admin drop-cache <ns>"))?;
                    store::cmd_admin_drop_cache(&cfg, &ns)?;
                }
                _ => fatal!("unknown admin command"),
            }
        }
        _ => { usage(); fatal!("unknown command"); }
    }
    Ok(())
}
