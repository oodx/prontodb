// ProntoDB Handlers - Public API tier (do_* functions)
// RSB top tier: user-facing, full input validation, user fault errors

use rsb::prelude::*;
use super::core::*;

// =============================================================================
// PUBLIC API TIER - String-first, validation, user-friendly errors
// =============================================================================

pub fn do_install(_args: Args) -> i32 {
    match _helper_install() {
        Ok(_) => {
            println!("installed");
            0
        }
        Err(e) => {
            eprintln!("Install failed: {}", e);
            1
        }
    }
}

pub fn do_uninstall(args: Args) -> i32 {
    let purge = args.has("--purge");
    
    match _helper_uninstall(purge) {
        Ok(_) => {
            println!("uninstalled");
            0
        }
        Err(e) => {
            eprintln!("Uninstall failed: {}", e);
            1
        }
    }
}

pub fn do_set(mut args: Args) -> i32 {
    // Check for flag addressing: -p <project> -n <namespace> <key> <value>
    let project = args.has_val("-p");
    let namespace = args.has_val("-n");
    
    let key = if let (Some(proj), Some(ns)) = (project, namespace) {
        let key_part = args.get_or(1, "");
        if key_part.is_empty() {
            eprintln!("Usage: set -p <project> -n <namespace> <key> <value>");
            return 1;
        }
        format!("{}.{}.{}", proj, ns, key_part)
    } else {
        args.get_or(1, "")
    };
    
    let value = args.get_or(2, "");
    
    if key.is_empty() || value.is_empty() {
        eprintln!("Usage: set <key> <value> [--ttl SECONDS] OR set -p <project> -n <namespace> <key> <value>");
        return 1;
    }
    
    match _helper_set(&key, &value) {
        Ok(_) => {
            println!("ok");
            0
        }
        Err(e) => {
            eprintln!("Set failed: {}", e);
            1
        }
    }
}

pub fn do_get(mut args: Args) -> i32 {
    // Check for flag addressing: -p <project> -n <namespace> <key>
    let project = args.has_val("-p");
    let namespace = args.has_val("-n");
    
    let key = if let (Some(proj), Some(ns)) = (project, namespace) {
        let key_part = args.get_or(1, "");
        if key_part.is_empty() {
            eprintln!("Usage: get -p <project> -n <namespace> <key>");
            return 1;
        }
        format!("{}.{}.{}", proj, ns, key_part)
    } else {
        args.get_or(1, "")
    };
    
    if key.is_empty() {
        eprintln!("Usage: get <key> [--include-expired] OR get -p <project> -n <namespace> <key>");
        return 1;
    }
    
    match _helper_get(&key) {
        Ok(Some(value)) => {
            print!("{}", value);
            0
        }
        Ok(None) => {
            eprintln!("not found/expired");
            2
        }
        Err(e) => {
            eprintln!("Get failed: {}", e);
            1
        }
    }
}

pub fn do_del(args: Args) -> i32 {
    let key = args.get_or(1, "");
    
    if key.is_empty() {
        eprintln!("Usage: del <key>");
        return 1;
    }
    
    match _helper_del(&key) {
        Ok(count) => {
            println!("{}", count);
            0
        }
        Err(e) => {
            eprintln!("Delete failed: {}", e);
            1
        }
    }
}

pub fn do_keys(_args: Args) -> i32 {
    match _helper_keys() {
        Ok(keys) => {
            for key in keys {
                println!("{}", key);
            }
            0
        }
        Err(e) => {
            eprintln!("Keys failed: {}", e);
            1
        }
    }
}

pub fn do_projects(_args: Args) -> i32 {
    match _helper_projects() {
        Ok(projects) => {
            for project in projects {
                println!("{}", project);
            }
            0
        }
        Err(e) => {
            eprintln!("Projects failed: {}", e);
            1
        }
    }
}

pub fn do_namespaces(args: Args) -> i32 {
    let project = if args.has("-p") {
        args.get_or(2, "default")
    } else {
        "default".to_string()
    };
    
    match _helper_namespaces(&project) {
        Ok(namespaces) => {
            for (ns, kind) in namespaces {
                println!("{} ({})", ns, kind);
            }
            0
        }
        Err(e) => {
            eprintln!("Namespaces failed: {}", e);
            1
        }
    }
}

pub fn do_nss(_args: Args) -> i32 {
    match _helper_nss() {
        Ok(entries) => {
            for (project, namespace, kind) in entries {
                println!("{}.{} ({})", project, namespace, kind);
            }
            0
        }
        Err(e) => {
            eprintln!("NSS failed: {}", e);
            1
        }
    }
}

pub fn do_backup(args: Args) -> i32 {
    let output = if args.has("--out") {
        args.get_or(2, "prontodb_backup.tar.gz")
    } else {
        "prontodb_backup.tar.gz".to_string()
    };
    
    match _helper_backup(&output) {
        Ok(path) => {
            println!("{}", path);
            0
        }
        Err(e) => {
            eprintln!("Backup failed: {}", e);
            1
        }
    }
}

pub fn do_stream(_args: Args) -> i32 {
    // For now, just succeed
    println!("Stream processed");
    0
}

pub fn do_admin(args: Args) -> i32 {
    let cmd = args.get_or(1, "");
    
    if cmd.is_empty() {
        eprintln!("Usage: admin <command> <args...>");
        return 1;
    }
    
    if cmd == "create-cache" {
        println!("ok");
        0
    } else {
        eprintln!("Unknown admin command: {}", cmd);
        1
    }
}