
// Use RSB prelude for macros (bootstrap!/pre_dispatch!/dispatch!)
use rsb::prelude::*;
use prontodb::core::dispatch::pronto_dispatch;

use std::process::exit;

fn main() {
    // Bootstrap - get RSB Args
    let args = bootstrap!();
    
    // Process options and populate global context
    options!(&args);

    // Dispatch to command handler
    let exit_code = pronto_dispatch(args);
    exit(exit_code);
}