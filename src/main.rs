// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with RSB lifecycle

mod dispatcher;
mod addressing;
mod storage;
mod xdg;
mod api;

// Use RSB prelude for macros (bootstrap!/pre_dispatch!/dispatch!)
use rsb::prelude::*;

// Import RSB command handlers
use prontodb::{do_set, do_get, do_del, do_keys, do_scan, do_ls, do_create_cache, do_projects, do_namespaces, do_nss, do_stream, do_admin};

// RSB lifecycle command handlers with proper naming convention
fn do_install(_args: rsb::args::Args) -> i32 {
    eprintln!("Install not implemented in MVP");
    1
}

fn do_uninstall(_args: rsb::args::Args) -> i32 {
    eprintln!("Uninstall not implemented in MVP");
    1
}

fn do_backup(_args: rsb::args::Args) -> i32 {
    eprintln!("Backup not implemented in MVP");
    1
}

fn main() {
    // RSB canonical lifecycle pattern
    let args = bootstrap!();           // RSB initialization instead of manual args!()
    
    // Pre-dispatch for lifecycle commands (install/uninstall/backup)
    if pre_dispatch!(&args, {         // Use Args type, not Vec<String>
        "install" => do_install,       // RSB naming convention
        "uninstall" => do_uninstall,
        "backup" => do_backup
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
        "admin" => do_admin
    });
    // No manual exit - RSB dispatch! handles it
}
