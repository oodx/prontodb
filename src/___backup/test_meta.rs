// Quick test to verify 4-layer addressing with meta table

use prontodb::addressing4::Address4;
use prontodb::storage_meta::MetaStorage;
use std::path::Path;

fn main() {
    println!("Testing 4-layer meta addressing...\n");
    
    // Create test database
    let db_path = Path::new("/tmp/test_meta.db");
    let storage = MetaStorage::new(db_path).expect("Failed to create storage");
    
    // Register meta namespaces for pantheon users
    println!("1. Registering meta namespaces:");
    storage.register_meta("keeper", "pantheon_keeper", Some("Keeper's realm"))
        .expect("Failed to register keeper");
    storage.register_meta("lucas", "pantheon_lucas", Some("Lucas's realm"))
        .expect("Failed to register lucas");
    println!("   ✓ Registered keeper -> pantheon_keeper");
    println!("   ✓ Registered lucas -> pantheon_lucas\n");
    
    // Test 4-dot addressing
    println!("2. Testing 4-dot addresses:");
    let keeper_addr = Address4::parse("keeper.app.config.api_key").unwrap();
    let lucas_addr = Address4::parse("lucas.app.config.api_key").unwrap();
    
    println!("   Keeper: {}", keeper_addr.to_storage_key());
    println!("   Lucas:  {}", lucas_addr.to_storage_key());
    
    // Test resolution
    println!("\n3. Testing address resolution:");
    let keeper_resolved = storage.resolve_address(&keeper_addr)
        .expect("Failed to resolve keeper address");
    let lucas_resolved = storage.resolve_address(&lucas_addr)
        .expect("Failed to resolve lucas address");
    
    println!("   keeper.app.config.api_key resolves to:");
    println!("     Project: {}", keeper_resolved.project);
    println!("     Namespace: {}", keeper_resolved.namespace);
    println!("     Key: {}", keeper_resolved.key);
    
    println!("   lucas.app.config.api_key resolves to:");
    println!("     Project: {}", lucas_resolved.project);
    println!("     Namespace: {}", lucas_resolved.namespace); 
    println!("     Key: {}", lucas_resolved.key);
    
    // Store and retrieve with isolation
    println!("\n4. Testing storage isolation:");
    storage.set(&keeper_addr, "keeper_secret_123", None)
        .expect("Failed to set keeper value");
    storage.set(&lucas_addr, "lucas_secret_456", None)
        .expect("Failed to set lucas value");
    
    let keeper_value = storage.get(&keeper_addr)
        .expect("Failed to get keeper value");
    let lucas_value = storage.get(&lucas_addr)
        .expect("Failed to get lucas value");
    
    println!("   Keeper retrieves: {:?}", keeper_value);
    println!("   Lucas retrieves:  {:?}", lucas_value);
    
    // Verify isolation
    println!("\n5. Verifying isolation:");
    if keeper_value == Some("keeper_secret_123".to_string()) &&
       lucas_value == Some("lucas_secret_456".to_string()) {
        println!("   ✓ ISOLATION WORKING! Each user sees only their data!");
    } else {
        println!("   ✗ ISOLATION FAILED! Data mixed up!");
    }
    
    // List all metas
    println!("\n6. Listing all registered metas:");
    let metas = storage.list_metas().expect("Failed to list metas");
    for (meta, project, desc) in metas {
        println!("   {} -> {} ({})", meta, project, desc.unwrap_or_default());
    }
    
    println!("\n✅ Test complete!");
}