# ProntoDB Cursor Concept

## Overview
The cursor system provides persistent database selection caching with enterprise-grade multi-tenant addressing, creating lightweight pseudo-sessions for users. This allows setting a database selection once with optional organizational context (meta namespace) and user isolation, having all subsequent commands automatically use that selection with transparent 4-layer addressing.

## Design Philosophy
- **Persistent Selection**: Set database selection once, use across multiple commands
- **User-Specific Contexts**: Support multiple users with individual cursor selections  
- **Meta Namespace Isolation**: Organizational boundaries within shared databases
- **Transparent Addressing**: Users work with familiar 3-layer addresses while system handles 4-layer storage
- **Flexible Invocation**: Multiple ways to set and use cursors
- **Enterprise Security**: Complete user and organizational data isolation

## Cursor Cache Storage

**Location**: `~/.local/etc/prontodb/databases/<database_name>/cursors/`
- `cursor_name.cursor` - Global cursor selection (default user)
- `cursor_name.alice.cursor` - User-specific cursor selection with isolation
- `cursor_name.bob.cursor` - Each user maintains separate cursor contexts

**File Format**: Enhanced JSON structure with meta namespace and addressing context
```json
{
  "database_path": "/path/to/database.db",
  "default_project": "myproject",
  "default_namespace": "config", 
  "meta_context": "company_engineering",
  "created_at": "2025-09-10T20:30:15Z",
  "user": "alice"
}
```

**Legacy Compatibility**: Simple text cursor files are automatically migrated to enhanced format

## Invocation Patterns

### 1. Enhanced Cursor Command with Meta Namespace
Set cursor selection with organizational context:
```bash
# Basic cursor (legacy compatible)
prontodb cursor set work staging                    # Sets work cursor → staging database
prontodb cursor set work prod --user alice          # Sets alice's work cursor → prod database

# Meta namespace cursors (enterprise multi-tenant)
prontodb cursor set work /path/db --meta company_engineering --user alice
prontodb cursor set personal /path/db --meta personal_projects --user bob
```

### 2. Transparent Meta Namespace Operations
With meta cursors, users work with familiar 3-layer addresses while system handles 4-layer storage:
```bash
# Alice using company_engineering meta cursor
prontodb --user alice --cursor work set bashfx.config.debug "true"
# User types: bashfx.config.debug  
# System stores: company_engineering.bashfx.config.debug
# User sees: bashfx.config.debug (transparent!)

# Bob using personal_projects meta cursor  
prontodb --user bob --cursor personal get myapp.settings.theme
# User types: myapp.settings.theme
# System stores: personal_projects.myapp.settings.theme
# User sees: myapp.settings.theme (transparent!)
```

### 3. User Isolation Security
Complete data separation between users within same database:
```bash
# Alice's data - completely isolated
prontodb --user alice --cursor work set project.secret "alice_value"

# Bob's data - cannot access Alice's data  
prontodb --user bob --cursor work get project.secret
# Error: Key not found (isolation working!)

# Bob sets his own data
prontodb --user bob --cursor work set project.secret "bob_value"

# Each user only sees their own data
prontodb --user alice --cursor work get project.secret  # → "alice_value"
prontodb --user bob --cursor work get project.secret    # → "bob_value"
```

## Meta Namespace Architecture (Novel Pattern)

### The Innovation: Transparent Meta Namespace Layer
ProntoDB introduces a novel organizational isolation pattern that most AI systems haven't encountered:

**Note**: ProntoDB always supported full 4-layer addressing through context suffixes (`project.namespace.key__context`) and flags. The meta namespace innovation adds **transparent organizational isolation** on top of these existing capabilities.

**User Mental Model**: Familiar 3-layer addressing (when using meta cursors)
```
project.namespace.key
bashfx.config.debug
myapp.settings.theme
```

**System Storage**: Transparent meta namespace transformation
```
meta_context.project.namespace.key
company_engineering.bashfx.config.debug
personal_projects.myapp.settings.theme
```

