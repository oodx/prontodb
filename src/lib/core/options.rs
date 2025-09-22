// // Handle global flags by parsing them and executing commands with context
// fn handle_global_flags_and_execute(args: Vec<String>) -> Option<i32> {
//     let mut cursor_name: Option<String> = None;
//     let mut user = "default".to_string();
//     let mut database = "main".to_string();
//     let mut meta_context: Option<String> = None;  // Track --meta flag
//     let mut command_args = Vec::new();
//     let mut explicit_cursor_flag = false;  // Track if --cursor was used
//     let mut explicit_database_flag = false;  // Track if --database was used
//     let mut i = 1; // Skip program name

//     // Parse global flags and remaining args
//     while i < args.len() {
//         match args[i].as_str() {
//             "--cursor" if i + 1 < args.len() => {
//                 cursor_name = Some(args[i + 1].clone());
//                 explicit_cursor_flag = true;
//                 i += 2;
//             }
//             "--user" if i + 1 < args.len() => {
//                 let user_value = args[i + 1].clone();
//                 if let Err(e) = validation::validate_username(&user_value) {
//                     eprintln!("Error: {}", e);
//                     return Some(1);
//                 }
//                 user = user_value;
//                 i += 2;
//             }
//             "--database" if i + 1 < args.len() => {
//                 database = args[i + 1].clone();
//                 explicit_database_flag = true;
//                 i += 2;
//             }
//             "--meta" if i + 1 < args.len() => {
//                 meta_context = Some(args[i + 1].clone());
//                 i += 2;
//             }
//             _ => {
//                 command_args.extend_from_slice(&args[i..]);
//                 break;
//             }
//         }
//     }

//     if command_args.is_empty() {
//         eprintln!("Error: No command specified after global flags");
//         return Some(1);
//     }

//     let command = &command_args[0];
//     let remaining_args: Vec<String> = command_args[1..].to_vec();

//     // Update cursor cache if --cursor flag was used
//     if explicit_cursor_flag {
//         if let Some(ref cursor_db) = cursor_name {
//             use prontodb::cursor_cache::CursorCache;
//             let cache = CursorCache::new();
//             let cache_user = if user == "default" { None } else { Some(user.as_str()) };

//             if let Err(e) = cache.set_cursor(cursor_db, cache_user) {
//                 eprintln!("Warning: Failed to update cursor cache: {}", e);
//                 // Continue execution - don't fail the command due to cache update failure
//             }
//         }
//     }

//     // Auto-selection logic: Check cursor cache if no explicit database flag was provided
//     if !explicit_database_flag {
//         use prontodb::cursor_cache::CursorCache;
//         let cache = CursorCache::new();

//         // Determine which user to check for cursor cache
//         let cache_user = if user == "default" { None } else { Some(user.as_str()) };

//         if let Some(cached_database) = cache.get_cursor(cache_user) {
//             database = cached_database;
//         }
//     }

//     // Execute command with global context
//     match command.as_str() {
//         "set" => Some(execute_with_context("set", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "get" => Some(execute_with_context("get", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "del" => Some(execute_with_context("del", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "keys" => Some(execute_with_context("keys", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "scan" => Some(execute_with_context("scan", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "ls" => Some(execute_with_context("ls", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "projects" => Some(execute_with_context("projects", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "namespaces" => Some(execute_with_context("namespaces", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "nss" => Some(execute_with_context("nss", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "create-cache" => Some(execute_with_context("create-cache", remaining_args, cursor_name.as_deref(), &user, &database, meta_context.as_deref())),
//         "cursor" => {
//             // For cursor command, we need to pass --user flag to the command as it handles it internally
//             let mut cursor_args = remaining_args;
//             cursor_args.push("--user".to_string());
//             cursor_args.push(user.clone());
//             let rsb_args = rsb::args::Args::new(&cursor_args);
//             Some(prontodb::do_cursor(rsb_args))
//         }
//         "backup" => {
//             // Convert command args back to RSB format for backup command
//             let mut backup_args = remaining_args;
//             // Add the database flag to the backup command args
//             backup_args.push("--database".to_string());
//             backup_args.push(database.clone());
//             let rsb_args = rsb::args::Args::new(&backup_args);
//             Some(commands::handle_backup_command(rsb_args))
//         }
//         "noop" => {
//             let mut noop_args = remaining_args;
//             // Add --user flag if specified
//             if user != "default" {
//                 noop_args.push("--user".to_string());
//                 noop_args.push(user.clone());
//             }
//             // Add --cursor flag if specified
//             if let Some(ref cursor) = cursor_name {
//                 noop_args.push("--cursor".to_string());
//                 noop_args.push(cursor.clone());
//             }
//             let rsb_args = rsb::args::Args::new(&noop_args);
//             Some(prontodb::do_noop(rsb_args))
//         }
//         "help" => {
//             let empty_args = Vec::new();
//             prontodb::do_help(rsb::args::Args::new(&empty_args));
//             Some(0)
//         }
//         _ => {
//             eprintln!("Error: Unknown command '{}'", command);
//             Some(1)
//         }
//     }
// }
