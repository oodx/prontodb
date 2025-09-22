//! Admin CLI module (MODULE_SPEC orchestrator).

mod commands;
mod runner;

pub use commands::{usage, AdminCommand, CommandError};
pub use runner::{ensure_capability_toggle, run_admin_cli};