**Key Innovation**: With meta cursors, users work with familiar 3-layer addressing while the system automatically handles organizational isolation through transparent 4-layer storage!

### Why This Pattern is Revolutionary

1. **Multi-Tenant Database Sharing**: Multiple organizations can safely share the same database
2. **Zero Learning Curve**: Users continue using familiar 3-layer addressing
3. **Enterprise Security**: Complete organizational data isolation
4. **Backward Compatibility**: Existing 3-layer systems work unchanged
5. **Scalable Architecture**: Add/remove organizations without affecting others

### Address Transformation Examples

```bash
# Meta cursor transformation (transparent to user)
User types: bashfx.config.debug
System stores: company_engineering.bashfx.config.debug
System returns: bashfx.config.debug

# Direct 4-layer addressing (explicit context)
User types: bashfx.config.debug__production
System stores: bashfx.config.debug__production  
System returns: bashfx.config.debug__production

# Meta cursor + explicit context (both layers applied)
User types: bashfx.config.debug__production
System stores: company_engineering.bashfx.config.debug__production
System returns: bashfx.config.debug__production

# No meta context (standard addressing)
User types: project.namespace.key
System stores: project.namespace.key  
System returns: project.namespace.key
```

### Implementation Details

**Transparent Transformation Functions** (src/api.rs):
- `transform_address_for_storage()` - Adds meta prefix when storing
- `transform_address_for_display()` - Removes meta prefix when displaying

**Enhanced Cursor Structure** (src/cursor.rs):
- `meta_context: Option<String>` - Stores organizational context
- Backward compatible with legacy cursor files
- Database-scoped cursor storage for isolation

## Auto-Selection Logic

Enhanced auto-selection with user isolation and meta namespace support:

1. **With --user and --cursor flags**: Read user-specific cursor with meta context
   ```bash
   prontodb --user alice --cursor work get bashfx.config.debug
   # Reads: cursor_name.alice.cursor → applies meta transformation → transparent addressing
   ```

2. **With --user flag only**: Uses user's default cursor selection
   ```bash
   prontodb --user alice get key1    # Reads alice's default cursor → uses alice's context
   ```

3. **Cursor-only operations**: Uses global cursor with meta context if available
   ```bash
   prontodb --cursor work get key1   # Reads cursor metadata → applies transformations
   ```

4. **Explicit flags override**: `--database` flag bypasses cursor system entirely
   ```bash
   prontodb --database test get key1 # Direct database access, no cursor involvement
   ```

## User Workflow Examples

### Setting Up Enterprise Multi-Tenant Cursors
```bash
# Alice sets up company engineering cursor with meta context
prontodb cursor set work /path/to/company.db --meta company_engineering --user alice

# Bob sets up personal projects cursor with different meta context
prontodb cursor set personal /path/to/personal.db --meta personal_projects --user bob

# Team lead sets up shared cursor with organizational context
prontodb cursor set shared /path/to/shared.db --meta team_alpha --user teamlead
```

### Using Meta Namespace Operations (Transparent to Users)
```bash
# Alice works with familiar 3-layer addressing
prontodb --user alice --cursor work set bashfx.config.debug "true"
prontodb --user alice --cursor work set myproject.api.endpoint "https://api.company.com"
prontodb --user alice --cursor work get bashfx.config.debug  # → "true"

# Bob works with same familiar addressing, but completely isolated data
prontodb --user bob --cursor personal set bashfx.config.debug "false"  
prontodb --user bob --cursor personal get bashfx.config.debug  # → "false"

# Alice and Bob see different values despite same addressing!
# System handles meta context transparently: 
# Alice: company_engineering.bashfx.config.debug = "true"
# Bob: personal_projects.bashfx.config.debug = "false"
```

