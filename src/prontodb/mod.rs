// ProntoDB module - RSB-compliant business logic
// Following RSB architecture patterns

pub mod core;
pub mod utils;
pub mod handlers;

// Re-export main functions for clean interface
pub use core::*;
pub use utils::*;
pub use handlers::*;