// Core command dispatcher for ProntoDB
// Routes commands to appropriate handlers and manages exit codes

use std::collections::HashMap;

use crate::addressing::{parse_address, Address};
use crate::storage::Storage;
use crate::xdg::XdgPaths;

// Exit codes per TEST-SPEC
pub const EXIT_OK: i32 = 0;
pub const EXIT_MISS: i32 = 2;  // Key not found or expired
pub const EXIT_ERROR: i32 = 1;  // General error

// Command context with parsed arguments
pub struct CommandContext {
    pub command: String,
    pub args: Vec<String>,
    pub flags: HashMap<String, String>,
    pub project: Option<String>,
    pub namespace: Option<String>,
    pub ns_delim: String,
}

impl CommandContext {
    // Parse command line arguments into context
    pub fn from_args(args: Vec<String>) -> Result<Self, String> {
        if args.len() < 2 {
            return Err("No command specified".to_string());
        }

        let mut command = String::new();
        let mut positional = Vec::new();
        let mut flags = HashMap::new();
        let mut project = None;
        let mut namespace = None;
        let mut ns_delim = ".".to_string();  // Default delimiter

        let mut i = 1;  // Skip program name
        while i < args.len() {
            let arg = &args[i];
            
            if arg.starts_with("--") {
                // Long flag
                let flag_name = arg.trim_start_matches("--");
                if flag_name == "ns-delim" && i + 1 < args.len() {
                    ns_delim = args[i + 1].clone();
                    i += 2;
                } else if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                    flags.insert(flag_name.to_string(), args[i + 1].clone());
                    i += 2;
                } else {
                    flags.insert(flag_name.to_string(), "true".to_string());
                    i += 1;
                }
            } else if arg.starts_with("-") {
                // Short flag
                let flag_name = arg.trim_start_matches("-");
                match flag_name {
                    "p" if i + 1 < args.len() => {
                        project = Some(args[i + 1].clone());
                        i += 2;
                    }
                    "n" if i + 1 < args.len() => {
                        namespace = Some(args[i + 1].clone());
                        i += 2;
                    }
                    _ if i + 1 < args.len() && !args[i + 1].starts_with("-") => {
                        flags.insert(flag_name.to_string(), args[i + 1].clone());
                        i += 2;
                    }
                    _ => {
                        flags.insert(flag_name.to_string(), "true".to_string());
                        i += 1;
                    }
                }
            } else if command.is_empty() {
                command = arg.clone();
                i += 1;
            } else {
                positional.push(arg.clone());
                i += 1;
            }
        }

        Ok(CommandContext {
            command,
            args: positional,
            flags,
            project,
            namespace,
            ns_delim,
        })
    }
}

// Main dispatcher function
pub fn dispatch(args: Vec<String>) -> i32 {
    let context = match CommandContext::from_args(args) {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Error: {}", e);
            return EXIT_ERROR;
        }
    };

    // Route to appropriate handler
    match context.command.as_str() {
        // Core KV operations
        "set" => handle_set(context),
        "get" => handle_get(context),
        "del" | "delete" => handle_del(context),
        "keys" => handle_keys(context),
        "scan" => handle_scan(context),
        "ls" => handle_ls(context),

        // TTL/Cache operations
        "create-cache" => handle_create_cache(context),
        
        // Discovery operations
        "projects" => handle_projects(context),
        "namespaces" => handle_namespaces(context),
        "nss" => handle_nss(context),

        // Stream operations
        "stream" => handle_stream(context),

        // Admin operations
        "admin" => handle_admin(context),

        // Lifecycle operations (stub for now)
        "install" => {
            eprintln!("Install not implemented in MVP");
            EXIT_ERROR
        }
        "uninstall" => {
            eprintln!("Uninstall not implemented in MVP");
            EXIT_ERROR
        }
        "backup" => {
            eprintln!("Backup not implemented in MVP");
            EXIT_ERROR
        }

        // Help
        "--help" | "-h" | "help" => {
            print_help();
            EXIT_OK
        }

        // Unknown command
        _ => {
            eprintln!("Unknown command: {}", context.command);
            eprintln!("Try 'prontodb --help' for usage information");
            EXIT_ERROR
        }
    }
}

// Implemented handlers

