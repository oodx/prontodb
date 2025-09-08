# ProntoDB

A fast, single-binary CLI key-value store built on SQLite with namespace support and TTL capabilities.

## Features

- **Simple CLI**: Single binary with intuitive commands
- **Namespaced Storage**: Organize data by project and namespace  
- **TTL Support**: Create time-to-live namespaces for caching
- **Multiple Addressing**: Use `-p/-n` flags or `project.namespace.key` paths
- **Context Support**: Store related values with `key__context` syntax
- **XDG Compliance**: Respects XDG Base Directory specification
- **RSB Integration**: Built with Rebel String-Biased architecture patterns

## Quick Start

```bash
# Build the project
cargo build --release

# Basic usage
./target/release/prontodb -p myproject -n data set config "debug=true"
./target/release/prontodb -p myproject -n data get config

# Full path addressing
./target/release/prontodb set myproject.data.user_id "12345"
./target/release/prontodb get myproject.data.user_id

# Context addressing
./target/release/prontodb set myproject.config.db__prod "host=prod.db"
./target/release/prontodb set myproject.config.db__dev "host=dev.db"

# TTL namespace
./target/release/prontodb create-cache myproject.sessions timeout=3600
./target/release/prontodb -p myproject -n sessions set session_123 "active"
```

## Commands

### Core Operations
- `set <key> <value>` - Store a value
- `get <key>` - Retrieve a value  
- `del <key>` - Delete a value
- `keys [prefix]` - List keys (requires -p and -n)
- `scan [prefix]` - List key-value pairs (requires -p and -n)

### Discovery
- `projects` - List all projects
- `namespaces -p <project>` - List namespaces in project
- `nss` - List all project.namespace combinations

### TTL Management
- `create-cache <project.namespace> timeout=SECONDS` - Create TTL namespace

### Addressing Options
- `-p <project>` - Set project context
- `-n <namespace>` - Set namespace context  
- `--ns-delim <char>` - Override delimiter (default: '.')

## Testing

### Run All Tests
```bash
cargo test
```

### User Acceptance Testing (UAT)
Run the ceremonious feature walkthrough:

```bash
./bin/uat.sh
```

The UAT script provides a comprehensive demonstration of all ProntoDB features with visual terminal output using `boxy`. It covers:

- Help system and discovery
- Basic CRUD operations
- Context addressing (`key__context`)
- Full path addressing (`project.namespace.key`)
- Keys and scan operations with prefix filtering
- TTL namespace creation and rule enforcement
- Error handling and exit codes
- Cleanup and data management

**UAT Options:**
- Set `CLEANUP_ON_EXIT=0` to preserve test data
- The script automatically cleans up test data by default

## Exit Codes

- `0` - Success
- `2` - Key not found or expired (MISS)
- `1` - Error

## Storage

- **Location**: XDG-compliant paths (`~/.local/share/odx/prontodb/pronto.db`)
- **Override**: Set `PRONTO_DB` environment variable for custom location
- **Format**: SQLite with WAL mode enabled
- **Schema**: Single `kv` table for MVP (per-namespace tables planned for future)

## Development

### Architecture Overview

- **RSB Integration**: Uses Rebel String-Biased patterns for argument processing
- **Modular Design**: Separate modules for addressing, storage, XDG paths, and API
- **Test Coverage**: Unit tests, integration tests, and sanity tests
- **Documentation**: Comprehensive docs in `.eggs/` directory (via China agent)

### Key Files
- `src/main.rs` - RSB-integrated entry point with lifecycle commands
- `src/dispatcher.rs` - Command routing and argument parsing
- `src/api.rs` - Centralized API layer for CLI operations  
- `src/storage.rs` - SQLite storage implementation
- `src/addressing.rs` - Namespace addressing and validation
- `src/xdg.rs` - XDG Base Directory compliance

### Building from Source
```bash
git clone <repository>
cd prontodb
cargo build --release
cargo test
./bin/uat.sh  # Run comprehensive feature demo
```

## Contributing

1. Review project documentation in `.eggs/` directory
2. Study `docs/RSB_USAGE.md` for RSB integration patterns
3. Run tests to ensure functionality
4. Follow existing code patterns and RSB principles

## License

[Add your license here]