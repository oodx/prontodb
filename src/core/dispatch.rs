// Minimal dispatcher for testing routing
use rsb::prelude::*;

pub fn pronto_dispatch(args: rsb::args::Args) -> i32 {
    info!("Dispatch called with {} args", args.all().len());
    
    if args.len() == 0 {
        info!("No command provided, showing help");
        return do_help(args);
    }

    let command = args.get_or(1, "");
    info!("Processing command: '{}'", command);
    
    // Use RSB dispatch! macro (now expects &Args)
    dispatch!(&args, {
        "set" => do_set,
        "get" => do_get, 
        "del" => do_del,
        "delete" => do_del,
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
        "nuclear-clean" => do_nuclear_clean,
        "install" => do_install,
        "uninstall" => do_uninstall,
        "backup" => do_backup,
        "restore" => do_restore,
        "version" => do_version,
        "help" => do_help
    })
}

// Command stubs - RSB dispatch expects fn(Args) -> i32
fn do_set(mut args: Args) -> i32 { 
    info!("✓ Executing: set"); 
    // Test if options were stored in global context
    if has_var("opt_verbose") {
        info!("Verbose mode enabled: {}", get_var("opt_verbose"));
    }
    0 
}
fn do_get(mut args: Args) -> i32 { info!("✓ Executing: get"); 0 }
fn do_del(mut args: Args) -> i32 { info!("✓ Executing: del"); 0 }
fn do_keys(mut args: Args) -> i32 { info!("✓ Executing: keys"); 0 }
fn do_scan(mut args: Args) -> i32 { info!("✓ Executing: scan"); 0 }
fn do_ls(mut args: Args) -> i32 { info!("✓ Executing: ls"); 0 }
fn do_create_cache(mut args: Args) -> i32 { info!("✓ Executing: create-cache"); 0 }
fn do_projects(mut args: Args) -> i32 { info!("✓ Executing: projects"); 0 }
fn do_namespaces(mut args: Args) -> i32 { info!("✓ Executing: namespaces"); 0 }
fn do_nss(mut args: Args) -> i32 { info!("✓ Executing: nss"); 0 }
fn do_stream(mut args: Args) -> i32 { info!("✓ Executing: stream"); 0 }
fn do_copy(mut args: Args) -> i32 { info!("✓ Executing: copy"); 0 }
fn do_admin(mut args: Args) -> i32 { info!("✓ Executing: admin"); 0 }
fn do_cursor(mut args: Args) -> i32 { info!("✓ Executing: cursor"); 0 }
fn do_nuclear_clean(mut args: Args) -> i32 { info!("✓ Executing: nuclear-clean"); 0 }
fn do_install(mut args: Args) -> i32 { info!("✓ Executing: install"); 0 }
fn do_uninstall(mut args: Args) -> i32 { info!("✓ Executing: uninstall"); 0 }
fn do_backup(mut args: Args) -> i32 { info!("✓ Executing: backup"); 0 }
fn do_restore(mut args: Args) -> i32 { info!("✓ Executing: restore"); 0 }
fn do_version(mut args: Args) -> i32 { info!("✓ Executing: version"); 0 }

fn do_help(mut args: Args) -> i32 {
    info!("ProntoDB - Available Commands:");
    println!("  set           - Set a value");
    println!("  get           - Get a value");  
    println!("  del/delete    - Delete a value");
    println!("  keys          - List keys");
    println!("  scan          - Scan namespace");
    println!("  ls            - List items");
    println!("  create-cache  - Create cache");
    println!("  projects      - Manage projects");
    println!("  namespaces    - Manage namespaces");
    println!("  nss           - Namespace shortcuts");
    println!("  stream        - Stream operations");
    println!("  copy          - Copy operations");
    println!("  admin         - Admin functions");
    println!("  cursor        - Cursor management");
    println!("  nuclear-clean - Clean everything");
    println!("  install       - Install ProntoDB");
    println!("  uninstall     - Uninstall ProntoDB");
    println!("  backup        - Backup data");
    println!("  restore       - Restore data");
    println!("  version       - Show version");
    println!("  help          - Show this help");
    0
}