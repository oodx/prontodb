// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with RSB lifecycle

mod dispatcher;
mod addressing;
mod storage;
mod xdg;
mod api;
mod backup;
mod cursor;

// Use RSB prelude for macros (bootstrap!/pre_dispatch!/dispatch!)
use rsb::prelude::*;

// Import RSB command handlers
use prontodb::{do_set, do_get, do_del, do_keys, do_scan, do_ls, do_create_cache, do_projects, do_namespaces, do_nss, do_stream, do_admin, do_help, do_cursor};

// RSB lifecycle command handlers with proper naming convention
fn do_install(args: rsb::args::Args) -> i32 {
    use std::path::PathBuf;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    
    // Parse install options
    let mut target_dir = None;
    let mut force = false;
    let mut quiet = false;
    
    let arg_list = args.all();
    let mut i = 0;
    while i < arg_list.len() {
        match arg_list[i].as_str() {
            "--target" | "-t" if i + 1 < arg_list.len() => {
                target_dir = Some(PathBuf::from(&arg_list[i + 1]));
                i += 2;
            }
            "--force" | "-f" => {
                force = true;
                i += 1;
            }
            "--quiet" | "-q" => {
                quiet = true;
                i += 1;
            }
            "--help" | "-h" => {
                println!("prontodb install - Install ProntoDB binary and setup environment");
                println!();
                println!("USAGE:");
                println!("  prontodb install [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  -t, --target <DIR>    Install to specific directory (default: ~/.local/bin)");
                println!("  -f, --force           Overwrite existing installation");
                println!("  -q, --quiet           Suppress output messages");
                println!("  -h, --help            Show this help message");
                return 0;
            }
            _ => {
                eprintln!("install: Unknown option '{}'", arg_list[i]);
                eprintln!("Use 'prontodb install --help' for usage information");
                return 1;
            }
        }
    }
    
    // Determine installation target
    let install_dir = target_dir.unwrap_or_else(|| {
        xdg::XdgPaths::new().home.join(".local").join("bin")
    });
    
    if !quiet {
        println!("Installing ProntoDB to: {}", install_dir.display());
    }
    
    // Create installation directory
    if let Err(e) = fs::create_dir_all(&install_dir) {
        eprintln!("install: Failed to create directory {}: {}", install_dir.display(), e);
        return 1;
    }
    
    // Get current executable path
    let current_exe = match std::env::current_exe() {
        Ok(path) => path,
        Err(e) => {
            eprintln!("install: Failed to get current executable path: {}", e);
            return 1;
        }
    };
    
    let target_exe = install_dir.join("prontodb");
    
    // Check if already installed
    if target_exe.exists() && !force {
        eprintln!("install: ProntoDB already installed at {}", target_exe.display());
        eprintln!("Use --force to overwrite existing installation");
        return 1;
    }
    
    // Copy binary
    if let Err(e) = fs::copy(&current_exe, &target_exe) {
        eprintln!("install: Failed to copy binary: {}", e);
        return 1;
    }
    
    // Make executable
    if let Err(e) = fs::set_permissions(&target_exe, fs::Permissions::from_mode(0o755)) {
        eprintln!("install: Failed to set executable permissions: {}", e);
        return 1;
    }
    
    // Setup XDG directory structure
    let paths = xdg::XdgPaths::new();
    if let Err(e) = paths.ensure_dirs() {
        eprintln!("install: Failed to create XDG directories: {}", e);
        return 1;
    }
    
    // Initialize default cursor
    let cursor_manager = cursor::CursorManager::new();
    if let Err(e) = cursor_manager.ensure_default_cursor("default") {
        eprintln!("install: Failed to initialize default cursor: {}", e);
        return 1;
    }
    
    if !quiet {
        println!("Installation completed successfully!");
        println!("Binary installed: {}", target_exe.display());
        println!("Data directory: {}", paths.data_dir.display());
        println!("Config directory: {}", paths.config_dir.display());
        println!("Cursor directory: {}", paths.cursor_dir.display());
        println!();
        println!("Add {} to your PATH to use 'prontodb' command globally", install_dir.display());
    }
    
    0
}

