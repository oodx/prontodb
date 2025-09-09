# ProntoDB Actual Syntax & Limitations

## Current Implementation Limitations

### 1. Dot Addressing - ONLY 3 Levels Supported
```bash
# ✅ WORKS: 
prontodb set myapp.config.debug "true"        # project.namespace.key
prontodb set config.debug "true"              # namespace.key (uses default project)
prontodb set debug "true"                     # key (uses default.default)

# ❌ FAILS: More than 3 dots
prontodb set myapp.prod.db.host "prod.db.com" # ERROR: 4 parts not supported
# Workaround: Use underscores in the key
prontodb set myapp.prod.db_host "prod.db.com" # This works
```

### 2. Keys Cannot Contain Dots
The key (3rd part) cannot contain the delimiter (dot):
```bash
# ❌ FAILS:
prontodb set myapp.config.db.host "value"  # ERROR: key "db.host" contains dot

# ✅ WORKS:
prontodb set myapp.config.db_host "value"  # Use underscore instead
```

### 3. Keys/Scan Commands Need Explicit Flags
The `keys` and `scan` commands don't parse dot addresses:
```bash
# ❌ WRONG (from UAT):
prontodb keys myapp.prod
prontodb scan myapp.prod

# ✅ CORRECT:
prontodb keys -p myapp -n prod
prontodb scan -p myapp -n prod
```

### 4. Create-Cache Syntax
```bash
# ❌ WRONG (from UAT):
prontodb create-cache sessions.cache 3600

# ✅ CORRECT:
prontodb create-cache sessions.cache timeout=3600
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
# Create TTL namespace (note the timeout= syntax)
prontodb create-cache myapp.sessions timeout=3600

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