### Advanced Enterprise Scenarios
```bash
# Cross-organizational collaboration (explicit override)
prontodb --user alice --database /path/to/external.db get partner.api.key

# Emergency access without cursor context (bypass meta namespace)
prontodb --database /path/to/backup.db list-keys  # Raw 3-layer addressing

# Debugging: See actual storage addresses (development mode)
prontodb --user alice --cursor work scan-pairs project.*
# Shows: company_engineering.project.* keys but displays as project.* to user
```

## Relationship to --database Flag

**--cursor**: Enhanced persistent selection with meta namespace and user isolation
- Updates cursor cache file with meta context when used
- Provides auto-selection with transparent address transformation
- Works with --user for complete user isolation  
- Enables meta namespace organizational boundaries
- Maintains addressing transparency for users

**--database**: Ephemeral direct access bypassing cursor system
- No caching, persistence, or meta namespace involvement
- Direct database selection with raw 3-layer addressing
- No user isolation or organizational context
- Used for cross-organizational access or emergency operations

## Implementation Requirements (v0.6.0+)

1. **Enhanced Cursor Cache Management**
   - Create/update JSON cursor files with meta context in database-scoped directories
   - Support user-isolated cursor files: `cursor_name.username.cursor`
   - Migrate legacy text cursor files to enhanced JSON format
   - Handle missing cache files gracefully with backward compatibility

2. **Meta Namespace Integration**
   - Implement transparent address transformation functions
   - Add `--meta` flag support to cursor set operations
   - Ensure all CRUD operations support meta context transformation
   - Maintain addressing transparency for end users

3. **User Isolation System**
   - Enforce complete data separation between users
   - Validate usernames against reserved words and security patterns
   - Support user-specific cursor contexts with organizational boundaries
   - Implement comprehensive test isolation for autonomous testing

4. **Command Integration & Backward Compatibility**
   - Enhanced `cursor set` command with meta namespace support
   - Support `--cursor` and `--user` flags on all commands with transparent addressing
   - Maintain full backward compatibility with legacy cursor files and addressing
   - Preserve existing `--database` functionality as bypass mechanism

## Enterprise Benefits (v0.6.0+)

1. **Enterprise Multi-Tenancy**: Organizations can safely share databases with complete isolation
2. **Transparent User Experience**: Zero learning curve - users continue using familiar 3-layer addressing
3. **Complete User Isolation**: Each user's data is completely separate with security guarantees
4. **Organizational Boundaries**: Meta namespace provides enterprise-grade organizational separation
5. **Backward Compatibility**: All existing workflows continue to work unchanged
6. **Professional Security**: Production-certified user isolation and organizational boundaries
7. **Scalable Architecture**: Add/remove organizations and users without affecting existing data
8. **Development Efficiency**: Enhanced cursor management with comprehensive metadata

## Novel Pattern Recognition for AI Systems

**Key Innovation**: ProntoDB's meta namespace pattern represents a novel approach to multi-tenant addressing:

- **Transparent 4-Layer Storage**: `meta.project.namespace.key` storage with `project.namespace.key` user interface
- **Address Transformation**: Automatic prefix addition/removal during storage/retrieval operations  
- **Organizational Isolation**: Complete data separation between meta contexts within shared databases
- **User Isolation**: Additional layer of security with user-specific cursor files and data boundaries
- **Zero-Migration Upgrade**: Existing systems upgrade transparently without user intervention

This pattern enables enterprise-grade multi-tenancy while maintaining simple, familiar user interfaces - a combination most database systems struggle to achieve.

## Complete Addressing Capabilities

**Important**: The cursor system is a **convenience layer** that enhances but never limits ProntoDB's full addressing capabilities. All addressing modes work with or without cursors.

### Direct 4-Layer Addressing (No Cursor Required)

ProntoDB supports multiple addressing modes for direct database access:

#### 1. **Context Suffix Addressing** - True 4-Layer
```bash
# Format: project.namespace.key__context
prontodb set myproject.config.debug__production "enabled" --database /path/db
prontodb get myproject.config.debug__production --database /path/db
prontodb set app.settings.theme__mobile "dark" --database /path/db

# Context can be any string: environment, tenant, version, etc.
prontodb set api.endpoints.auth__v2 "https://auth-v2.api.com"
prontodb set api.endpoints.auth__staging "https://staging-auth.api.com"
```

