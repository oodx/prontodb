// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with command dispatcher

mod dispatcher;
mod addressing;
mod storage;
mod xdg;
mod api;

// Use RSB prelude for macros (bootstrap!/pre_dispatch!/args!)
use rsb::prelude::*;

fn install_cmd(_args: rsb::args::Args) -> i32 {
    eprintln!("Install not implemented in MVP");
    1
}

fn uninstall_cmd(_args: rsb::args::Args) -> i32 {
    eprintln!("Uninstall not implemented in MVP");
    1
}

fn backup_cmd(_args: rsb::args::Args) -> i32 {
    eprintln!("Backup not implemented in MVP");
    1
}

fn main() {
    // Get arguments via RSB; run pre-dispatch for bootstrap commands
    let vec_args: Vec<String> = args!();
    let rsb_args = rsb::args::Args::new(&vec_args);

    if pre_dispatch!(&vec_args, {
        "install" => install_cmd,
        "uninstall" => uninstall_cmd,
        "backup" => backup_cmd
    }) {
        return;
    }
    
    // Dispatch to command handler
    let exit_code = dispatcher::dispatch(vec_args);
    
    // Exit with appropriate code
    std::process::exit(exit_code);
}
