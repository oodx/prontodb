




// // Execute command with cursor, user, and database context
// fn execute_with_context(command: &str, args: Vec<String>, cursor_name: Option<&str>, user: &str, database: &str, meta_context: Option<&str>) -> i32 {
//     use prontodb::api::{*, SetValueConfig};
//     use prontodb::addressing::parse_address;
    
//     match command {
//         "set" => {
//             if args.len() < 2 {
//                 eprintln!("set: Missing arguments");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] set <address> <value>");
//                 return 1;
//             }
            
//             let address_str = &args[0];
//             let value = &args[1];
            
//             match parse_address(Some(address_str), None, None, None, ".") {
//                 Ok(_address) => {
//                     let config = SetValueConfig {
//                         project: None,
//                         namespace: None,
//                         key_or_path: address_str,
//                         value,
//                         ns_delim: ".",
//                         ttl_flag: None,
//                         cursor_name,
//                         user,
//                         database,
//                         meta_context_override: meta_context,
//                     };
//                     match set_value_with_cursor(config) {
//                         Ok(()) => {
//                             println!("Set {}={}", address_str, value);
//                             0
//                         }
//                         Err(e) => {
//                             eprintln!("set: {}", e);
//                             1
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("set: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "get" => {
//             if args.is_empty() {
//                 eprintln!("get: Missing address");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] get <address>");
//                 return 1;
//             }
            
//             let address_str = &args[0];
            
//             match parse_address(Some(address_str), None, None, None, ".") {
//                 Ok(_address) => {
//                     match get_value_with_cursor_and_database(None, None, address_str, ".", cursor_name, user, database, meta_context) {
//                         Ok(Some(value)) => {
//                             println!("{}", value);
//                             0
//                         }
//                         Ok(None) => {
//                             // Key not found - exit code 2
//                             2
//                         }
//                         Err(e) => {
//                             eprintln!("get: {}", e);
//                             1
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("get: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "del" => {
//             if args.is_empty() {
//                 eprintln!("del: Missing address");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] del <address>");
//                 return 1;
//             }
            
//             let address_str = &args[0];
            
//             match parse_address(Some(address_str), None, None, None, ".") {
//                 Ok(_address) => {
//                     match delete_value_with_cursor_and_database(None, None, address_str, ".", cursor_name, user, database) {
//                         Ok(()) => {
//                             println!("Deleted {}", address_str);
//                             0
//                         }
//                         Err(e) => {
//                             eprintln!("del: {}", e);
//                             1
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("del: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "keys" => {
//             if args.is_empty() {
//                 eprintln!("keys: Missing address");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] keys <project.namespace[.prefix]>");
//                 return 1;
//             }
            
//             let address_str = &args[0];
            
//             match parse_address(Some(address_str), None, None, None, ".") {
//                 Ok(_address) => {
//                     match list_keys_flexible_with_database(None, None, address_str, ".", cursor_name, user, database) {
//                         Ok(keys) => {
//                             for k in keys {
//                                 println!("{}", k);
//                             }
//                             0
//                         }
//                         Err(e) => {
//                             eprintln!("keys: {}", e);
//                             1
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("keys: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "scan" => {
//             if args.is_empty() {
//                 eprintln!("scan: Missing address");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] scan <project.namespace[.prefix]>");
//                 return 1;
//             }
            
//             let address_str = &args[0];
            
//             match parse_address(Some(address_str), None, None, None, ".") {
//                 Ok(_address) => {
//                     match scan_pairs_flexible_with_database(None, None, address_str, ".", cursor_name, user, database) {
//                         Ok(pairs) => {
//                             for (k, v) in pairs {
//                                 println!("{}={}", k, v);
//                             }
//                             0
//                         }
//                         Err(e) => {
//                             eprintln!("scan: {}", e);
//                             1
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     eprintln!("scan: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "projects" => {
//             match projects_with_cursor(cursor_name, user) {
//                 Ok(projects) => {
//                     for project in projects {
//                         println!("{}", project);
//                     }
//                     0
//                 }
//                 Err(e) => {
//                     eprintln!("projects: {}", e);
//                     1
//                 }
//             }
//         }
        
//         "namespaces" => {
//             if args.is_empty() {
//                 eprintln!("namespaces: Missing project argument");
//                 eprintln!("Usage: prontodb [--cursor <name>] [--user <user>] namespaces <project>");
//                 return 1;
//             }
            
//             let project = &args[0];
//             match prontodb::api::namespaces_with_cursor(project, cursor_name, user) {
//                 Ok(namespaces) => {
//                     for namespace in namespaces {
//                         println!("{}", namespace);
//                     }
//                     0
//                 }
//                 Err(e) => {
//                     eprintln!("namespaces: {}", e);
//                     1
//                 }
//             }
//         }
        
//         _ => {
//             eprintln!("Command '{}' with global flags not yet implemented", command);
//             1
//         }
//     }
// }
