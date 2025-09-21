// Minimal dispatcher for testing routing
use rsb::prelude::*;

// Import RSB visual macros directly (compiler suggested)
use rsb::info;

pub fn pronto_dispatch(args: rsb::args::Args) -> i32 {
    info!("Dispatch called with {} args", args.all().len());
    
    if args.len() == 0 {
        info!("No command provided, showing help");
        return do_help(args);
    }

    let command = args.get_or(1, "");
    info!("Processing command: '{}'", command);
    
    // Try RSB dispatch! macro (now that global functions are fixed)
    dispatch!(&args, {
        "version" => do_version,
        "help" => do_help
    })
}

// Command stubs - RSB dispatch expects fn(Args) -> i32
fn do_set(mut args: Args) -> i32 {
    info!("Executing: set");
    // Surface parsed options for E2E verification
    if has_var("opt_verbose") { info!("Verbose mode enabled: {}", get_var("opt_verbose")); }
    if has_var("opt_debug") { info!("Debug mode enabled: {}", get_var("opt_debug")); }
    if has_var("opt_config") { info!("Config path: {}", get_var("opt_config")); }
    0
}

fn do_version(mut args: Args) -> i32 { info!("Executing: version"); 0 }

fn do_help(mut args: Args) -> i32 {
    info!("ProntoDB - Available Commands:");
    0
}
