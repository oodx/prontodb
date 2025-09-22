// Core ProntoDB Application - NOT the admin CLI
// The admin CLI is separate in src/bin/admin-cli.rs

use rsb::prelude::*;
//new base dispatch

fn main() {
    // Core ProntoDB app bootstrap
    let args = bootstrap!();
    options!(&args);

    // Core application dispatch (separate from admin CLI)
    std::process::exit(0); // Placeholder - implement core_dispatch later
}
