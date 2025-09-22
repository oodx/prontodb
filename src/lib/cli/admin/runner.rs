use crate::lib::adpt::sqlite::{
    SqliteBaseAdapter, SqliteConnectionConfig, SqliteRecordAdapter, SqliteTableAdapter,
};
use crate::lib::core::crud::{
    CrudContext, CrudDomain, CrudError, CrudObjectKind, CrudResource, CrudVerb,
};
use rsb::prelude::*;

use super::commands::{self, AdminCommand, CommandError};

pub fn run_admin_cli() -> i32 {
    let args = bootstrap!();
    options!(&args);

    match commands::resolve_command() {
        Ok(AdminCommand::Capabilities) => {
            print_capabilities();
            0
        }
        Ok(AdminCommand::Crud { object, verb }) => match execute_crud(object, verb) {
            Ok(_) => 0,
            Err(error) => {
                eprintln!("error: {}", error);
                1
            }
        },
        Err(error) => {
            eprintln!("{}\nUsage: {}", error, commands::usage());
            1
        }
    }
}

fn print_capabilities() {
    let config = SqliteConnectionConfig::default();
    let base = SqliteBaseAdapter::new(config.clone());
    let table = SqliteTableAdapter::new(config.clone());
    let record = SqliteRecordAdapter::new(config);

    println!("[capabilities] base");
    render_capability_entries(base.capabilities());
    println!("[capabilities] table");
    render_capability_entries(table.capabilities());
    println!("[capabilities] record");
    render_capability_entries(record.capabilities());
}

fn execute_crud(object: CrudObjectKind, verb: CrudVerb) -> Result<(), CrudError> {
    let config = SqliteConnectionConfig::default();
    let mut ctx = CrudContext::new(CrudDomain::Sqlite, object.clone(), verb);
    hydrate_context_options(&mut ctx);

    let outcome = match object {
        CrudObjectKind::Base => SqliteBaseAdapter::new(config.clone()).dispatch(verb, ctx),
        CrudObjectKind::Table => SqliteTableAdapter::new(config.clone()).dispatch(verb, ctx),
        CrudObjectKind::Record => SqliteRecordAdapter::new(config).dispatch(verb, ctx),
        other => Err(CrudError::unsupported(CrudDomain::Sqlite, other, verb)),
    }?;

    println!("{:?}", outcome.status);
    Ok(())
}

pub fn ensure_capability_toggle() -> Result<(), CommandError> {
    if !has_var("opt_object") && !has_var("opt_capabilities") {
        return Err(CommandError::new("no admin action requested"));
    }
    Ok(())
}

fn hydrate_context_options(ctx: &mut CrudContext) {
    let database_path = get_var("opt_database_path");
    if !database_path.is_empty() {
        ctx.options.insert("database_path".into(), database_path);
    }

    let target_path = get_var("opt_target_path");
    if !target_path.is_empty() {
        ctx.options.insert("target_path".into(), target_path);
    }

    let source_path = get_var("opt_source_path");
    if !source_path.is_empty() {
        ctx.options.insert("source_path".into(), source_path);
    }
}

fn render_capability_entries(map: crate::lib::core::crud::CapabilityMap) {
    let entries: Vec<String> = map.entries().map(|entry| entry.to_string()).collect();
    if entries.is_empty() {
        println!("  (no verbs registered)");
    } else {
        for entry in entries {
            println!("  {}", entry);
        }
    }
}
