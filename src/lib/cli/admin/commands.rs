use std::fmt;
use std::str::FromStr;

use crate::lib::core::crud::{CrudObjectKind, CrudVerb};
use rsb::prelude::*;

#[derive(Debug)]
pub struct CommandError {
    message: String,
}

impl CommandError {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for CommandError {}

#[derive(Clone, Debug)]
pub enum AdminCommand {
    Capabilities,
    Crud {
        object: CrudObjectKind,
        verb: CrudVerb,
    },
}

pub fn resolve_command() -> Result<AdminCommand, CommandError> {
    if has_var("opt_capabilities") {
        return Ok(AdminCommand::Capabilities);
    }

    let object_raw = get_var("opt_object");
    let verb_raw = get_var("opt_verb");

    if object_raw.is_empty() || verb_raw.is_empty() {
        return Err(CommandError::new(
            "missing required options: --object=<base|table|record> --verb=<crud verb>",
        ));
    }

    let object = CrudObjectKind::from_str(&object_raw)
        .map_err(|_| CommandError::new(format!("unknown object kind: {}", object_raw)))?;
    let verb = CrudVerb::from_str(&verb_raw)
        .map_err(|_| CommandError::new(format!("unknown verb: {}", verb_raw)))?;

    Ok(AdminCommand::Crud { object, verb })
}

pub fn usage() -> &'static str {
    "prontodb-admin --object=<base|table|record> --verb=<create|read|update|delete|list|find|backup|restore|alias> [--database-path=PATH] [--target-path=PATH] [--source-path=PATH]"
}
