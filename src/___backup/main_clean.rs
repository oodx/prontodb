// Clean main.rs using AddressResolver for proper separation of concerns

use prontodb::{AddressResolver, ResolvedAddress, MetaStorage, Storage};
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        return;
    }

    // Parse command line arguments
    let mut resolver = AddressResolver::new();
    let mut command = None;
    let mut address_str = None;
    let mut value = None;
    
    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];
        
        if arg == "--meta" && i + 1 < args.len() {
            resolver = resolver.with_meta(Some(args[i + 1].clone()));
            i += 2;
        } else if arg == "-p" && i + 1 < args.len() {
            resolver = resolver.with_project(Some(args[i + 1].clone()));
            i += 2;
        } else if arg == "-n" && i + 1 < args.len() {
            resolver = resolver.with_namespace(Some(args[i + 1].clone()));
            i += 2;
        } else if command.is_none() {
            command = Some(arg.clone());
            i += 1;
        } else if address_str.is_none() {
            address_str = Some(arg.clone());
            i += 1;
        } else if value.is_none() {
            value = Some(arg.clone());
            i += 1;
        } else {
            i += 1;
        }
    }

    let command = match command {
        Some(cmd) => cmd,
        None => {
            eprintln!("Error: No command provided");
            print_usage();
            return;
        }
    };

    let address_str = match address_str {
        Some(addr) => addr,
        None => {
            eprintln!("Error: No address provided");
            return;
        }
    };

    // Resolve the address
    let resolved_address = match resolver.resolve_address(&address_str) {
        Ok(addr) => {
            println!("Resolved address: {}", addr.to_display_key());
            addr
        }
        Err(e) => {
            eprintln!("Error resolving address: {}", e);
            return;
        }
    };

    // Execute the command
    match command.as_str() {
        "set" => {
            let value = match value {
                Some(v) => v,
                None => {
                    eprintln!("Error: No value provided for set command");
                    return;
                }
            };
            handle_set(resolved_address, &value);
        }
        "get" => {
            handle_get(resolved_address);
        }
        "del" => {
            handle_del(resolved_address);
        }
        _ => {
            eprintln!("Error: Unknown command '{}'", command);
            print_usage();
        }
    }
}

fn handle_set(address: ResolvedAddress, value: &str) {
    match address {
        ResolvedAddress::FourLayer(addr) => {
            let db_path = Path::new("/tmp/meta_storage.db");
            let storage = match MetaStorage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            // Auto-register meta if needed
            if addr.meta != "default" {
                storage.register_meta(&addr.meta, &format!("pantheon_{}", addr.meta), None).ok();
            }

            match storage.set(&addr, value, None) {
                Ok(_) => {
                    println!("✅ Set: {} = {}", addr.to_storage_key(), value);
                    if let Ok(resolved) = storage.resolve_address(&addr) {
                        println!("   Internal project: {}", resolved.project);
                    }
                }
                Err(e) => eprintln!("❌ Failed to set: {}", e),
            }
        }
        ResolvedAddress::ThreeLayer(addr) => {
            let db_path = Path::new("/tmp/regular_storage.db");
            let storage = match Storage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            match storage.set(&addr.project, &addr.namespace, &addr.key, addr.context.as_deref(), value, None) {
                Ok(_) => {
                    let display_key = if let Some(ctx) = &addr.context {
                        format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                    } else {
                        format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                    };
                    println!("✅ Set: {} = {}", display_key, value);
                }
                Err(e) => eprintln!("❌ Failed to set: {}", e),
            }
        }
    }
}

fn handle_get(address: ResolvedAddress) {
    match address {
        ResolvedAddress::FourLayer(addr) => {
            let db_path = Path::new("/tmp/meta_storage.db");
            let storage = match MetaStorage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            match storage.get(&addr) {
                Ok(Some(value)) => println!("✅ Get: {} = {}", addr.to_storage_key(), value),
                Ok(None) => println!("❌ Key not found: {}", addr.to_storage_key()),
                Err(e) => eprintln!("❌ Failed to get: {}", e),
            }
        }
        ResolvedAddress::ThreeLayer(addr) => {
            let db_path = Path::new("/tmp/regular_storage.db");
            let storage = match Storage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            match storage.get(&addr.project, &addr.namespace, &addr.key, addr.context.as_deref()) {
                Ok(Some(value)) => {
                    let display_key = if let Some(ctx) = &addr.context {
                        format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                    } else {
                        format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                    };
                    println!("✅ Get: {} = {}", display_key, value);
                }
                Ok(None) => {
                    let display_key = if let Some(ctx) = &addr.context {
                        format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                    } else {
                        format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                    };
                    println!("❌ Key not found: {}", display_key);
                }
                Err(e) => eprintln!("❌ Failed to get: {}", e),
            }
        }
    }
}

fn handle_del(address: ResolvedAddress) {
    match address {
        ResolvedAddress::FourLayer(addr) => {
            let db_path = Path::new("/tmp/meta_storage.db");
            let storage = match MetaStorage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            match storage.del(&addr) {
                Ok(true) => println!("✅ Deleted: {}", addr.to_storage_key()),
                Ok(false) => println!("❌ Key not found: {}", addr.to_storage_key()),
                Err(e) => eprintln!("❌ Failed to delete: {}", e),
            }
        }
        ResolvedAddress::ThreeLayer(addr) => {
            let db_path = Path::new("/tmp/regular_storage.db");
            let storage = match Storage::new(db_path) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("❌ Failed to create storage: {}", e);
                    return;
                }
            };

            match storage.del(&addr.project, &addr.namespace, &addr.key, addr.context.as_deref()) {
                Ok(true) => {
                    let display_key = if let Some(ctx) = &addr.context {
                        format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                    } else {
                        format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                    };
                    println!("✅ Deleted: {}", display_key);
                }
                Ok(false) => {
                    let display_key = if let Some(ctx) = &addr.context {
                        format!("{}.{}.{}__{}", addr.project, addr.namespace, addr.key, ctx)
                    } else {
                        format!("{}.{}.{}", addr.project, addr.namespace, addr.key)
                    };
                    println!("❌ Key not found: {}", display_key);
                }
                Err(e) => eprintln!("❌ Failed to delete: {}", e),
            }
        }
    }
}

fn print_usage() {
    println!("ProntoDB Address Resolution Test");
    println!();
    println!("USAGE:");
    println!("  prontodb <command> [--meta <meta>] [-p <project>] [-n <namespace>] <address> [value]");
    println!();
    println!("COMMANDS:");
    println!("  set <address> <value>   Store a value");
    println!("  get <address>           Retrieve a value"); 
    println!("  del <address>           Delete a value");
    println!();
    println!("EXAMPLES:");
    println!("  4-layer addressing:");
    println!("    prontodb set keeper.app.config.key value");
    println!("    prontodb get keeper.app.config.key");
    println!();
    println!("  --meta flag (3-layer becomes 4-layer):");
    println!("    prontodb set --meta keeper app.config.key value");
    println!("    prontodb get --meta lucas app.config.key");
    println!();
    println!("  Traditional 3-layer:");
    println!("    prontodb set app.config.key value");
    println!("    prontodb get app.config.key");
}