// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with command dispatcher

mod dispatcher;
mod addressing;
mod storage;
mod xdg;

use std::env;

fn main() {
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Dispatch to command handler
    let exit_code = dispatcher::dispatch(args);
    
    // Exit with appropriate code
    std::process::exit(exit_code);
}
