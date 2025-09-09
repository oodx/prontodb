# ProntoDB

**Production-Ready Infrastructure Key-Value Store for Multi-Agent Workflows**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Tests](https://img.shields.io/badge/tests-passing-brightgreen.svg)](#testing)
[![RSB](https://img.shields.io/badge/RSB-compliant-blue.svg)](https://github.com/oodx/rsb-framework)
[![Production](https://img.shields.io/badge/status-production--ready-success.svg)](#production-deployment)

> **A blazingly fast, single-binary key-value store designed for coordinated AI agent systems and infrastructure automation.**

ProntoDB delivers enterprise-grade key-value storage with **multi-database cursors**, **multi-user isolation**, **TTL cache namespaces**, and **complete lifecycle management** — all in a single, dependency-free binary optimized for production deployment.

---

## 🚀 **Quick Start**

### Installation
```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
./bin/deploy.sh
```

The deployment script handles everything:
- Builds optimized release binary
- Installs to XDG-compliant paths
- Creates proper symlinks
- Runs verification tests
- Ready for production use

### Immediate Usage
```bash
# Store configuration with dot addressing
prontodb set myapp.config.environment "production"
prontodb set myapp.secrets.api_key "secret123"

# Retrieve values
prontodb get myapp.config.environment
# Output: production

# Multi-agent isolation
prontodb --user agent1 --cursor prod set status "active"
prontodb --user agent2 --cursor dev set status "testing"

# Backup entire system
prontodb backup --output ./backups
```

---

## 🔥 **Core Features**

### **Multi-Agent Infrastructure Ready**
- **🎯 Multi-User Isolation**: Complete separation with `--user` flags
- **🗂️ Multi-Database Cursors**: Context switching with `--cursor` for different environments
- **🔄 Combined Operations**: `--user agent1 --cursor prod` for precise targeting
- **⚡ Concurrent Safe**: Multiple agents operate simultaneously without conflicts

### **Production-Grade Storage**
- **📍 Dot Addressing**: Intuitive `project.namespace.key` syntax
- **⏰ TTL Cache Namespaces**: Automatic expiry for session/cache data
- **📦 Single Binary**: Zero external dependencies, easy deployment
- **🛡️ XDG Compliance**: Follows system standards for data storage
- **🔒 Data Integrity**: SQLite backend with WAL journaling

### **Complete Lifecycle Management**
- **📥 Install/Uninstall**: Automated system integration
- **💾 Backup/Restore**: Full system state preservation
- **🔧 Deploy Scripts**: Production-ready automation
- **📊 UAT Testing**: Comprehensive acceptance validation

---

## 🏗️ **Multi-Agent Workflows**

### **Agent Isolation Example**
```bash
# Agent coordination scenario
prontodb --user orchestrator --cursor main set workflow.status "initializing"
prontodb --user worker1 --cursor tasks set current_job "processing_batch_1" 
prontodb --user worker2 --cursor tasks set current_job "processing_batch_2"
prontodb --user monitor --cursor logs set last_check "$(date)"

# Each agent has isolated view
prontodb --user worker1 --cursor tasks get current_job
# Only sees: processing_batch_1

# Orchestrator can coordinate via main database
prontodb --user orchestrator --cursor main get workflow.status
# Output: initializing
```

### **Context Switching with Cursors**
```bash
# Setup multiple environments
prontodb cursor set staging ./staging.db
prontodb cursor set production ./prod.db
prontodb cursor set development ./dev.db

# Deploy configuration to specific environments
prontodb --cursor staging set app.config.debug "true"
prontodb --cursor production set app.config.debug "false"
prontodb --cursor development set app.config.debug "verbose"

# List available cursors
prontodb cursor list
# Shows: staging, production, development
```

### **TTL Cache for Temporary Data**
```bash
# Create session cache with 1-hour expiry
prontodb create-cache sessions.cache timeout=3600

# Store temporary agent state
prontodb set sessions.cache.agent_123 "active_session_data"
prontodb set sessions.cache.agent_456 "processing_task_xyz"

# Data automatically expires after timeout
```

---

## 📊 **Complete Command Reference**

### **Core Key-Value Operations**
```bash
prontodb set <key> <value>        # Store value
prontodb get <key>                # Retrieve value (exit 2 if not found)
prontodb del <key>                # Delete value
prontodb keys [prefix]            # List keys with optional prefix
prontodb scan [prefix]            # List key=value pairs with optional prefix
```

### **Multi-Database Cursor Management**
```bash
prontodb cursor set <name> <path>   # Create/update cursor to database path
prontodb cursor list                # List all cursors for current user
prontodb cursor active              # Show active cursor
prontodb cursor delete <name>       # Remove cursor
```

### **Discovery & Navigation**
```bash
prontodb projects                   # List all projects
prontodb namespaces -p <project>    # List namespaces in project  
prontodb keys -p <project> -n <ns>  # List keys in specific namespace
prontodb scan -p <project> -n <ns>  # List key-value pairs in namespace
```

### **TTL Cache Management**
```bash
prontodb create-cache <project.namespace> timeout=<seconds>
```

### **Lifecycle Commands**
```bash
prontodb install --target <path>           # Install binary to target
prontodb backup --output <directory>       # Create system backup
prontodb backup --restore <backup-file>    # Restore from backup
prontodb backup --list                     # List available backups
prontodb uninstall                         # Clean system removal
```

### **Global Flags**
```bash
--cursor <name>            # Use named cursor for database context
--user <user>              # Use specific user context (default: 'default')
--ns-delim <char>          # Override namespace delimiter (default: '.')
```

---

## 🏭 **Production Deployment**

### **Infrastructure Requirements**
- **Minimum**: Linux/macOS/Windows with Rust 1.70+
- **Storage**: XDG-compliant directories (`~/.local/share/odx/prontodb/`)
- **Permissions**: User-level access (no root required)
- **Dependencies**: None (single static binary)

### **Deployment Process**
```bash
# 1. Clone and deploy
git clone https://github.com/oodx/prontodb.git
cd prontodb
./bin/deploy.sh

# 2. Verify installation
prontodb version
prontodb help

# 3. Run comprehensive tests
./test.sh

# 4. Setup PATH (if needed)
echo 'export PATH="$HOME/.local/bin/odx:$PATH"' >> ~/.bashrc
```

### **Production Validation**
The included UAT suite validates all production scenarios:
```bash
./test.sh  # Comprehensive test suite covering:
           # - Basic CRUD operations
           # - Multi-user isolation
           # - Cursor management
           # - TTL cache functionality  
           # - Backup/restore cycles
           # - Error handling
           # - Performance benchmarks
```

### **Monitoring & Maintenance**
```bash
# Monitor storage usage
du -sh ~/.local/share/odx/prontodb/

# Create system backups
prontodb backup --output /backup/location

# Health check
prontodb cursor list    # Should show expected cursors
prontodb projects       # Should show expected projects
```

---

## 🧪 **Testing & Quality Assurance**

### **Automated Test Suite**
```bash
cargo test  # Runs complete test suite
```

**Test Coverage:**
- **Unit Tests**: Core functionality validation
- **Integration Tests**: End-to-end workflows
- **Sanity Tests**: Behavioral verification
- **Performance Tests**: Speed and reliability
- **UAT Tests**: Production scenario validation

### **User Acceptance Testing**
Experience all features with the comprehensive UAT demonstration:
```bash
./test.sh  # Interactive walkthrough covering:
           # - Multi-agent coordination
           # - Cursor context switching
           # - TTL cache behavior
           # - Backup/restore cycles
           # - Error handling
           # - Production readiness
```

### **Quality Metrics**
- ✅ **100% Test Pass Rate**: All automated tests pass
- ✅ **Zero Critical Issues**: Production-ready stability
- ✅ **Performance Verified**: <10s for 100 operations
- ✅ **Memory Safe**: Rust's memory safety guarantees
- ✅ **Concurrent Safe**: Multi-agent operation validated

---

## 🏗️ **Architecture & Design**

### **RSB Framework Integration**
ProntoDB is built with [Rebel String-Biased (RSB) framework](https://github.com/oodx/rsb-framework):
- **String-biased APIs**: Optimized for CLI operations
- **Standard Entry Patterns**: `args!()` and `pre_dispatch!()` integration
- **Lifecycle Commands**: Full install/backup/uninstall support
- **Team Standards**: Documented patterns for organizational learning

### **Storage Architecture**
- **Backend**: SQLite with WAL (Write-Ahead Logging) mode
- **Multi-Database**: Each cursor points to separate SQLite file
- **User Isolation**: Separate databases per user context
- **XDG Compliance**: Standard system paths with override support
- **Performance**: Optimized for fast key-value operations

### **Exit Codes & Standards**
```bash
0 - Success
1 - Error (invalid syntax, storage failure, etc.)
2 - Key not found or expired (MISS)
```

### **Project Structure**
```
src/
├── main.rs           # RSB-integrated entry point
├── dispatcher.rs     # Multi-command routing with cursor/user support
├── api.rs           # Centralized API layer for all operations
├── storage.rs       # SQLite storage with multi-database support
├── addressing.rs    # Namespace addressing and validation
└── xdg.rs          # XDG Base Directory compliance

bin/
├── deploy.sh        # Production deployment automation  
└── test.sh         # Comprehensive UAT test suite
```

---

## 🔧 **Development**

### **Building from Source**
```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
cargo build --release
cargo test
./test.sh  # Run UAT suite
```

### **Build Configurations**
```bash
# Default build (bundled SQLite)
cargo build --release

# System SQLite (smaller binary)  
cargo build --release --no-default-features --features json

# With optional features
cargo build --release --features compression-zstd
cargo build --release --features encryption-aes
```

### **Development Workflow**
1. **Documentation**: Review `.eggs/` summaries and `docs/RSB_USAGE.md`
2. **Testing**: Run `cargo test` and `./test.sh` for validation
3. **Standards**: Follow RSB patterns and architectural guidelines
4. **Quality**: Ensure all tests pass before committing

---

## 🤝 **Contributing**

### **Getting Started**
1. **Study the Codebase**: Review comprehensive documentation in `.eggs/` directory
2. **Understand RSB**: Read `docs/RSB_USAGE.md` for framework integration patterns
3. **Run Tests**: Execute full test suite to understand expected behavior
4. **Review Architecture**: Study modular design and separation of concerns

### **Development Guidelines**
- Follow existing RSB integration patterns
- Maintain comprehensive test coverage
- Document new features thoroughly  
- Ensure backward compatibility
- Run UAT suite for end-to-end validation

### **Feature Development Process**
1. Review current roadmap in `ROADMAP.md`
2. Implement following established patterns
3. Add comprehensive tests
4. Update documentation
5. Validate with UAT suite

---

## 🎯 **Use Cases**

### **AI Agent Coordination**
- **Multi-agent workflows**: Isolated contexts per agent
- **State coordination**: Shared configuration management
- **Session management**: TTL cache for temporary state
- **Environment management**: Cursor-based context switching

### **Infrastructure Automation**
- **Configuration management**: Hierarchical namespace organization
- **Service coordination**: Multi-database environment isolation
- **Deployment pipelines**: Backup/restore for rollback capability
- **Monitoring systems**: Fast key-value lookup for metrics

### **Development Workflows**
- **Environment configuration**: Per-environment database contexts
- **Feature flags**: Dynamic configuration management
- **Testing isolation**: Separate test data per context
- **CI/CD integration**: Automated backup/restore in pipelines

---

## 📝 **License**

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 **Acknowledgments**

- **RSB Framework**: [github.com/oodx/rsb-framework](https://github.com/oodx/rsb-framework) for consistent CLI patterns
- **SQLite**: Reliable, embedded database engine
- **XDG Standards**: System integration best practices
- **China Agent**: Comprehensive documentation and knowledge management system

---

## 🚀 **Ready for Production**

**ProntoDB is production-ready with complete feature set, comprehensive testing, and enterprise-grade reliability.**

### **Deployment Checklist**
- ✅ **Single Binary**: Zero external dependencies
- ✅ **Complete Lifecycle**: Install/backup/uninstall automation
- ✅ **Multi-Agent Ready**: User isolation and cursor management  
- ✅ **Production Tested**: Comprehensive UAT validation
- ✅ **Standards Compliant**: RSB framework and XDG paths
- ✅ **Performance Verified**: Fast, reliable operations
- ✅ **Documentation Complete**: Comprehensive guides and examples

### **Get Started Now**
```bash
git clone https://github.com/oodx/prontodb.git
cd prontodb
./bin/deploy.sh  # One command to production readiness
```

**For help**: `prontodb help`  
**For testing**: `./test.sh`  
**For documentation**: Review `.eggs/` directory

---

**Deploy with confidence. Scale with ease. ProntoDB delivers.** ⚡