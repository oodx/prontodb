
// use crate::xdg;


// fn do_uninstall(args: rsb::args::Args) -> i32 {
//     use std::path::PathBuf;
//     use std::fs;
//     use std::io::{self, Write};
    
//     // Parse uninstall options
//     let mut target_dir = None;
//     let mut purge = false;
//     let mut nuke = false;
//     let mut force = false;
//     let mut quiet = false;
    
//     let arg_list = args.all();
//     let mut i = 0;
//     while i < arg_list.len() {
//         match arg_list[i].as_str() {
//             "--target" | "-t" if i + 1 < arg_list.len() => {
//                 target_dir = Some(PathBuf::from(&arg_list[i + 1]));
//                 i += 2;
//             }
//             "--purge" | "-p" => {
//                 purge = true;
//                 i += 1;
//             }
//             "--nuke" => {
//                 nuke = true;
//                 purge = true; // --nuke implies --purge
//                 i += 1;
//             }
//             "--force" | "-f" => {
//                 force = true;
//                 i += 1;
//             }
//             "--quiet" | "-q" => {
//                 quiet = true;
//                 i += 1;
//             }
//             "--help" | "-h" => {
//                 println!("prontodb uninstall - Remove ProntoDB binary and optionally data");
//                 println!();
//                 println!("USAGE:");
//                 println!("  prontodb uninstall [OPTIONS]");
//                 println!();
//                 println!("OPTIONS:");
//                 println!("  -t, --target <DIR>    Uninstall from specific directory (default: ~/.local/bin)");
//                 println!("  -p, --purge           Remove all data, config, and cursors (requires confirmation)");
//                 println!("      --nuke            Remove everything with automatic safety backup (requires confirmation)");
//                 println!("  -f, --force           Skip confirmation prompts");
//                 println!("  -q, --quiet           Suppress output messages");
//                 println!("  -h, --help            Show this help message");
//                 println!();
//                 println!("EXAMPLES:");
//                 println!("  prontodb uninstall                # Remove binary only");
//                 println!("  prontodb uninstall --purge        # Remove binary and all data (with confirmation)");
//                 println!("  prontodb uninstall --nuke         # Nuclear option: backup then remove everything");
//                 println!("  prontodb uninstall --nuke -f      # Nuclear without confirmation (backup still created)");
//                 println!();
//                 println!("WARNING: --purge will permanently delete all data! --nuke creates safety backup first.");
//                 return 0;
//             }
//             _ => {
//                 eprintln!("uninstall: Unknown option '{}'", arg_list[i]);
//                 eprintln!("Use 'prontodb uninstall --help' for usage information");
//                 return 1;
//             }
//         }
//     }
    
//     // Determine uninstall target
//     let install_dir = target_dir.unwrap_or_else(|| {
//         xdg::XdgPaths::new().home.join(".local").join("bin")
//     });
    
//     let target_exe = install_dir.join("prontodb");
    
//     // Check if binary exists
//     if !target_exe.exists() {
//         if !quiet {
//             println!("ProntoDB binary not found at {}", target_exe.display());
//             println!("Nothing to uninstall.");
//         }
//         return 0;
//     }
    
//     if !quiet {
//         println!("Uninstalling ProntoDB from: {}", target_exe.display());
//     }
    
//     // Handle purge confirmation
//     if purge && !force {
//         println!();
//         println!("WARNING: This will permanently delete ALL ProntoDB data:");
//         let paths = xdg::XdgPaths::new();
//         println!("  - Database: {}", paths.get_db_path().display());
//         println!("  - Config directory: {}", paths.config_dir.display());
//         println!("  - Data directory: {}", paths.data_dir.display());
//         println!("  - Cursor directory: {}", paths.cursor_dir.display());
//         println!();
//         print!("Are you sure you want to proceed? [y/N]: ");
//         io::stdout().flush().unwrap();
        
//         let mut input = String::new();
//         if let Err(e) = io::stdin().read_line(&mut input) {
//             eprintln!("uninstall: Failed to read input: {}", e);
//             return 1;
//         }
        
//         let response = input.trim().to_lowercase();
//         if response != "y" && response != "yes" {
//             if !quiet {
//                 println!("Uninstall cancelled.");
//             }
//             return 0;
//         }
//     }
    
//     // Remove binary
//     if let Err(e) = fs::remove_file(&target_exe) {
//         eprintln!("uninstall: Failed to remove binary {}: {}", target_exe.display(), e);
//         return 1;
//     }
    
//     if !quiet {
//         println!("Binary removed: {}", target_exe.display());
//     }
    
//     // Handle --nuke safety backup before purge
//     if nuke && purge {
//         if !quiet {
//             println!("Creating safety backup before nuclear uninstall...");
//         }
        
//         // Create automatic safety backup with nuke timestamp
//         let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
//         let backup_args = vec![
//             "backup".to_string(),
//             "--output".to_string(),
//             format!("~/repos/zindex/cache/backup/safety_backup_nuke_{}.tar.gz", timestamp)
//         ];
        
//         let backup_args_obj = rsb::args::Args::new(&backup_args);
//         let backup_result = commands::handle_backup_command(backup_args_obj);
        
//         if backup_result != 0 {
//             eprintln!("uninstall: Failed to create safety backup, aborting nuclear uninstall");
//             eprintln!("Use --purge instead of --nuke if you want to proceed without backup");
//             return backup_result;
//         }
        
//         if !quiet {
//             println!("Safety backup created successfully!");
//         }
//     }
    
//     // Handle purge if requested
//     if purge {
//         let paths = xdg::XdgPaths::new();
        
//         // Remove data directory (contains database and cursors)
//         if paths.data_dir.exists() {
//             if let Err(e) = fs::remove_dir_all(&paths.data_dir) {
//                 eprintln!("uninstall: Failed to remove data directory {}: {}", paths.data_dir.display(), e);
//                 return 1;
//             }
//             if !quiet {
//                 println!("Data directory removed: {}", paths.data_dir.display());
//             }
//         }
        
//         // Remove config directory
//         if paths.config_dir.exists() {
//             if let Err(e) = fs::remove_dir_all(&paths.config_dir) {
//                 eprintln!("uninstall: Failed to remove config directory {}: {}", paths.config_dir.display(), e);
//                 return 1;
//             }
//             if !quiet {
//                 println!("Config directory removed: {}", paths.config_dir.display());
//             }
//         }
        
//         // Remove cache directory
//         if paths.cache_dir.exists() {
//             if let Err(e) = fs::remove_dir_all(&paths.cache_dir) {
//                 eprintln!("uninstall: Failed to remove cache directory {}: {}", paths.cache_dir.display(), e);
//                 return 1;
//             }
//             if !quiet {
//                 println!("Cache directory removed: {}", paths.cache_dir.display());
//             }
//         }
        
//         // Try to remove parent odx directories if empty
//         let odx_data_parent = paths.data_dir.parent().unwrap_or(&paths.data_dir);
//         let odx_config_parent = paths.config_dir.parent().unwrap_or(&paths.config_dir);
        
//         // Silently attempt to remove parent directories if empty
//         let _ = fs::remove_dir(odx_data_parent);
//         let _ = fs::remove_dir(odx_config_parent);
        
//         if !quiet {
//             println!("All ProntoDB data has been permanently removed.");
//         }
//     }
    
//     if !quiet {
//         if purge {
//             println!("ProntoDB has been completely uninstalled.");
//         } else {
//             println!("ProntoDB binary has been uninstalled.");
//             println!("To remove all data, run: prontodb uninstall --purge");
//         }
//     }
    
//     0
// }
