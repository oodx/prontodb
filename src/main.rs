// ProntoDB v0.1 - Pragmatic MVP implementation
// Main entry point with command dispatcher

mod dispatcher;
mod addressing;
mod storage;
mod xdg;

// Prefer RSB's args! macro for ergonomic argument collection
use rsb::args;

fn main() {
    // Get command line arguments via RSB macro (string-first)
    let args: Vec<String> = args!();
    
    // Dispatch to command handler
    let exit_code = dispatcher::dispatch(args);
    
    // Exit with appropriate code
    std::process::exit(exit_code);
}