fn do_uninstall(args: rsb::args::Args) -> i32 {
    use std::path::PathBuf;
    use std::fs;
    use std::io::{self, Write};
    
    // Parse uninstall options
    let mut target_dir = None;
    let mut purge = false;
    let mut force = false;
    let mut quiet = false;
    
    let arg_list = args.all();
    let mut i = 0;
    while i < arg_list.len() {
        match arg_list[i].as_str() {
            "--target" | "-t" if i + 1 < arg_list.len() => {
                target_dir = Some(PathBuf::from(&arg_list[i + 1]));
                i += 2;
            }
            "--purge" | "-p" => {
                purge = true;
                i += 1;
            }
            "--force" | "-f" => {
                force = true;
                i += 1;
            }
            "--quiet" | "-q" => {
                quiet = true;
                i += 1;
            }
            "--help" | "-h" => {
                println!("prontodb uninstall - Remove ProntoDB binary and optionally data");
                println!();
                println!("USAGE:");
                println!("  prontodb uninstall [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  -t, --target <DIR>    Uninstall from specific directory (default: ~/.local/bin)");
                println!("  -p, --purge           Remove all data, config, and cursors (requires confirmation)");
                println!("  -f, --force           Skip confirmation prompts");
                println!("  -q, --quiet           Suppress output messages");
                println!("  -h, --help            Show this help message");
                println!();
                println!("EXAMPLES:");
                println!("  prontodb uninstall                # Remove binary only");
                println!("  prontodb uninstall --purge        # Remove binary and all data (with confirmation)");
                println!("  prontodb uninstall --purge -f     # Remove everything without confirmation");
                println!();
                println!("WARNING: --purge will permanently delete all ProntoDB data and cannot be undone!");
                return 0;
            }
            _ => {
                eprintln!("uninstall: Unknown option '{}'", arg_list[i]);
                eprintln!("Use 'prontodb uninstall --help' for usage information");
                return 1;
            }
        }
    }
    
    // Determine uninstall target
    let install_dir = target_dir.unwrap_or_else(|| {
        xdg::XdgPaths::new().home.join(".local").join("bin")
    });
    
    let target_exe = install_dir.join("prontodb");
    
    // Check if binary exists
    if !target_exe.exists() {
        if !quiet {
            println!("ProntoDB binary not found at {}", target_exe.display());
            println!("Nothing to uninstall.");
        }
        return 0;
    }
    
    if !quiet {
        println!("Uninstalling ProntoDB from: {}", target_exe.display());
    }
    
    // Handle purge confirmation
    if purge && !force {
        println!();
        println!("WARNING: This will permanently delete ALL ProntoDB data:");
        let paths = xdg::XdgPaths::new();
        println!("  - Database: {}", paths.get_db_path().display());
        println!("  - Config directory: {}", paths.config_dir.display());
        println!("  - Data directory: {}", paths.data_dir.display());
        println!("  - Cursor directory: {}", paths.cursor_dir.display());
        println!();
        print!("Are you sure you want to proceed? [y/N]: ");
        io::stdout().flush().unwrap();
        
        let mut input = String::new();
        if let Err(e) = io::stdin().read_line(&mut input) {
            eprintln!("uninstall: Failed to read input: {}", e);
            return 1;
        }
        
        let response = input.trim().to_lowercase();
        if response != "y" && response != "yes" {
            if !quiet {
                println!("Uninstall cancelled.");
            }
            return 0;
        }
    }
    
    // Remove binary
    if let Err(e) = fs::remove_file(&target_exe) {
        eprintln!("uninstall: Failed to remove binary {}: {}", target_exe.display(), e);
        return 1;
    }
    
    if !quiet {
        println!("Binary removed: {}", target_exe.display());
    }
    
    // Handle purge if requested
    if purge {
        let paths = xdg::XdgPaths::new();
        
        // Remove data directory (contains database and cursors)
        if paths.data_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&paths.data_dir) {
                eprintln!("uninstall: Failed to remove data directory {}: {}", paths.data_dir.display(), e);
                return 1;
            }
            if !quiet {
                println!("Data directory removed: {}", paths.data_dir.display());
            }
        }
        
        // Remove config directory
        if paths.config_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&paths.config_dir) {
                eprintln!("uninstall: Failed to remove config directory {}: {}", paths.config_dir.display(), e);
                return 1;
            }
            if !quiet {
                println!("Config directory removed: {}", paths.config_dir.display());
            }
        }
        
        // Remove cache directory
        if paths.cache_dir.exists() {
            if let Err(e) = fs::remove_dir_all(&paths.cache_dir) {
                eprintln!("uninstall: Failed to remove cache directory {}: {}", paths.cache_dir.display(), e);
                return 1;
            }
            if !quiet {
                println!("Cache directory removed: {}", paths.cache_dir.display());
            }
        }
        
        // Try to remove parent odx directories if empty
        let odx_data_parent = paths.data_dir.parent().unwrap_or(&paths.data_dir);
        let odx_config_parent = paths.config_dir.parent().unwrap_or(&paths.config_dir);
        
        // Silently attempt to remove parent directories if empty
        let _ = fs::remove_dir(odx_data_parent);
        let _ = fs::remove_dir(odx_config_parent);
        
        if !quiet {
            println!("All ProntoDB data has been permanently removed.");
        }
    }
    
    if !quiet {
        if purge {
            println!("ProntoDB has been completely uninstalled.");
        } else {
            println!("ProntoDB binary has been uninstalled.");
            println!("To remove all data, run: prontodb uninstall --purge");
        }
    }
    
    0
}

