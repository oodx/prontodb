# ProntoDB

**A fast, production-ready, single-binary CLI key-value store built on SQLite with namespace support and TTL capabilities.**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-31%20passing-green.svg)](#testing)

ProntoDB provides a lightweight, namespaced key-value database with time-to-live (TTL) support, built with [RSB framework](https://github.com/oodx/rsb-framework) integration and designed for developers who need fast, reliable data storage with simple CLI operations.

## 🚀 Key Features

- **🏃 Single Binary**: No external dependencies, just one executable
- **📍 Dot Addressing**: Primary `project.namespace.key` syntax for intuitive data organization
- **📁 Namespaced Storage**: Hierarchical organization with automatic project/namespace creation
- **⏰ TTL Support**: Create time-to-live namespaces for caching with automatic expiry
- **🔀 Flexible Addressing**: Dot notation (primary), flags (`-p/-n`), and context syntax (`key__context`)
- **📂 XDG Compliance**: Follows XDG Base Directory specification for storage
- **🧪 RSB Integration**: Built with Rebel String-Biased architecture patterns
- **⚡ SQLite Backend**: Fast, reliable storage with WAL journaling mode
- **🎯 Predictable Exit Codes**: Standard Unix exit codes (0=success, 1=error, 2=miss)

## 📦 Installation

### Quick Install (Recommended)

```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
./bin/deploy.sh
```

The deploy script will:
- Build the release binary
- Install to `~/.local/lib/odx/prontodb/prontodb`
- Create a symlink at `~/.local/bin/odx/prontodb`
- Verify installation with functional tests
- Display usage examples

Ensure `~/.local/bin/odx` is in your PATH:
```bash
echo 'export PATH="$HOME/.local/bin/odx:$PATH"' >> ~/.bashrc
source ~/.bashrc
```

### Manual Build

```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
cargo build --release
# Binary will be at: ./target/release/prontodb
```

### Verify Installation

```bash
prontodb version     # Should output: prontodb 0.1.0
prontodb help        # Show all available commands
```

## 🎯 Quick Start

### Dot Addressing (Primary Method)

ProntoDB uses **dot addressing** as the primary way to specify keys. This is the recommended approach:

```bash
# Store values using dot addressing (project.namespace.key)
prontodb set myapp.config.environment "production"
prontodb set myapp.config.debug true
prontodb set myapp.secrets.api_key "secret123"

# Retrieve values
prontodb get myapp.config.environment
prontodb get myapp.config.debug

# Delete values  
prontodb del myapp.config.debug

# Check if key exists (exit code 2 = not found)
prontodb get myapp.config.nonexistent
echo $?  # Will be 2
```

### Flag Addressing (Alternative Method)

You can also use flags when needed:

```bash
# Equivalent operations using flags
prontodb set -p myapp -n config environment "production"
prontodb get -p myapp -n config environment
```

### Context Addressing

Store related configuration with the `key__context` pattern:

```bash
# Database configurations for different environments
prontodb set myapp.config.database__prod "host=prod.db.com,user=app"
prontodb set myapp.config.database__dev "host=dev.db.com,user=dev"
prontodb set myapp.config.database__test "host=localhost,user=test"

# Retrieve specific context
prontodb get myapp.config.database__prod
```

### TTL Namespaces (Caching)

Create namespaces with automatic expiry:

```bash
# Create a cache namespace with 1-hour TTL
prontodb create-cache myapp.sessions timeout=3600

# Store session data (will auto-expire in 1 hour)
prontodb -p myapp -n sessions set user_123 "active_session_data"
prontodb -p myapp -n sessions set user_456 "another_session"

# Regular retrieval
prontodb get myapp.sessions.user_123
```

### Discovery Commands

```bash
# List all projects
prontodb projects

# List namespaces in a project
prontodb -p myapp namespaces

# List all project.namespace combinations
prontodb nss

# List keys in a namespace
prontodb -p myapp -n config keys

# List key-value pairs with optional prefix filtering
prontodb -p myapp -n config scan
prontodb -p myapp -n config scan db  # Only keys starting with "db"
```

## 📚 Complete Command Reference

### Core Operations
```bash
prontodb set <key> <value>        # Store a value
prontodb get <key>                # Retrieve a value
prontodb del <key>                # Delete a value
prontodb keys [prefix]            # List keys (requires -p and -n)
prontodb scan [prefix]            # List key-value pairs (requires -p and -n)
```

### Discovery & Navigation
```bash
prontodb projects                 # List all projects
prontodb -p <project> namespaces  # List namespaces in project
prontodb nss                      # List all project.namespace combinations
```

### TTL Management
```bash
prontodb create-cache <project.namespace> timeout=<seconds>
```

### Addressing Options
```bash
-p <project>              # Set project context
-n <namespace>            # Set namespace context
--ns-delim <character>    # Override delimiter (default: '.')
```

### System Commands
```bash
prontodb help             # Show help information
prontodb version          # Show version information
```

## 🧪 Testing & Quality Assurance

ProntoDB includes comprehensive testing:

### Run All Tests
```bash
cargo test  # Runs 31 tests across unit, integration, and sanity suites
```

### User Acceptance Testing
Experience all features with the ceremonious UAT demonstration:

```bash
./bin/uat.sh
```

The UAT script provides an interactive walkthrough covering:
- Help system and discovery commands
- Basic CRUD operations with all addressing modes
- Context addressing (`key__context`) examples
- Full path addressing (`project.namespace.key`)
- Keys and scan operations with prefix filtering
- TTL namespace creation and expiry behavior
- Error handling and exit code validation
- Data cleanup and management

**UAT Options:**
- Set `CLEANUP_ON_EXIT=0` to preserve test data for inspection
- Automatic cleanup runs by default

### Test Coverage
- **Unit Tests**: 10 tests covering core functionality
- **Integration Tests**: 3 tests for end-to-end workflows  
- **Sanity Tests**: 8 tests for behavioral verification
- **Additional Suites**: 10 tests for extended coverage

## 🏗️ Architecture & Design

### Storage Design
- **Backend**: SQLite with WAL (Write-Ahead Logging) mode
- **Location**: XDG-compliant (`~/.local/share/odx/prontodb/pronto.db`)
- **Override**: Set `PRONTO_DB` environment variable for custom location
- **Schema**: Single `kv` table for MVP (designed for future per-namespace optimization)

### RSB Integration
ProntoDB is built using [Rebel String-Biased (RSB) framework](https://github.com/oodx/rsb-framework) patterns:
- String-biased API design for CLI operations
- Proper RSB main entry pattern with `args!()` and `pre_dispatch!()`
- Lifecycle command support (install/uninstall/backup)
- Documented RSB usage patterns for team learning

### Exit Codes
Following Unix conventions:
- `0` - Success
- `1` - Error (invalid syntax, storage failure, etc.)
- `2` - Key not found or expired (MISS)

### Project Structure
```
src/
├── main.rs           # RSB-integrated entry point
├── dispatcher.rs     # Command routing and argument parsing
├── api.rs           # Centralized API layer for CLI operations
├── storage.rs       # SQLite storage implementation  
├── addressing.rs    # Namespace addressing and validation
└── xdg.rs          # XDG Base Directory compliance
```

## 🔧 Development

### Building from Source
```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
cargo build --release
cargo test
./bin/uat.sh  # Run comprehensive feature demo
```

### Optional Features
Configure build with Cargo features:
```bash
# Default build (includes JSON support and bundled SQLite)
cargo build --release

# System SQLite (smaller binary)
cargo build --release --no-default-features --features json

# With compression support
cargo build --release --features compression-zstd

# With encryption support  
cargo build --release --features encryption-aes
```

### Documentation
Comprehensive project documentation available in:
- `.eggs/` directory - Complete knowledge base created by China agent
- `docs/` directory - Technical specifications and RSB integration guides
- `MVP_STATUS.md` - Detailed completion status and testing results

## 🤝 Contributing

1. Review project documentation in `.eggs/` directory for comprehensive context
2. Study `docs/RSB_USAGE.md` for RSB integration patterns and team standards
3. Run tests to ensure functionality: `cargo test`
4. Follow existing code patterns and RSB architectural principles
5. Add tests for new functionality
6. Run the UAT script to verify end-to-end behavior

## 📋 Production Readiness

ProntoDB is production-ready with:
- ✅ **Complete feature set**: All MVP features plus advanced capabilities
- ✅ **Comprehensive testing**: 31 tests covering all functionality  
- ✅ **Professional deployment**: Automated deployment script with verification
- ✅ **Clean architecture**: Modular, maintainable Rust codebase
- ✅ **RSB compliance**: Follows team architectural standards
- ✅ **Operational tooling**: Version management, help system, predictable behavior

### Recommended Production Setup
1. Deploy using `./bin/deploy.sh` for proper installation
2. Ensure `~/.local/bin/odx` is in system PATH
3. Run `./bin/uat.sh` to verify all features work in your environment
4. Monitor storage location (`~/.local/share/odx/prontodb/`) for disk usage

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- Built with [RSB Framework](https://github.com/oodx/rsb-framework) for consistent CLI patterns
- Uses SQLite for reliable, embedded storage
- Follows XDG Base Directory specification for system integration
- Comprehensive documentation and knowledge management by China agent

---

**Ready for production use** - Deploy with confidence! 🚀

For additional help: `prontodb help`  
For feature demonstration: `./bin/uat.sh`  
For comprehensive project context: Review `.eggs/` documentation