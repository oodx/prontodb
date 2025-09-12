// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with RSB lifecycle

mod dispatcher;
mod addressing;
mod storage;
mod xdg;
mod api;
mod commands;
mod cursor;
mod cursor_cache;
mod pipe_cache;
mod streaming;
mod validation;

// Use RSB prelude for macros (bootstrap!/pre_dispatch!/dispatch!)
use rsb::prelude::*;


// Import RSB command handlers
use prontodb::{do_set, do_get, do_del, do_keys, do_scan, do_ls, do_create_cache, do_projects, do_namespaces, do_nss, do_stream, do_copy, do_admin, do_help, do_version, do_cursor, do_noop};







fn main() {
    // Check for version and help flags first (highest priority)
    let raw_args: Vec<String> = std::env::args().collect();
    
    // Handle version flags early
    if raw_args.iter().any(|arg| arg == "--version" || arg == "-v") {
        prontodb::do_version(rsb::args::Args::new(&[]));
        std::process::exit(0);
    }
    
    // Handle help flags early (but after version)
    if raw_args.iter().any(|arg| arg == "--help" || arg == "-h") {
        prontodb::do_help(rsb::args::Args::new(&[]));
        std::process::exit(0);
    }
    
    // If we find global flags, intercept and handle them
    if raw_args.iter().any(|arg| arg == "--cursor" || arg == "--user" || arg == "--database") {
        // Handle global flag parsing and command execution directly
        if let Some(exit_code) = handle_global_flags_and_execute(raw_args) {
            std::process::exit(exit_code);
        }
    }
    
    // RSB canonical lifecycle pattern for normal commands (without global flags)
    let args = bootstrap!();           // RSB initialization
    
    // Pre-dispatch for lifecycle commands (install/uninstall/backup/restore)
    if pre_dispatch!(&args, {         // Use Args type, not Vec<String>
        "install" => do_install,       // RSB naming convention
        "uninstall" => do_uninstall,
        "backup" => do_backup,
        "restore" => do_restore
    }) {
        return;  // RSB handles exit automatically
    }
    
    // Load ProntoDB configuration files  
    info!("Loading ProntoDB configuration...");
    src!("~/.config/prontodb/config.conf", "./prontodb.conf");
    
    // RSB standard dispatch - replaces custom dispatcher
    dispatch!(&args, {
        "set" => do_set,
        "get" => do_get,
        "del" => do_del,
        "keys" => do_keys,
        "scan" => do_scan,
        "ls" => do_ls,
        "create-cache" => do_create_cache,
        "projects" => do_projects,
        "namespaces" => do_namespaces,
        "nss" => do_nss,
        "stream" => do_stream,
        "copy" => do_copy,
        "admin" => do_admin,
        "cursor" => do_cursor,
        "noop" => do_noop,
        "version" => do_version,
        "help" => do_help
    });
    // No manual exit - RSB dispatch! handles it
}



