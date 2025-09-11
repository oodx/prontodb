// Core command dispatcher for ProntoDB
// Routes commands to appropriate handlers and manages exit codes
// Note: This module is being phased out in favor of RSB architecture,
// but is still used as a bridge from RSB handlers in lib.rs

#![allow(dead_code)]  // Temporary while transitioning to full RSB

use std::collections::HashMap;

use crate::api::{self, SetValueConfig};
use crate::pipe_cache;
use crate::validation;

use crate::cursor_cache::CursorCache;

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
    pub cursor: Option<String>,
    pub user: String,
    pub database: String,
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
        let mut cursor = None;
        let mut user = "default".to_string();  // Default user
        let mut database = "main".to_string();  // Default database
        let mut explicit_database_flag = false;  // Track if --database was explicitly set

        let mut i = 1;  // Skip program name
        while i < args.len() {
            let arg = &args[i];
            
            if arg.starts_with("--") {
                // Long flag
                let flag_name = arg.trim_start_matches("--");
                if flag_name == "ns-delim" && i + 1 < args.len() {
                    ns_delim = args[i + 1].clone();
                    i += 2;
                } else if flag_name == "cursor" && i + 1 < args.len() {
                    cursor = Some(args[i + 1].clone());
                    i += 2;
                } else if flag_name == "user" && i + 1 < args.len() {
                    let user_value = args[i + 1].clone();
                    if let Err(e) = validation::validate_username(&user_value) {
                        return Err(format!("Error: {}", e));
                    }
                    user = user_value;
                    i += 2;
                } else if flag_name == "database" && i + 1 < args.len() {
                    database = args[i + 1].clone();
                    explicit_database_flag = true;
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

        // Auto-selection logic: Check cursor cache if no explicit database flag was provided
        // AND no explicit cursor was specified (cursor takes precedence over cache)
        if !explicit_database_flag && cursor.is_none() {
            let cache = CursorCache::new();
            
            // Determine which user to check for cursor cache
            let cache_user = if user == "default" { None } else { Some(user.as_str()) };
            
            if let Some(cached_database) = cache.get_cursor(cache_user) {
                database = cached_database;
            }
        }

        Ok(CommandContext {
            command,
            args: positional,
            flags,
            project,
            namespace,
            ns_delim,
            cursor,
            user,
            database,
        })
    }
}

// Main dispatcher function
pub fn dispatch(args: Vec<String>) -> i32 {
    // Handle version early (before parsing context)
    if args.len() == 2 && (args[1] == "--version" || args[1] == "-v" || args[1] == "version") {
        print_version();
        return EXIT_OK;
    }

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
        
        // Cursor operations
        "cursor" => handle_cursor(context),
        
        // Discovery operations
        "projects" => handle_projects(context),
        "namespaces" => handle_namespaces(context),
        "nss" => handle_nss(context),

        // Stream operations
        "stream" => handle_stream(context),

        // Copy operations
        "copy" => handle_copy(context),

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
            eprintln!("Backup command handled by pre-dispatcher");
            EXIT_ERROR
        }

        // Version
        "--version" | "-v" | "version" => {
            print_version();
            EXIT_OK
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
    if ctx.args.is_empty() {
        eprintln!("Usage: set <path|key> [value] [--ttl SECONDS]");
        eprintln!("  If no value provided, will attempt to read from piped input");
        return EXIT_ERROR;
    }

    let key_or_path = &ctx.args[0];
    
    // Check for piped input first (before consuming args)
    let piped_content = pipe_cache::detect_pipe_input();
    
    let value = if ctx.args.len() >= 2 {
        // Value provided as parameter
        &ctx.args[1]
    } else if let Some(ref content) = piped_content {
        // No value parameter, but we have piped content - use it directly
        content
    } else {
        // No value parameter and no piped input
        eprintln!("Usage: set <path|key> <value> [--ttl SECONDS]");
        eprintln!("  Or pipe content: echo 'data' | prontodb set <address>");
        return EXIT_ERROR;
    };
    let ttl_flag = ctx.flags.get("ttl").and_then(|s| s.parse::<u64>().ok());
    let config = SetValueConfig {
        project: ctx.project.as_deref(),
        namespace: ctx.namespace.as_deref(),
        key_or_path,
        value,
        ns_delim: &ctx.ns_delim,
        ttl_flag,
        cursor_name: ctx.cursor.as_deref(),
        user: &ctx.user,
        database: &ctx.database,
    };
    if let Err(e) = api::set_value_with_cursor(config) {
        // If we have piped content and the address is invalid, create pipe cache entry with education
        if let Some(ref content) = piped_content {
            let (cache_key, cached_content, education) = 
                pipe_cache::prepare_pipe_cache_with_education(content, key_or_path);
            
            // Store the cached content
            let cache_config = SetValueConfig {
                project: None,
                namespace: None,
                key_or_path: &cache_key,
                value: &cached_content,
                ns_delim: ".",
                ttl_flag: Some(pipe_cache::DEFAULT_PIPE_CACHE_TTL),
                cursor_name: ctx.cursor.as_deref(),
                user: &ctx.user,
                database: &ctx.database,
            };
            
            match api::set_value_with_cursor(cache_config) {
                Ok(()) => {
                    // Provide user feedback with progressive education
                    eprintln!("⚠️  Invalid address '{}' - content cached as: {}", key_or_path, cache_key);
                    eprintln!("💡 Use: prontodb copy {} <proper.address>", cache_key);
                    
                    // Provide XStream education if applicable
                    if let Some(suggestion) = education {
                        eprintln!("{}", suggestion);
                    }
                    return EXIT_OK;
                }
                Err(cache_err) => {
                    eprintln!("Warning: Failed to cache piped content: {}", cache_err);
                    eprintln!("Original error: {}", e);
                    return EXIT_ERROR;
                }
            }
        } else {
            // No piped input, show original error
            eprintln!("{}", e);
            return EXIT_ERROR;
        }
    }
    EXIT_OK
}

fn handle_get(ctx: CommandContext) -> i32 {
    if ctx.args.is_empty() {
        eprintln!("Usage: get <path|key>");
        return EXIT_ERROR;
    }

    let key_or_path = &ctx.args[0];
    match api::get_value_with_cursor_and_database(
        ctx.project.as_deref(),
        ctx.namespace.as_deref(),
        key_or_path,
        &ctx.ns_delim,
        ctx.cursor.as_deref(),
        &ctx.user,
        &ctx.database,
    ) {
        Ok(Some(val)) => {
            println!("{}", val);
            EXIT_OK
        }
        Ok(None) => EXIT_MISS,
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

    let key_or_path = &ctx.args[0];
    if let Err(e) = api::delete_value_with_cursor_and_database(
        ctx.project.as_deref(),
        ctx.namespace.as_deref(),
        key_or_path,
        &ctx.ns_delim,
        ctx.cursor.as_deref(),
        &ctx.user,
        &ctx.database,
    ) {
        eprintln!("Failed to delete: {}", e);
        return EXIT_ERROR;
    }
    EXIT_OK
}

fn handle_keys(ctx: CommandContext) -> i32 {
    // Support both explicit -p/-n flags and dot addressing
    if ctx.project.is_some() && ctx.namespace.is_some() {
        // Use explicit flags with optional prefix
        let project = ctx.project.as_ref().unwrap();
        let namespace = ctx.namespace.as_ref().unwrap();
        let prefix = ctx.args.first().map(|s| s.as_str());
        
        match api::list_keys_with_cursor_and_database(project, namespace, prefix, ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
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
    } else if !ctx.args.is_empty() {
        // Use dot addressing from first argument
        let path_or_prefix = &ctx.args[0];
        
        match api::list_keys_flexible_with_database(
            ctx.project.as_deref(),
            ctx.namespace.as_deref(),
            path_or_prefix,
            &ctx.ns_delim,
            ctx.cursor.as_deref(),
            &ctx.user,
            &ctx.database,
        ) {
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
    } else {
        eprintln!("Usage: keys <project.namespace[.prefix]> OR keys -p <project> -n <namespace> [prefix]");
        EXIT_ERROR
    }
}

fn handle_scan(ctx: CommandContext) -> i32 {
    // Support both explicit -p/-n flags and dot addressing
    if ctx.project.is_some() && ctx.namespace.is_some() {
        // Use explicit flags with optional prefix
        let project = ctx.project.as_ref().unwrap();
        let namespace = ctx.namespace.as_ref().unwrap();
        let prefix = ctx.args.first().map(|s| s.as_str());
        
        match api::scan_pairs_with_cursor_and_database(project, namespace, prefix, ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
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
    } else if !ctx.args.is_empty() {
        // Use dot addressing from first argument
        let path_or_prefix = &ctx.args[0];
        
        match api::scan_pairs_flexible_with_database(
            ctx.project.as_deref(),
            ctx.namespace.as_deref(),
            path_or_prefix,
            &ctx.ns_delim,
            ctx.cursor.as_deref(),
            &ctx.user,
            &ctx.database,
        ) {
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
    } else {
        eprintln!("Usage: scan <project.namespace[.prefix]> OR scan -p <project> -n <namespace> [prefix]");
        EXIT_ERROR
    }
}

fn handle_ls(ctx: CommandContext) -> i32 {
    // Alias to scan for MVP
    handle_scan(ctx)
}

fn handle_create_cache(ctx: CommandContext) -> i32 {
    if ctx.args.len() < 2 {
        eprintln!("Usage: create-cache <project.namespace> <timeout_seconds>");
        return EXIT_ERROR;
    }

    // Parse namespace path (project.namespace)
    let ns_path = &ctx.args[0];
    let parts: Vec<&str> = ns_path.split(&ctx.ns_delim).collect();
    if parts.len() != 2 {
        eprintln!("Namespace must be in form project{}namespace", ctx.ns_delim);
        return EXIT_ERROR;
    }
    let project = parts[0];
    let namespace = parts[1];

    // Parse timeout from second positional argument
    let timeout = match ctx.args[1].parse::<u64>() {
        Ok(t) if t > 0 => t,
        Ok(_) => {
            eprintln!("Timeout must be greater than 0");
            return EXIT_ERROR;
        }
        Err(_) => {
            eprintln!("Invalid timeout value: '{}'. Must be a positive number", ctx.args[1]);
            return EXIT_ERROR;
        }
    };

    match api::create_ttl_namespace_with_cursor_and_database(project, namespace, timeout, ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
        Ok(()) => EXIT_OK,
        Err(e) => {
            eprintln!("Failed to create TTL namespace: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_projects(ctx: CommandContext) -> i32 {
    match api::projects_with_cursor_and_database(ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
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
    match api::namespaces_with_cursor_and_database(project, ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
        Ok(list) => { for n in list { println!("{}", n); } EXIT_OK }
        Err(e) => { eprintln!("{}", e); EXIT_ERROR }
    }
}

fn handle_nss(ctx: CommandContext) -> i32 {
    // Aggregate namespaces across projects
    let projects = match api::projects_with_cursor_and_database(ctx.cursor.as_deref(), &ctx.user, &ctx.database) { Ok(p) => p, Err(e) => { eprintln!("{}", e); return EXIT_ERROR; } };
    for p in projects {
        match api::namespaces_with_cursor_and_database(&p, ctx.cursor.as_deref(), &ctx.user, &ctx.database) {
            Ok(list) => { for n in list { println!("{}{}{}", p, ctx.ns_delim, n); } }
            Err(e) => { eprintln!("{}", e); return EXIT_ERROR; }
        }
    }
    EXIT_OK
}

fn handle_stream(_ctx: CommandContext) -> i32 {
    use crate::streaming;
    
    // Check if streaming feature is enabled
    if !streaming::is_streaming_enabled() {
        eprintln!("Streaming feature not enabled. Build with '--features streaming' to enable XStream integration.");
        eprintln!("Use: cargo build --features streaming");
        return EXIT_ERROR;
    }
    
    // Handle the stream command
    match streaming::handle_stream_command() {
        Ok(()) => EXIT_OK,
        Err(e) => {
            eprintln!("Stream error: {}", e);
            EXIT_ERROR
        }
    }
}

fn handle_copy(ctx: CommandContext) -> i32 {
    if ctx.args.len() < 2 {
        eprintln!("Usage: copy <source> <destination>");
        eprintln!("  Copy value from source address to destination address");
        eprintln!("  Automatically cleans up pipe cache entries after successful copy");
        return EXIT_ERROR;
    }

    let source = &ctx.args[0];
    let destination = &ctx.args[1];
    
    // Get the source value
    let source_value = match api::get_value_with_cursor_and_database(
        ctx.project.as_deref(),
        ctx.namespace.as_deref(),
        source,
        &ctx.ns_delim,
        ctx.cursor.as_deref(),
        &ctx.user,
        &ctx.database,
    ) {
        Ok(Some(value)) => value,
        Ok(None) => {
            eprintln!("Source '{}' not found", source);
            return EXIT_ERROR;
        }
        Err(e) => {
            eprintln!("Failed to read source '{}': {}", source, e);
            return EXIT_ERROR;
        }
    };
    
    // Set the destination value
    let set_config = SetValueConfig {
        project: ctx.project.as_deref(),
        namespace: ctx.namespace.as_deref(),
        key_or_path: destination,
        value: &source_value,
        ns_delim: &ctx.ns_delim,
        ttl_flag: None, // Don't inherit TTL in copy operations
        cursor_name: ctx.cursor.as_deref(),
        user: &ctx.user,
        database: &ctx.database,
    };
    
    if let Err(e) = api::set_value_with_cursor(set_config) {
        eprintln!("Failed to copy to destination '{}': {}", destination, e);
        return EXIT_ERROR;
    }
    
    // If source was a pipe cache entry, clean it up
    if source.starts_with("pipe.cache.") {
        match api::delete_value_with_cursor_and_database(
            ctx.project.as_deref(),
            ctx.namespace.as_deref(), 
            source,
            &ctx.ns_delim,
            ctx.cursor.as_deref(),
            &ctx.user,
            &ctx.database,
        ) {
            Ok(()) => {
                println!("✅ Copied '{}' to '{}' and cleaned up cache", source, destination);
            }
            Err(e) => {
                println!("✅ Copied '{}' to '{}'", source, destination);
                eprintln!("⚠️  Warning: Failed to cleanup cache entry: {}", e);
            }
        }
    } else {
        println!("✅ Copied '{}' to '{}'", source, destination);
    }
    
    EXIT_OK
}

fn handle_cursor(ctx: CommandContext) -> i32 {
    use crate::cursor_cache::CursorCache;
    use crate::cursor::CursorManager;
    use std::path::PathBuf;
    
    if ctx.args.is_empty() {
        eprintln!("cursor: Missing subcommand");
        eprintln!("Usage: prontodb [--user <user>] cursor set <name> <path> [--meta <context>]");
        eprintln!("   or: prontodb [--user <user>] cursor list");
        eprintln!("   or: prontodb [--user <user>] cursor clear");
        eprintln!("   or: prontodb [--user <user>] cursor active");
        eprintln!("   or: prontodb [--user <user>] cursor delete <name>");
        return EXIT_ERROR;
    }
    
    let cache = CursorCache::new();
    let cursor_manager = CursorManager::new();
    
    match ctx.args[0].as_str() {
        "set" => {
            // Enhanced cursor management: cursor set <name> <path> [--meta <context>]
            if ctx.args.len() < 3 {
                eprintln!("cursor set: Missing cursor name or database path");
                eprintln!("Usage: prontodb [--user <user>] cursor set <name> <path> [--meta <context>]");
                return EXIT_ERROR;
            }
            
            let cursor_name = &ctx.args[1];
            let db_path = PathBuf::from(&ctx.args[2]);
            
            // Parse optional --meta flag from flags map
            let meta_context = if let Some(meta_value) = ctx.flags.get("meta") {
                // Validate meta context using project name validation (same rules)
                if let Err(e) = crate::validation::validate_project_name(meta_value) {
                    eprintln!("Error: Invalid meta context '{}': {}", meta_value, e);
                    return EXIT_ERROR;
                }
                Some(meta_value.clone())
            } else {
                None
            };
            
            // Set cursor using CursorManager
            cursor_manager.set_cursor_with_meta(
                cursor_name,
                db_path.clone(),
                &ctx.user,
                meta_context.clone(),
                None, // default_project
                None, // default_namespace
            );
            
            match meta_context {
                Some(meta) => println!("Cursor '{}' set to '{}' with meta context '{}'", cursor_name, db_path.display(), meta),
                None => println!("Cursor '{}' set to '{}'", cursor_name, db_path.display()),
            }
            EXIT_OK
        }
        
        "list" => {
            // Show both cache cursors and persistent cursors (filtered by user)
            let user_cache_cursor = cache.get_cursor(Some(&ctx.user));
            let persistent_cursors = cursor_manager.list_cursors(&ctx.user);
            
            println!("Cursor Management:");
            println!();
            
            let cache_empty = user_cache_cursor.is_none();
            if !cache_empty {
                println!("Cache Cursors (lightweight database selection):");
                if let Some(database) = user_cache_cursor {
                    println!("  {}: {}", ctx.user, database);
                }
                println!();
            }
            
            match persistent_cursors {
                Ok(cursors) if !cursors.is_empty() => {
                    println!("Persistent Cursors (enhanced with meta context):");
                    for (cursor_name, cursor_data) in cursors {
                        let meta_info = match &cursor_data.meta_context {
                            Some(meta) => format!(" [meta: {}]", meta),
                            None => String::new(),
                        };
                        println!("  {}: {}{}", cursor_name, cursor_data.database_path.display(), meta_info);
                    }
                }
                Ok(_) => {
                    if cache_empty {
                        println!("No cursors found. Use 'cursor set' to create persistent cursors.");
                    }
                }
                Err(e) => {
                    eprintln!("cursor list: Failed to list persistent cursors: {}", e);
                    return EXIT_ERROR;
                }
            }
            EXIT_OK
        }
        
        "delete" => {
            if ctx.args.len() < 2 {
                eprintln!("cursor delete: Missing cursor name");
                eprintln!("Usage: prontodb [--user <user>] cursor delete <name>");
                return EXIT_ERROR;
            }
            
            let cursor_name = &ctx.args[1];
            match cursor_manager.delete_cursor(cursor_name, &ctx.user) {
                Ok(true) => {
                    println!("Cursor '{}' deleted for user '{}'", cursor_name, ctx.user);
                    EXIT_OK
                }
                Ok(false) => {
                    eprintln!("cursor delete: Cursor '{}' not found for user '{}'", cursor_name, ctx.user);
                    EXIT_ERROR
                }
                Err(e) => {
                    eprintln!("cursor delete: Failed to delete cursor: {}", e);
                    EXIT_ERROR
                }
            }
        }
        
        "clear" => {
            let cache_user = if ctx.user == "default" { None } else { Some(ctx.user.as_str()) };
            match cache.clear_cursor(cache_user) {
                Ok(()) => {
                    println!("Cursor cache cleared for user '{}'", ctx.user);
                    EXIT_OK
                }
                Err(e) => {
                    eprintln!("cursor clear: Failed to clear cursor cache: {}", e);
                    EXIT_ERROR
                }
            }
        }
        
        "active" => {
            // Display current active cursor
            let cache_user = if ctx.user == "default" { None } else { Some(ctx.user.as_str()) };
            match cache.get_cursor(cache_user) {
                Some(database) => {
                    println!("Current cursor: {} (for user '{}')", database, ctx.user);
                    EXIT_OK
                }
                None => {
                    println!("No active cursor set (using default database)");
                    EXIT_OK
                }
            }
        }
        
        database_name => {
            // Set cursor to database name
            let cache_user = if ctx.user == "default" { None } else { Some(ctx.user.as_str()) };
            match cache.set_cursor(database_name, cache_user) {
                Ok(()) => {
                    println!("Global cursor set to '{}' for user '{}'", database_name, ctx.user);
                    EXIT_OK
                }
                Err(e) => {
                    eprintln!("cursor: Failed to set cursor: {}", e);
                    EXIT_ERROR
                }
            }
        }
    }
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


fn print_version() {
    // ASCII art logo
    println!("                                                        ");
    println!(" ▄▄▄▄▄                         ▄           ▄▄▄▄   ▄▄▄▄▄ ");
    println!(" █   ▀█  ▄ ▄▄   ▄▄▄   ▄ ▄▄   ▄▄█▄▄   ▄▄▄   █   ▀▄ █    █");
    println!(" █▄▄▄█▀  █▀  ▀ █▀ ▀█  █▀  █    █    █▀ ▀█  █    █ █▄▄▄▄▀");
    println!(" █       █     █   █  █   █    █    █   █  █    █ █    █");
    println!(" █       █     ▀█▄█▀  █   █    ▀▄▄  ▀█▄█▀  █▄▄▄▀  █▄▄▄▄▀");
    println!("                                                        ");
    
    println!("prontodb {}", env!("CARGO_PKG_VERSION"));
    println!("License: {}", env!("CARGO_PKG_LICENSE"));
}

fn print_help() {
    println!("ProntoDB - Fast key-value store with TTL support");
    println!();
    println!("Usage: prontodb <command> [options] [args]");
    println!();
    println!("Core Commands:");
    println!("  set <path|key> <value> [--ttl SECONDS]  Store a value");
    println!("  get <path|key>                          Retrieve a value");
    println!("  del <path|key>                          Delete a value");
    println!("  keys [prefix]                           List keys (supports dot addressing or -p/-n)");
    println!("  scan [prefix]                           Scan key-value pairs (supports dot addressing or -p/-n)");
    println!("  ls [prefix]                             Alias for scan");
    println!();
    println!("Discovery Commands:");
    println!("  projects                                List all projects");
    println!("  namespaces -p <project>                 List namespaces in project");
    println!("  nss                                     List all project.namespace combinations");
    println!();
    println!("TTL Commands:");
    println!("  create-cache <project.namespace> <ttl_seconds>    Create TTL namespace");
    println!();
    println!("Cursor Commands:");
    println!("  cursor <database>                              Set global database cursor");
    println!("  cursor list                                    List all cursor cache entries");
    println!("  cursor clear                                   Clear cursor cache for user");
    println!("  cursor set <name> <path> [--meta <context>]    Create persistent cursor with meta namespace");
    println!("  cursor active                                  Show active cursor information");
    println!("  cursor delete <name>                           Delete persistent cursor");
    println!();
    println!("Admin Commands:");
    println!("  admin <subcommand>                      Admin operations");
    println!();
    println!("Addressing Options:");
    println!("  -p <project>               Set project");
    println!("  -n <namespace>             Set namespace");
    println!("  --ns-delim <delim>         Override delimiter (default: '.')");
    println!("  --database <name>          Set database (default: 'main')");
    println!();
    println!("Addressing Formats:");
    println!("  project.namespace.key      Full path addressing");
    println!("  key__context              Context suffix (stored as context column)");
    println!("  Meta namespace (when cursor has --meta context):");
    println!("    User types: project.namespace.key (3-layer)");
    println!("    System stores: meta.project.namespace.key (4-layer)");
    println!();
    println!("Stream Operations:");
    println!("  stream                     Process token stream from stdin (deferred)");
    println!();
    println!("Lifecycle Commands (RSB):");
    println!("  install                    Install ProntoDB (deferred)");
    println!("  uninstall                  Uninstall ProntoDB (deferred)");
    println!("  backup                     Backup data (deferred)");
    println!();
    println!("Exit Codes:");
    println!("  0 = Success");
    println!("  2 = Key not found (MISS)");
    println!("  1 = Error");
    println!();
    println!("Examples:");
    println!("  prontodb -p myapp -n config set debug true");
    println!("  prontodb get myapp.config.debug");
    println!("  prontodb set myapp.cache.user__prod 'active'");
    println!("  prontodb create-cache myapp.sessions 3600");
    println!("  prontodb --database test get myapp.config.debug");
    println!("  prontodb cursor staging                    # Set global cursor");
    println!("  prontodb cursor prod --user alice          # Set alice's cursor");
    println!("  prontodb --database staging backup");
}