fn do_backup(args: rsb::args::Args) -> i32 {
    use std::path::PathBuf;
    
    // Parse backup options
    let mut output_dir = None;
    let mut list_backups = false;
    let mut restore_file = None;
    let mut quiet = false;
    
    let arg_list = args.all();
    let mut i = 0;
    while i < arg_list.len() {
        match arg_list[i].as_str() {
            "--output" | "-o" if i + 1 < arg_list.len() => {
                output_dir = Some(PathBuf::from(&arg_list[i + 1]));
                i += 2;
            }
            "--list" | "-l" => {
                list_backups = true;
                i += 1;
            }
            "--restore" | "-r" if i + 1 < arg_list.len() => {
                restore_file = Some(PathBuf::from(&arg_list[i + 1]));
                i += 2;
            }
            "--quiet" | "-q" => {
                quiet = true;
                i += 1;
            }
            "--help" | "-h" => {
                println!("prontodb backup - Backup and restore ProntoDB data");
                println!();
                println!("USAGE:");
                println!("  prontodb backup [OPTIONS]");
                println!();
                println!("OPTIONS:");
                println!("  -o, --output <DIR>     Output directory for backup (default: current directory)");
                println!("  -l, --list             List existing backup files in current directory");
                println!("  -r, --restore <FILE>   Restore from backup file");
                println!("  -q, --quiet            Suppress output messages");
                println!("  -h, --help             Show this help message");
                println!();
                println!("EXAMPLES:");
                println!("  prontodb backup                           # Create backup in current directory");
                println!("  prontodb backup --output ~/backups        # Create backup in specific directory");
                println!("  prontodb backup --list                    # List existing backups");
                println!("  prontodb backup --restore backup.tar.gz   # Restore from backup");
                return 0;
            }
            _ => {
                eprintln!("backup: Unknown option '{}'", arg_list[i]);
                eprintln!("Use 'prontodb backup --help' for usage information");
                return 1;
            }
        }
    }
    
    let backup_manager = backup::BackupManager::new();
    
    // Handle list operation
    if list_backups {
        let search_dir = output_dir.as_deref();
        match backup_manager.list_backups(search_dir) {
            Ok(backups) => {
                if backups.is_empty() {
                    if !quiet {
                        println!("No backup files found");
                    }
                } else {
                    if !quiet {
                        println!("Found {} backup file(s):", backups.len());
                    }
                    for backup in backups {
                        let metadata = std::fs::metadata(&backup)
                            .map(|m| format!("{} bytes", m.len()))
                            .unwrap_or_else(|_| "unknown size".to_string());
                        
                        let modified = std::fs::metadata(&backup)
                            .and_then(|m| m.modified())
                            .map(|t| t.duration_since(std::time::UNIX_EPOCH)
                                     .map(|d| d.as_secs())
                                     .unwrap_or(0))
                            .map(|s| chrono::DateTime::from_timestamp(s as i64, 0)
                                     .map(|dt| dt.format("%Y-%m-%d %H:%M:%S UTC").to_string())
                                     .unwrap_or_else(|| "unknown".to_string()))
                            .unwrap_or_else(|_| "unknown".to_string());
                            
                        println!("  {} ({}, {})", backup.display(), metadata, modified);
                    }
                }
                return 0;
            }
            Err(e) => {
                eprintln!("backup: Failed to list backups: {}", e);
                return 1;
            }
        }
    }
    
    // Handle restore operation
    if let Some(restore_path) = restore_file {
        if !quiet {
            println!("Restoring from backup: {}", restore_path.display());
        }
        
        match backup_manager.restore_backup(&restore_path) {
            Ok(()) => {
                if !quiet {
                    println!("Backup restored successfully!");
                }
                return 0;
            }
            Err(e) => {
                eprintln!("backup: Failed to restore backup: {}", e);
                return 1;
            }
        }
    }
    
    // Default: create backup
    if !quiet {
        println!("Creating backup...");
    }
    
    match backup_manager.create_backup(output_dir.as_deref()) {
        Ok(result) => {
            if !quiet {
                println!("Backup created successfully!");
                println!("File: {}", result.file_path.display());
                println!("Size: {} bytes", result.size_bytes);
                println!("Timestamp: {}", result.timestamp);
                if !result.contents.is_empty() {
                    println!("Contents:");
                    for item in result.contents {
                        println!("  {}", item);
                    }
                }
            }
            0
        }
        Err(e) => {
            eprintln!("backup: Failed to create backup: {}", e);
            1
        }
    }
}

