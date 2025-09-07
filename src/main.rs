// ProntoDB v0.1 - RSB-compliant implementation
// Clean main with minimal entry point and dispatch only

use rsb::prelude::*;

mod prontodb;
use prontodb::*;

fn main() {
    let args = bootstrap!();
    
    dispatch!(&args, {
        "install" => do_install,
        "uninstall" => do_uninstall,
        "set" => do_set,
        "get" => do_get,
        "del" => do_del,
        "keys" => do_keys,
        "ls" => do_keys,
        "projects" => do_projects,
        "namespaces" => do_namespaces,
        "nss" => do_nss,
        "backup" => do_backup,
        "stream" => do_stream,
        "admin" => do_admin
    });
}