fn handle_set(ctx: CommandContext) -> i32 {
    if ctx.args.len() < 2 {
        eprintln!("Usage: set <path|key> <value> [--ttl SECONDS]");
        return EXIT_ERROR;
    }

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();

    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    let key_or_path = &ctx.args[0];
    let value = &ctx.args[1];

    // Parse address: if it contains delimiter (dot/piped variant), treat as full path.
    // Otherwise use -p/-n + key, preserving optional __context suffix.
    let addr = if key_or_path.contains(&ctx.ns_delim) {
        match Address::parse(key_or_path, &ctx.ns_delim) {
            Ok(a) => a,
            Err(e) => { eprintln!("{}", e); return EXIT_ERROR; }
        }
    } else {
        // extract optional __context from key
        let (key, context) = if let Some(idx) = key_or_path.rfind("__") {
            let k = &key_or_path[..idx];
            let ctxs = &key_or_path[idx+2..];
            (k.to_string(), if ctxs.is_empty() { None } else { Some(ctxs.to_string()) })
        } else {
            (key_or_path.to_string(), None)
        };
        Address::from_parts(ctx.project.clone(), ctx.namespace.clone(), key, context)
    };

    // Validate key against active delimiter
    if let Err(e) = addr.validate_key(&ctx.ns_delim) {
        eprintln!("{}", e);
        return EXIT_ERROR;
    }

    // TTL handling
    let ttl_flag = ctx.flags.get("ttl").and_then(|s| s.parse::<u64>().ok());
    let ns_default_ttl = match storage.get_namespace_ttl(&addr.project, &addr.namespace) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Failed to query namespace TTL: {}", e);
            return EXIT_ERROR;
        }
    };

    let effective_ttl = match (ns_default_ttl, ttl_flag) {
        (Some(_default), Some(ttl)) => Some(ttl),              // explicit ttl in TTL ns
        (Some(default), None) => Some(default),                 // default ttl for TTL ns
        (None, Some(_)) => {
            eprintln!("TTL not allowed: namespace is not TTL-enabled");
            return EXIT_ERROR;
        }
        (None, None) => None,                                  // no ttl in standard ns
    };

    if let Err(e) = storage.set(&addr, value, effective_ttl) {
        eprintln!("Failed to set value: {}", e);
        return EXIT_ERROR;
    }
    EXIT_OK
}

