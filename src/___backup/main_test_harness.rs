// Minimal main.rs to test Address4 parsing with --meta flag

use prontodb::addressing4::Address4;
use prontodb::storage_meta::MetaStorage;
use std::env;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        println!("Test harness for Address4 parsing");
        println!("Usage: prontodb <command> [--meta <meta>] <address> [value]");
        println!("\nExamples:");
        println!("  prontodb set keeper.app.config.key value");
        println!("  prontodb set --meta keeper app.config.key value");
        println!("  prontodb get keeper.app.config.key");
        println!("  prontodb get --meta lucas app.config.key");
        return;
    }

    let command = &args[1];
    
    // Parse --meta flag
    let (meta_flag, remaining_args) = if args.len() >= 4 && args[2] == "--meta" {
        (Some(args[3].clone()), args[4..].to_vec())
    } else {
        (None, args[2..].to_vec())
    };
    
    if remaining_args.is_empty() {
        eprintln!("Error: No address provided");
        return;
    }
    
    let address_str = &remaining_args[0];
    
    // Parse address with 4-layer support
    let dot_count = address_str.matches('.').count();
    
    let address = if dot_count >= 3 {
        // Full 4-dot address
        match Address4::parse(address_str) {
            Ok(addr) => {
                println!("Parsed 4-dot address:");
                println!("  Meta: {}", addr.meta);
                println!("  Project: {}", addr.project);
                println!("  Namespace: {}", addr.namespace);
                println!("  Key: {}", addr.key);
                addr
            }
            Err(e) => {
                eprintln!("Failed to parse 4-dot address: {}", e);
                return;
            }
        }
    } else if let Some(meta) = meta_flag {
        // 3-dot or less with --meta flag
        let parts: Vec<&str> = address_str.split('.').collect();
        let addr = match parts.len() {
            3 => Address4::new(&meta, parts[0], parts[1], parts[2]),
            2 => Address4::new(&meta, "default", parts[0], parts[1]),
            1 => Address4::new(&meta, "default", "default", parts[0]),
            _ => {
                eprintln!("Invalid address format");
                return;
            }
        };
        println!("Parsed with --meta flag:");
        println!("  Meta: {}", addr.meta);
        println!("  Project: {}", addr.project);
        println!("  Namespace: {}", addr.namespace);
        println!("  Key: {}", addr.key);
        addr
    } else {
        // Regular 3-dot, use default meta
        let parts: Vec<&str> = address_str.split('.').collect();
        let addr = match parts.len() {
            3 => Address4::new("default", parts[0], parts[1], parts[2]),
            2 => Address4::new("default", "default", parts[0], parts[1]),
            1 => Address4::new("default", "default", "default", parts[0]),
            _ => {
                eprintln!("Invalid address format");
                return;
            }
        };
        println!("Parsed as 3-layer with default meta:");
        println!("  Meta: {}", addr.meta);
        println!("  Project: {}", addr.project);
        println!("  Namespace: {}", addr.namespace);
        println!("  Key: {}", addr.key);
        addr
    };
    
    // Test with MetaStorage
    let db_path = Path::new("/tmp/test_addr4.db");
    let storage = MetaStorage::new(db_path).expect("Failed to create storage");
    
    // Register some test metas
    storage.register_meta("keeper", "pantheon_keeper", Some("Keeper's realm")).ok();
    storage.register_meta("lucas", "pantheon_lucas", Some("Lucas's realm")).ok();
    
    match command.as_str() {
        "set" if remaining_args.len() >= 2 => {
            let value = &remaining_args[1];
            match storage.set(&address, value, None) {
                Ok(_) => {
                    println!("\n✅ Successfully stored: {} = {}", address.to_storage_key(), value);
                    
                    // Show how it resolved
                    if let Ok(resolved) = storage.resolve_address(&address) {
                        println!("Resolved to internal project: {}", resolved.project);
                    }
                }
                Err(e) => eprintln!("❌ Failed to store: {}", e),
            }
        }
        "get" => {
            match storage.get(&address) {
                Ok(Some(value)) => {
                    println!("\n✅ Retrieved: {} = {}", address.to_storage_key(), value);
                }
                Ok(None) => {
                    println!("\n❌ Key not found: {}", address.to_storage_key());
                }
                Err(e) => eprintln!("❌ Failed to retrieve: {}", e),
            }
        }
        "del" => {
            match storage.del(&address) {
                Ok(true) => println!("\n✅ Deleted: {}", address.to_storage_key()),
                Ok(false) => println!("\n❌ Key not found: {}", address.to_storage_key()),
                Err(e) => eprintln!("❌ Failed to delete: {}", e),
            }
        }
        _ => {
            eprintln!("Unknown command or missing value for set");
        }
    }
}