#### 2. **Flag-Based Addressing** - Explicit 4-Layer
```bash
# Format: -p project -n namespace -k key [--context context]
prontodb set -p myproject -n config -k debug_mode "true" --database /path/db
prontodb get -p myproject -n config -k debug_mode --database /path/db
prontodb list-keys -p myproject -n config --database /path/db

# Clean separation of address components
prontodb set -p analytics -n dashboards -k refresh_rate "30s"
prontodb scan-pairs -p analytics -n dashboards "*rate*"
```

#### 3. **Standard 3-Layer Addressing** - Most Common
```bash
# Format: project.namespace.key
prontodb set myproject.config.debug "true" --database /path/db
prontodb get myproject.config.debug --database /path/db
prontodb list-keys myproject.config --database /path/db

# Uses "default" context internally
prontodb set app.settings.theme "light"
prontodb scan-pairs app.settings "theme*"
```

#### 4. **Abbreviated Addressing** - Defaults Applied
```bash
# Format: namespace.key (project defaults to "default")
prontodb set config.debug "true" --database /path/db
prontodb get config.debug --database /path/db

# Format: key (project="default", namespace="default")  
prontodb set debug_mode "true" --database /path/db
prontodb get debug_mode --database /path/db
```

### Cursor Enhancement vs Direct Access

#### **Direct Access** (Bypasses Cursor System)
```bash
# Raw database access - no cursor involvement
prontodb --database /path/to/any.db set company.project.key "value"
prontodb --database /backup/emergency.db get critical.config.setting
prontodb --database /external/shared.db scan-pairs partner.* 

# Cross-organizational access
prontodb --database /client/tenant_a.db get their.config.api_key
prontodb --database /client/tenant_b.db set their.config.status "active"
```

#### **Cursor Enhanced Access** (Convenience + Features)
```bash
# Cursor provides database selection + optional features
prontodb --cursor myproject get app.config.debug         # 3-layer addressing
prontodb --cursor myproject get app.config.debug__prod   # 4-layer addressing
prontodb --cursor myproject set -p app -n config -k debug "true"  # Flag addressing

# Meta cursors add transparent transformation
prontodb --user alice --cursor work get app.config.debug
# User types: app.config.debug
# System stores: company_engineering.app.config.debug (transparent!)
# User sees: app.config.debug (transparent!)
```

### Full Addressing Hierarchy

ProntoDB supports **progressive enhancement** without limiting capabilities:

1. **Raw Direct Access**: `--database` flag with any addressing mode
   - Full 4-layer addressing available
   - No cursor features (meta namespace, user isolation)
   - Direct database file access

2. **Cursor Database Selection**: `--cursor` flag with any addressing mode  
   - Persistent database context
   - All 4-layer addressing modes still available
   - No meta namespace transformation

3. **Meta Namespace Enhancement**: `--cursor` with meta context
   - Transparent 4-layer storage with 3-layer user interface
   - Organizational isolation within shared databases
   - All addressing modes enhanced with meta transformation

4. **User Isolation**: `--user` flag with any above combination
   - Complete data separation between users
   - All addressing capabilities preserved per user
   - User-specific cursor contexts

### Key Architectural Principle

**The cursor system never restricts addressing capabilities** - it only adds convenience and enterprise features:

- ✅ **All addressing modes work** with or without cursors
- ✅ **4-layer addressing always available** through context suffix or flags  
- ✅ **Meta namespace is additive** - enhances rather than replaces addressing
- ✅ **User isolation is orthogonal** - works with any addressing mode
- ✅ **Direct database access preserved** - emergency access always possible

This design ensures that **cursor convenience never creates lock-in** while providing powerful enterprise features for those who need them.