fn handle_get(ctx: CommandContext) -> i32 {
    if ctx.args.is_empty() {
        eprintln!("Usage: get <path|key>");
        return EXIT_ERROR;
    }

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    let key_or_path = &ctx.args[0];
    let addr = if key_or_path.contains(&ctx.ns_delim) {
        match Address::parse(key_or_path, &ctx.ns_delim) {
            Ok(a) => a,
            Err(e) => { eprintln!("{}", e); return EXIT_ERROR; }
        }
    } else {
        let (key, context) = if let Some(idx) = key_or_path.rfind("__") {
            let k = &key_or_path[..idx];
            let ctxs = &key_or_path[idx+2..];
            (k.to_string(), if ctxs.is_empty() { None } else { Some(ctxs.to_string()) })
        } else { (key_or_path.to_string(), None) };
        Address::from_parts(ctx.project.clone(), ctx.namespace.clone(), key, context)
    };

    match storage.get(&addr) {
        Ok(Some(val)) => {
            println!("{}", val);
            EXIT_OK
        }
        Ok(None) => {
            // MISS (not found or expired and removed)
            EXIT_MISS
        }
        Err(e) => {
            eprintln!("Failed to get value: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_del(ctx: CommandContext) -> i32 {
    if ctx.args.is_empty() {
        eprintln!("Usage: del <path|key>");
        return EXIT_ERROR;
    }

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    let key_or_path = &ctx.args[0];
    let addr = if key_or_path.contains(&ctx.ns_delim) {
        match Address::parse(key_or_path, &ctx.ns_delim) {
            Ok(a) => a,
            Err(e) => { eprintln!("{}", e); return EXIT_ERROR; }
        }
    } else {
        let (key, context) = if let Some(idx) = key_or_path.rfind("__") {
            let k = &key_or_path[..idx];
            let ctxs = &key_or_path[idx+2..];
            (k.to_string(), if ctxs.is_empty() { None } else { Some(ctxs.to_string()) })
        } else { (key_or_path.to_string(), None) };
        Address::from_parts(ctx.project.clone(), ctx.namespace.clone(), key, context)
    };

    if let Err(e) = storage.delete(&addr) {
        eprintln!("Failed to delete: {}", e);
        return EXIT_ERROR;
    }
    EXIT_OK
}

fn handle_keys(ctx: CommandContext) -> i32 {
    // Require -p and -n (explicit)
    let project = match &ctx.project {
        Some(p) => p,
        None => {
            eprintln!("keys requires -p <project> and -n <namespace>");
            return EXIT_ERROR;
        }
    };
    let namespace = match &ctx.namespace {
        Some(n) => n,
        None => {
            eprintln!("keys requires -p <project> and -n <namespace>");
            return EXIT_ERROR;
        }
    };
    let prefix = ctx.args.get(0).map(|s| s.as_str());

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    match storage.list_keys(project, namespace, prefix) {
        Ok(keys) => {
            for k in keys {
                println!("{}", k);
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("Failed to list keys: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_scan(ctx: CommandContext) -> i32 {
    // Require -p and -n (explicit)
    let project = match &ctx.project {
        Some(p) => p,
        None => {
            eprintln!("scan requires -p <project> and -n <namespace>");
            return EXIT_ERROR;
        }
    };
    let namespace = match &ctx.namespace {
        Some(n) => n,
        None => {
            eprintln!("scan requires -p <project> and -n <namespace>");
            return EXIT_ERROR;
        }
    };
    let prefix = ctx.args.get(0).map(|s| s.as_str());

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    match storage.scan(project, namespace, prefix) {
        Ok(pairs) => {
            for (k, v) in pairs {
                println!("{}={}", k, v);
            }
            EXIT_OK
        }
        Err(e) => {
            eprintln!("Failed to scan: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_ls(ctx: CommandContext) -> i32 {
    // Alias to scan for MVP
    handle_scan(ctx)
}

fn handle_create_cache(ctx: CommandContext) -> i32 {
    if ctx.args.len() < 2 {
        eprintln!("Usage: create-cache <project.namespace> timeout=SECONDS");
        return EXIT_ERROR;
    }

    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };

    // Parse namespace path (project.namespace)
    let ns_path = &ctx.args[0];
    let parts: Vec<&str> = ns_path.split(&ctx.ns_delim).collect();
    if parts.len() != 2 {
        eprintln!("Namespace must be in form project{}namespace", ctx.ns_delim);
        return EXIT_ERROR;
    }
    let project = parts[0];
    let namespace = parts[1];

    // Parse timeout
    let mut timeout_opt: Option<u64> = None;
    if let Some(v) = ctx.flags.get("timeout") {
        timeout_opt = v.parse::<u64>().ok();
    }
    if timeout_opt.is_none() {
        // try arg2: key=value
        if let Some(arg2) = ctx.args.get(1) {
            if let Some(eq) = arg2.find('=') {
                let (k, v) = arg2.split_at(eq);
                if k == "timeout" {
                    timeout_opt = v[1..].parse::<u64>().ok();
                }
            }
        }
    }

    let timeout = match timeout_opt {
        Some(t) if t > 0 => t,
        _ => {
            eprintln!("Invalid or missing timeout; expected timeout=SECONDS > 0");
            return EXIT_ERROR;
        }
    };

    match storage.create_ttl_namespace(project, namespace, timeout) {
        Ok(()) => EXIT_OK,
        Err(e) => {
            eprintln!("Failed to create TTL namespace: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_projects(_ctx: CommandContext) -> i32 {
    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to open db: {}", e);
            return EXIT_ERROR;
        }
    };
    match storage.list_projects() {
        Ok(list) => {
            for p in list { println!("{}", p); }
            EXIT_OK
        }
        Err(e) => { eprintln!("{}", e); EXIT_ERROR }
    }
}

fn handle_namespaces(ctx: CommandContext) -> i32 {
    let project = match &ctx.project {
        Some(p) => p,
        None => {
            eprintln!("namespaces requires -p <project>");
            return EXIT_ERROR;
        }
    };
    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Failed to open db: {}", e); return EXIT_ERROR; }
    };
    match storage.list_namespaces(project) {
        Ok(list) => { for n in list { println!("{}", n); } EXIT_OK }
        Err(e) => { eprintln!("{}", e); EXIT_ERROR }
    }
}

fn handle_nss(ctx: CommandContext) -> i32 {
    // Aggregate namespaces across projects
    let paths = XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("Failed to ensure XDG dirs: {}", e);
        return EXIT_ERROR;
    }
    let db_path = paths.get_db_path();
    let storage = match Storage::open(&db_path) {
        Ok(s) => s,
        Err(e) => { eprintln!("Failed to open db: {}", e); return EXIT_ERROR; }
    };
    let projects = match storage.list_projects() { Ok(p) => p, Err(e) => { eprintln!("{}", e); return EXIT_ERROR; } };
    for p in projects {
        match storage.list_namespaces(&p) {
            Ok(list) => { for n in list { println!("{}{}{}", p, ctx.ns_delim, n); } }
            Err(e) => { eprintln!("{}", e); return EXIT_ERROR; }
        }
    }
    EXIT_OK
}

fn handle_stream(_ctx: CommandContext) -> i32 {
    // MVP: streams not implemented; security required by default
    eprintln!("stream: security/auth required (feature deferred)");
    EXIT_ERROR
}

fn handle_admin(ctx: CommandContext) -> i32 {
    // TODO: Admin subcommands
    if ctx.args.is_empty() {
        eprintln!("Admin requires subcommand");
        return EXIT_ERROR;
    }
    
    match ctx.args[0].as_str() {
        "create-cache" => handle_create_cache(ctx),
        _ => {
            eprintln!("Unknown admin subcommand: {}", ctx.args[0]);
            EXIT_ERROR
        }
    }
}

fn print_help() {
    println!("ProntoDB - Fast key-value store with TTL support");
    println!();
    println!("Usage: prontodb <command> [options] [args]");
    println!();
    println!("Core Commands:");
    println!("  set <path|key> <value>    Store a value");
    println!("  get <path|key>             Retrieve a value");
    println!("  del <path|key>             Delete a value");
    println!("  keys [prefix]              List keys");
    println!("  scan [prefix]              Scan key-value pairs");
    println!();
    println!("Addressing Options:");
    println!("  -p <project>               Set project");
    println!("  -n <namespace>             Set namespace");
    println!("  --ns-delim <delim>         Override delimiter (default: '.')");
    println!();
    println!("Stream Operations:");
    println!("  stream                     Process token stream from stdin");
    println!();
    println!("Exit Codes:");
    println!("  0 = Success");
    println!("  2 = Key not found (MISS)");
    println!("  1 = Error");
}
