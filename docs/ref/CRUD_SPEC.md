# ProntoDB CRUD+ Specification

_Last updated: 2025-09-21_

## Purpose
ProntoDB is rebuilding its storage surface around a Forge-inspired **CRUD+** abstraction. This document translates the Forge concepts for zero-context readers and scopes how they apply inside ProntoDB today.

## Forge Concepts (Reader Primer)
Forge is the Oxidex ecosystem’s universal CRUD engine. It standardizes how any "Domain Object" (file, table, queue item, etc.) is created, inspected, mutated, and cataloged. The core ideas we adopt:

- **Domains** describe where an object lives (e.g., filesystem, SQLite).
- **Domain Objects (xDOs)** are typed handles within a domain (file, folder, table, record).
- **CRUD+ Verbs**: `create`, `read`, `update`, `delete` plus `list`, `find`, `backup`, `restore`, `alias`, `invalid`.
- **Lifecycle Stages**: normalize inputs → probe existence/capabilities → apply operation → emit outcome.
- **Capabilities**: Adapters declare which verbs they support for each object kind; unsupported verbs fail deterministically.
- **Hooks**: Before/after/error hooks let policy/logging wrap every operation.

Forge is intentionally domain-agnostic; it defines the assembly line. ProntoDB embraces the same vocabulary without importing the entire Forge runtime.

## ProntoDB Scope
Our immediate focus is the SQLite domain. The CRUD+ layer will:

1. Provide a trait (`CrudResource`) that models the Forge verbs and lifecycle.
2. Represent domain/object metadata via enums/structs so adapters can advertise capabilities.
3. Ship SQLite adapters for database base, table, and record objects.
4. Expose the adapters through an admin CLI backed by RSB bootstrap/options/dispatch patterns.
5. Guarantee that every RSB feature we use (GLOBAL, HOST, OPTIONS, STRINGS, FS) has a matching sanity test.

We are not implementing Foundry orchestration, queues, or non-SQLite domains yet.

## Architecture Overview
```
+--------------------+
| admin CLI (RSB)    |  <- commands call CRUD verbs
+---------+----------+
          |
          v
+--------------------+
| core::crud module  |  <- trait, context, lifecycle, capability map
+---------+----------+
          |
          v
+--------------------+
| sqlite adapters    |  <- base/table/record adapters, utils
+--------------------+
```

### core::crud module
- `mod.rs`: orchestrator per MODULE_SPEC.
- `trait CrudResource`: verbs, lifecycle hooks, capability queries.
- Supporting types: `CrudDomain`, `CrudObjectKind`, `CrudVerb`, `CrudContext`, `CrudOutcome`, `CrudError`.
- Lifecycle helpers: `pre_stage`, `post_stage`, `on_error` default implementations.

### sqlite adapters
- `base.rs`: database-level tasks (create/open, backup, restore, metadata).
- `table.rs`: table existence, schema inspection, list/find.
- `record.rs`: row-level read/update/delete.
- `utils.rs`: connection helpers, WAL toggles, path normalization, capability registration.

## CRUD+ Verbs (ProntoDB interpretation)
| Verb      | Description | Example |
|-----------|-------------|---------|
| create    | Make a new object | Create table, create record |
| read      | Fetch metadata or content | Inspect table schema |
| update    | Modify existing object | Alter record |
| delete    | Remove object | Drop table or record |
| list      | Enumerate children | List table names |
| find      | Query by property | Filter records by key |
| backup    | Export object | Dump database/table |
| restore   | Rehydrate object | Load backup into fresh base |
| alias     | Provide alternate handles | Alias table name (optional) |
| invalid   | Explicitly unsupported verb | Adapter returns `Unsupported` |

Adapters must implement only the verbs they support. Unsupported verbs should return a structured `CrudError::unsupported()` pointing to the verb/object/domain.

## Lifecycle Hooks
All operations flow through three stages:
1. **normalize** – parse inputs, apply defaults, resolve references via RSB Global if needed.
2. **probe** – check existence, validate capabilities, gather preconditions.
3. **apply** – perform the operation, capture `CrudOutcome`.

Hooks available to adapters:
- `fn before(&self, verb, ctx) -> Result<()>`
- `fn after(&self, verb, ctx, &CrudOutcome)`
- `fn on_error(&self, verb, ctx, &CrudError)`

ProntoDB adapters can override defaults to plug logging, metrics, retries, etc. Future work may expose hook registries for cross-cutting policies.

## Capabilities & Metadata
- Each adapter exposes a `CapabilityMap` advertising per-verb support.
- The admin CLI will include a `capabilities` command that prints what operations are allowed for each object kind.
- Metadata structures (`CrudMetadata`) capture size, timestamps, schema details, or adapter-defined extras.

## Error Handling
- `CrudError` wraps `hub::error_ext::anyhow::Error` and includes verb/domain/object identifiers for observability.
- Helper constructors: `CrudError::unsupported`, `CrudError::invalid_input`, `CrudError::conflict`, `CrudError::not_found`.

## RSB Integration Requirements
- **GLOBAL**: store/resolve user options, connection strings, file paths.
- **HOST**: XDG-aware database placement.
- **OPTIONS**: parse CLI flags (`--list`, `--find` etc.).
- **STRINGS**: name normalization (snake/camel case transforms).
- **FS**: backup/restore file IO.
Each feature requires a sanity test in `tests/sanity/` (e.g., `sanity/global_cli.rs`).

## Deliverables Checklists
1. CRUD Specification Doc (this file) ✅
2. Core trait + lifecycle module
3. SQLite adapters (base/table/record + utils)
4. Admin CLI module + binary
5. RSB sanity suites
6. Updated START/QUICK_REF referencing CRUD+

## Out of Scope (for now)
- Non-SQLite domains (filesystem, network services).
- Advanced Forge concepts like multi-stage policy composition or Foundry workflows.
- Automated migration from previous ProntoDB schema.

## Glossary
- **CRUD+**: CRUD plus list/find/backup/restore/alias/invalid verbs.
- **Capability**: Declared support for a verb/object pairing.
- **Lifecycle Hook**: Method invoked before/after/apply/error for each operation.
- **Domain**: Logical storage location category (SQLite, filesystem, etc.).
- **Object Kind**: Specific entity in a domain (base, table, record).

## Next Steps
- Implement `core::crud` module per FORGE-inspired design.
- Build SQLite adapters and run baseline tests.
- Ship admin CLI with documentation referencing this primer.
- Ensure START.txt and QUICK_REF highlight CRUD+ requirements.