fn main() {
    // Check for global flags before RSB processes them as unknown commands
    let raw_args: Vec<String> = std::env::args().collect();
    
    // If we find global flags, intercept and handle them
    if raw_args.iter().any(|arg| arg == "--cursor" || arg == "--user") {
        // Handle global flag parsing and command execution directly
        if let Some(exit_code) = handle_global_flags_and_execute(raw_args) {
            std::process::exit(exit_code);
        }
    }
    
    // RSB canonical lifecycle pattern for normal commands (without global flags)
    let args = bootstrap!();           // RSB initialization
    
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
        "admin" => do_admin,
        "cursor" => do_cursor,
        "help" => do_help
    });
    // No manual exit - RSB dispatch! handles it
}

// Handle global flags by parsing them and executing commands with context
fn handle_global_flags_and_execute(args: Vec<String>) -> Option<i32> {
    let mut cursor_name: Option<String> = None;
    let mut user = "default".to_string();
    let mut command_args = Vec::new();
    let mut i = 1; // Skip program name
    
    // Parse global flags and remaining args
    while i < args.len() {
        match args[i].as_str() {
            "--cursor" if i + 1 < args.len() => {
                cursor_name = Some(args[i + 1].clone());
                i += 2;
            }
            "--user" if i + 1 < args.len() => {
                user = args[i + 1].clone();
                i += 2;
            }
            _ => {
                command_args.extend_from_slice(&args[i..]);
                break;
            }
        }
    }
    
    if command_args.is_empty() {
        eprintln!("Error: No command specified after global flags");
        return Some(1);
    }
    
    let command = &command_args[0];
    let remaining_args: Vec<String> = command_args[1..].to_vec();
    
    // Execute command with global context
    match command.as_str() {
        "set" => Some(execute_with_context("set", remaining_args, cursor_name.as_deref(), &user)),
        "get" => Some(execute_with_context("get", remaining_args, cursor_name.as_deref(), &user)),
        "del" => Some(execute_with_context("del", remaining_args, cursor_name.as_deref(), &user)),
        "keys" => Some(execute_with_context("keys", remaining_args, cursor_name.as_deref(), &user)),
        "scan" => Some(execute_with_context("scan", remaining_args, cursor_name.as_deref(), &user)),
        "ls" => Some(execute_with_context("ls", remaining_args, cursor_name.as_deref(), &user)),
        "projects" => Some(execute_with_context("projects", remaining_args, cursor_name.as_deref(), &user)),
        "namespaces" => Some(execute_with_context("namespaces", remaining_args, cursor_name.as_deref(), &user)),
        "nss" => Some(execute_with_context("nss", remaining_args, cursor_name.as_deref(), &user)),
        "create-cache" => Some(execute_with_context("create-cache", remaining_args, cursor_name.as_deref(), &user)),
        "cursor" => {
            // For cursor command, we need to pass --user flag to the command as it handles it internally
            let mut cursor_args = remaining_args;
            cursor_args.push("--user".to_string());
            cursor_args.push(user.clone());
            let rsb_args = rsb::args::Args::new(&cursor_args);
            Some(prontodb::do_cursor(rsb_args))
        }
        "help" => {
            let empty_args = Vec::new();
            prontodb::do_help(rsb::args::Args::new(&empty_args));
            Some(0)
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            Some(1)
        }
    }
}

