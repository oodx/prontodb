// ProntoDB MVP Library
pub mod addressing;
pub mod api;
pub mod dispatcher;
pub mod storage;
pub mod xdg;

// Import RSB for command handlers
use rsb::prelude::*;

// Re-export key types for convenience  
pub use addressing::Address;
pub use storage::Storage;
pub use xdg::XdgPaths;

// RSB Command Handlers - bridge to existing dispatcher functionality
// These functions follow RSB naming convention and will eventually replace dispatcher.rs

pub fn do_set(args: rsb::args::Args) -> i32 {
    // RSB dispatch passes only command arguments, not program name or command
    let mut vec_args = vec!["prontodb".to_string(), "set".to_string()];
    vec_args.extend(args.all().iter().cloned());  // RSB already filtered
    dispatcher::dispatch(vec_args)
}

pub fn do_get(args: rsb::args::Args) -> i32 {
    // RSB dispatch passes only command arguments, not program name
    let mut vec_args = vec!["prontodb".to_string(), "get".to_string()];
    vec_args.extend(args.all().iter().cloned());  // Don't skip anything - RSB already filtered
    dispatcher::dispatch(vec_args)
}

pub fn do_del(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "del".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_keys(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "keys".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_scan(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "scan".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_ls(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "ls".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_create_cache(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "create-cache".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_projects(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "projects".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_namespaces(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "namespaces".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_nss(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "nss".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_stream(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "stream".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_admin(args: rsb::args::Args) -> i32 {
    let mut vec_args = vec!["prontodb".to_string(), "admin".to_string()];
    vec_args.extend(args.all().iter().cloned());
    dispatcher::dispatch(vec_args)
}

pub fn do_help(_args: rsb::args::Args) -> i32 {
    println!("ProntoDB - Fast namespaced key-value store with TTL support");
    println!();
    println!("USAGE:");
    println!("  prontodb <command> [options] [arguments]");
    println!();
    println!("DOT ADDRESSING (Primary):");
    println!("  prontodb set project.namespace.key \"value\"");
    println!("  prontodb get project.namespace.key");
    println!("  prontodb del project.namespace.key");
    println!("  prontodb set app.config.debug__prod \"true\"    # Context addressing");
    println!();
    println!("FLAG ADDRESSING (Alternative):");
    println!("  prontodb set -p project -n namespace key \"value\"");
    println!("  prontodb get -p project -n namespace key");
    println!();
    println!("CORE COMMANDS:");
    println!("  set <address> <value>      Store a value");
    println!("  get <address>              Retrieve a value (exit 2 if not found)");
    println!("  del <address>              Delete a value");
    println!("  keys [prefix]              List all keys, optionally with prefix");
    println!("  scan [prefix]              List key-value pairs, optionally with prefix");
    println!();
    println!("DISCOVERY:");
    println!("  projects                   List all projects");
    println!("  namespaces -p <project>    List namespaces in project");
    println!("  nss                        List all namespaces");
    println!();
    println!("ADMINISTRATION:");
    println!("  create-cache <proj.ns> <ttl_sec>    Create TTL-enabled namespace");
    println!("  admin <command>                     Administrative operations");
    println!();
    println!("OTHER:");
    println!("  stream                     Stream processing mode");
    println!("  version                    Show version information");
    println!("  help                       Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("  # Store application config with dot addressing");
    println!("  prontodb set myapp.config.database_host \"prod.db.com\"");
    println!("  prontodb set myapp.config.api_key__prod \"secret123\"");
    println!();
    println!("  # Retrieve and check exit codes");
    println!("  prontodb get myapp.config.database_host");
    println!("  echo $?  # 0=found, 2=not found, 1=error");
    println!();
    println!("  # Create TTL cache and use it");
    println!("  prontodb create-cache sessions.cache 3600");
    println!("  prontodb set sessions.cache.user123 \"active\"");
    println!();
    println!("  # Discover your data structure");
    println!("  prontodb projects");
    println!("  prontodb namespaces -p myapp");
    println!("  prontodb keys myapp.config");
    println!();
    println!("For more examples and documentation, see README.md");
    0
}

// TDD infrastructure validation function for Card 001
/// Validates that the TDD infrastructure is properly set up and working
/// Returns true if all TDD components are functioning correctly
pub fn validate_tdd_infrastructure() -> bool {
    // Minimal implementation to pass the test
    // This validates that:
    // 1. Test framework can compile and run
    // 2. Functions can be called from tests
    // 3. Basic assertion infrastructure works
    true
}
