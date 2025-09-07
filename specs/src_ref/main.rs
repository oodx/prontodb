
mod common;
mod store;
mod stream;

use crate::common::{load_config, derive_paths, parse_addr, Addr};
use crate::store::Store;
use std::env;
use std::path::PathBuf;

fn usage() {
    eprintln!("prontodb: [-p P] [-n N] [-d DB] [--ns-delim C] <cmd>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 { usage(); return; }

    let mut project: Option<String> = None;
    let mut namespace: Option<String> = None;
    let mut ns_delim: Option<char> = None;
    let mut db_override: Option<PathBuf> = None;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "-p" if i+1 < args.len() => { project = Some(args[i+1].clone()); i+=2; }
            "-n" if i+1 < args.len() => { namespace = Some(args[i+1].clone()); i+=2; }
            "-d" if i+1 < args.len() => { db_override = Some(PathBuf::from(&args[i+1])); i+=2; }
            "--ns-delim" if i+1 < args.len() => { ns_delim = args[i+1].chars().next(); i+=2; }
            s if !s.starts_with('-') => { break; }
            _ => { i+=1; }
        }
    }
    if i >= args.len() { usage(); return; }
    let cmd = args[i].as_str();
    let rest = &args[i+1..];

    let home = dirs_next::home_dir().unwrap_or_else(|| PathBuf::from("."));
    let mut cfg = load_config(&home).expect("load config");
    if let Some(d) = db_override { cfg.db_path = d; }
    if let Some(c) = ns_delim { cfg.ns_delim = c; }
    let paths = common::derive_paths(&home, &cfg);
    let store = Store::open(&cfg).expect("open db");

    match cmd {
        "install" => {
            store.install(&paths.etc, &paths.data).expect("install");
            println!("installed");
        }
        "uninstall" => {
            let purge = rest.iter().any(|s| s == "--purge");
            store.uninstall(&paths.data, &paths.etc, purge).expect("uninstall");
            println!("uninstalled");
        }
        "admin" => {
            if rest.len() >= 2 && rest[0] == "create-cache" {
                let ns = &rest[1];
                let parts: Vec<&str> = ns.split(cfg.ns_delim).collect();
                if parts.len() != 2 { eprintln!("use <project.namespace>"); std::process::exit(1); }
                let mut timeout = 60i64;
                for kv in &rest[2..] {
                    if let Some(v) = kv.strip_prefix("timeout=") { timeout = v.parse().unwrap_or(60); }
                }
                store.create_cache(parts[0], parts[1], timeout, cfg.ns_delim).expect("create-cache");
                println!("ok");
            } else {
                eprintln!("admin create-cache <project.namespace> timeout=SECONDS");
                std::process::exit(1);
            }
        }
        "set" => {
            if rest.len() < 2 { eprintln!("set <k|p.n.k[__ctx]> <v> [--ttl SECONDS]"); std::process::exit(1); }
            let key = &rest[0];
            let val = rest[1].as_bytes();
            let mut ttl: Option<i64> = None;
            let mut j = 2;
            while j < rest.len() {
                if rest[j] == "--ttl" && j+1 < rest.len() { ttl = rest[j+1].parse().ok(); j+=2; } else { j+=1; }
            }
            let addr = if key.contains(cfg.ns_delim) {
                parse_addr(key, cfg.ns_delim).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); })
            } else {
                let p = project.clone().unwrap_or_else(|| "default".into());
                let n = namespace.clone().unwrap_or_else(|| "default".into());
                if key.contains(cfg.ns_delim) { eprintln!("key cannot contain delimiter"); std::process::exit(1); }
                Addr { project: p, namespace: n, key: key.clone(), ctx: None }
            };
            store.ensure_ns(&addr.project, &addr.namespace, cfg.ns_delim).expect("ensure ns");
            if let Err(e) = store.set(&addr, val, ttl) {
                eprintln!("set failed: {}", e);
                std::process::exit(1);
            }
            println!("ok");
        }
        "get" => {
            if rest.is_empty() { eprintln!("get <k|p.n.k[__ctx]> [--include-expired]"); std::process::exit(1); }
            let include_expired = rest.iter().any(|s| s == "--include-expired");
            let key = &rest[0];
            let addr = if key.contains(cfg.ns_delim) {
                parse_addr(key, cfg.ns_delim).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); })
            } else {
                let p = project.clone().unwrap_or_else(|| "default".into());
                let n = namespace.clone().unwrap_or_else(|| "default".into());
                if key.contains(cfg.ns_delim) { eprintln!("key cannot contain delimiter"); std::process::exit(1); }
                Addr { project: p, namespace: n, key: key.clone(), ctx: None }
            };
            match store.get(&addr, include_expired) {
                Ok(Some(v)) => { print!("{}", String::from_utf8_lossy(&v)); std::process::exit(0); }
                Ok(None) => { eprintln!("not found/expired"); std::process::exit(2); }
                Err(e) => { eprintln!("{}", e); std::process::exit(1); }
            }
        }
        "del" => {
            if rest.is_empty() { eprintln!("del <k|p.n.k[__ctx]>"); std::process::exit(1); }
            let key = &rest[0];
            let addr = if key.contains(cfg.ns_delim) {
                parse_addr(key, cfg.ns_delim).unwrap_or_else(|e| { eprintln!("{}", e); std::process::exit(1); })
            } else {
                let p = project.clone().unwrap_or_else(|| "default".into());
                let n = namespace.clone().unwrap_or_else(|| "default".into());
                if key.contains(cfg.ns_delim) { eprintln!("key cannot contain delimiter"); std::process::exit(1); }
                Addr { project: p, namespace: n, key: key.clone(), ctx: None }
            };
            let n = store.del(&addr).unwrap_or(0);
            println!("{}", n);
        }
        "keys" | "ls" => {
            let p = project.clone().unwrap_or_else(|| "default".into());
            let n = namespace.clone().unwrap_or_else(|| "default".into());
            store.ensure_ns(&p, &n, cfg.ns_delim).expect("ensure ns");
            match store.keys(&p, &n) {
                Ok(list) => for k in list { println!("{}", k); },
                Err(e) => { eprintln!("{}", e); std::process::exit(1); }
            }
        }
        "projects" | "namespaces" | "nss" => {
            if cmd == "projects" {
                let mut st = store.conn.prepare("SELECT DISTINCT project FROM sys_namespaces ORDER BY 1").unwrap();
                let rows = st.query_map([], |r| r.get::<_, String>(0)).unwrap();
                for p in rows { println!("{}", p.unwrap()); }
            } else if cmd == "namespaces" {
                let mut pflag: Option<String> = None;
                let mut k=0; while k < rest.len() { if rest[k]=="-p" && k+1<rest.len() { pflag=Some(rest[k+1].clone()); k+=2; } else { k+=1; } }
                let p = pflag.unwrap_or_else(|| "default".into());
                let mut st = store.conn.prepare("SELECT namespace, kind FROM sys_namespaces WHERE project=?1 ORDER BY 1").unwrap();
                let rows = st.query_map(rusqlite::params![p], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?))).unwrap();
                for r in rows { let (ns,k) = r.unwrap(); println!("{} ({})", ns, k); }
            } else {
                let mut st = store.conn.prepare("SELECT project, namespace, kind FROM sys_namespaces ORDER BY 1,2").unwrap();
                let rows = st.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?, r.get::<_, String>(2)?))).unwrap();
                for r in rows { let (p,n,k) = r.unwrap(); println!("{}.{} ({})", p, n, k); }
            }
        }
        "backup" => {
            let mut out: Option<String> = None;
            let mut j=0; while j < rest.len() { if rest[j]=="--out" && j+1<rest.len() { out=Some(rest[j+1].clone()); j+=2; } else { j+=1; } }
            let out = out.unwrap_or_else(|| "prontodb_backup.tar.gz".into());
            // naive: copy DB to out if no tar
            let dbs = cfg.db_path.to_string_lossy().to_string();
            if std::process::Command::new("tar").arg("--version").output().is_ok() {
                let home = dirs_next::home_dir().unwrap();
                let conf = common::derive_paths(&home, &cfg).conf_file;
                let _ = std::process::Command::new("tar").args(["-czf",&out,&dbs, &conf.to_string_lossy()]).status();
            } else {
                let _ = std::fs::copy(&dbs, &out);
            }
            println!("{}", out);
        }
        "stream" => {
            let mut s = String::new();
            std::io::stdin().read_to_string(&mut s).unwrap();
            let code = stream::handle_stream(&store, &cfg, &s);
            std::process::exit(code);
        }
        _ => usage()
    }
}