// Execute command with cursor and user context
fn execute_with_context(command: &str, args: Vec<String>, cursor_name: Option<&str>, user: &str) -> i32 {
    use prontodb::api::*;
    use prontodb::addressing::parse_address;
    
    match command {
        "set" => {
            if args.len() < 2 {
                eprintln!("set: Missing arguments");
                eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] set <address> <value>");
                return 1;
            }
            
            let address_str = &args[0];
            let value = &args[1];
            
            match parse_address(Some(address_str), None, None, None, ".") {
                Ok(_address) => {
                    match set_value_with_cursor(None, None, address_str, value, ".", None, cursor_name, user) {
                        Ok(()) => {
                            println!("Set {}={}", address_str, value);
                            0
                        }
                        Err(e) => {
                            eprintln!("set: {}", e);
                            1
                        }
                    }
                }
                Err(e) => {
                    eprintln!("set: {}", e);
                    1
                }
            }
        }
        
        "get" => {
            if args.is_empty() {
                eprintln!("get: Missing address");
                eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] get <address>");
                return 1;
            }
            
            let address_str = &args[0];
            
            match parse_address(Some(address_str), None, None, None, ".") {
                Ok(_address) => {
                    match get_value_with_cursor(None, None, address_str, ".", cursor_name, user) {
                        Ok(Some(value)) => {
                            println!("{}", value);
                            0
                        }
                        Ok(None) => {
                            // Key not found - exit code 2
                            2
                        }
                        Err(e) => {
                            eprintln!("get: {}", e);
                            1
                        }
                    }
                }
                Err(e) => {
                    eprintln!("get: {}", e);
                    1
                }
            }
        }
        
        "del" => {
            if args.is_empty() {
                eprintln!("del: Missing address");
                eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] del <address>");
                return 1;
            }
            
            let address_str = &args[0];
            
            match parse_address(Some(address_str), None, None, None, ".") {
                Ok(_address) => {
                    match delete_value_with_cursor(None, None, address_str, ".", cursor_name, user) {
                        Ok(()) => {
                            println!("Deleted {}", address_str);
                            0
                        }
                        Err(e) => {
                            eprintln!("del: {}", e);
                            1
                        }
                    }
                }
                Err(e) => {
                    eprintln!("del: {}", e);
                    1
                }
            }
        }
        
        "keys" => {
            // For list_keys_with_cursor, we need project and namespace
            // This is a simplified version that lists all keys if no specific project/namespace
            eprintln!("Warning: Global keys listing not yet fully implemented with cursor context");
            eprintln!("Use: prontodb [--cursor name] keys project.namespace.prefix");
            1
        }
        
        "scan" => {
            // For scan_pairs_with_cursor, we need project and namespace  
            // This is a simplified version
            eprintln!("Warning: Global scan not yet fully implemented with cursor context");
            eprintln!("Use: prontodb [--cursor name] scan project.namespace.prefix");
            1
        }
        
        "projects" => {
            match projects_with_cursor(cursor_name, user) {
                Ok(projects) => {
                    for project in projects {
                        println!("{}", project);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("projects: {}", e);
                    1
                }
            }
        }
        
        "namespaces" => {
            if args.is_empty() {
                eprintln!("namespaces: Missing project argument");
                eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] namespaces <project>");
                return 1;
            }
            
            let project = &args[0];
            match prontodb::api::namespaces_with_cursor(project, cursor_name, user) {
                Ok(namespaces) => {
                    for namespace in namespaces {
                        println!("{}", namespace);
                    }
                    0
                }
                Err(e) => {
                    eprintln!("namespaces: {}", e);
                    1
                }
            }
        }
        
        _ => {
            eprintln!("Command '{}' with global flags not yet implemented", command);
            1
        }
    }
}
