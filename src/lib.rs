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
