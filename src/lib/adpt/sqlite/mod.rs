//! SQLite adapters implementing the core CRUD traits.

mod base;
mod record;
mod table;
pub mod utils;

pub use base::SqliteBaseAdapter;
pub use record::SqliteRecordAdapter;
pub use table::SqliteTableAdapter;
pub use utils::{SqliteConnectionConfig, SqlitePathResolver};
