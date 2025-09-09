# ProntoDB CORRECTED API - Consistent Dot Addressing

## FIXED: Consistent Dot Addressing Support

### 1. Core CRUD Operations - Full Dot Addressing ✅
```bash
# All core operations support dot addressing:
prontodb set myapp.config.debug "true"        # project.namespace.key
prontodb get myapp.config.debug               # ✅ WORKS
prontodb del myapp.config.debug               # ✅ WORKS

# Shorter forms:
prontodb set config.debug "true"              # namespace.key (uses default project)
prontodb set debug "true"                     # key (uses default.default)
```

### 2. Discovery Operations - NOW WITH DOT ADDRESSING ✅
```bash
# ✅ FIXED: All discovery commands now support dot addressing
prontodb keys myapp.config                    # List keys in myapp.config namespace
prontodb keys myapp.config prefix             # List keys with prefix
prontodb scan myapp.config                    # Scan key-value pairs
prontodb scan myapp.config prefix             # Scan with prefix

# Also still works with explicit flags:
prontodb keys -p myapp -n config
prontodb scan -p myapp -n config
```

### 3. Create-Cache - FIXED SYNTAX ✅
```bash
# ✅ CORRECTED: Simple positional arguments
prontodb create-cache myapp.sessions 3600     # project.namespace ttl_seconds

# Old timeout= syntax removed for consistency
```

### 4. Addressing Limitations (Unchanged)
```bash
# Still limited to 3 levels maximum:
# ❌ FAILS: More than 3 dots
prontodb set myapp.prod.db.host "prod.db.com" # ERROR: 4 parts not supported
# ✅ WORKAROUND: Use underscores in the key
prontodb set myapp.prod.db_host "prod.db.com" # This works

# Keys still cannot contain dots (3rd part only):
# ❌ FAILS: prontodb set myapp.config.db.host "value"  
# ✅ WORKS: prontodb set myapp.config.db_host "value"
```

## Working Examples

### Basic Operations
```bash
# Set/Get with 3-level addressing
prontodb set myapp.config.debug "true"
prontodb get myapp.config.debug
prontodb del myapp.config.debug

# With context suffix
prontodb set myapp.config.api_key__prod "secret123"
prontodb get myapp.config.api_key__prod
```

### Discovery Operations
```bash
# List all projects
prontodb projects

# List namespaces (requires -p flag)
prontodb namespaces -p myapp

# List keys (requires both -p and -n flags)
prontodb keys -p myapp -n config

# Scan key-value pairs (requires both -p and -n flags)
prontodb scan -p myapp -n config
```

### TTL/Cache Operations
```bash
# Create TTL namespace (positional timeout argument)
prontodb create-cache myapp.sessions 3600

# Then use it normally
prontodb set myapp.sessions.user123 "active"
```

### Cursor Operations
```bash
# Set a cursor
prontodb cursor set staging /path/to/staging.db

# Use cursor with global flag
prontodb --cursor staging set app.config.debug "true"
prontodb --cursor staging get app.config.debug
```

## Recommended Naming Conventions

Since we're limited to 3 levels and keys can't contain dots:

```bash
# For hierarchical data, use underscores in keys:
prontodb set myapp.prod.db_host "prod.db.com"
prontodb set myapp.prod.db_port "5432"
prontodb set myapp.prod.db_user "admin"

# Or use the namespace for grouping:
prontodb set myapp.database.host "prod.db.com"
prontodb set myapp.database.port "5432"
prontodb set myapp.database.user "admin"

# For environment-specific values, use context suffix:
prontodb set myapp.config.api_url__dev "http://localhost:3000"
prontodb set myapp.config.api_url__prod "https://api.example.com"
```

## Future Improvements Needed

1. **Flexible dot addressing**: Support arbitrary depth (myapp.prod.db.config.host)
2. **Smart command parsing**: Make `keys` and `scan` parse dot addresses
3. **Better error messages**: Explain the 3-level limitation when user tries 4+ levels
4. **Consistent syntax**: Make all commands support dot addressing uniformly