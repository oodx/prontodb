# ProntoDB Cursor Concept

## Overview
The cursor system provides persistent database selection caching, creating lightweight pseudo-sessions for users. This allows setting a database selection once and having all subsequent commands automatically use that selection.

## Design Philosophy
- **Persistent Selection**: Set database selection once, use across multiple commands
- **User-Specific Contexts**: Support multiple users with individual cursor selections
- **Flexible Invocation**: Multiple ways to set and use cursors
- **Global Storage**: Cursor cache files stored in global config directory

## Cursor Cache Storage

**Location**: `~/.local/etc/prontodb/`
- `cursor` - Global cursor selection (default user)
- `cursor_<username>` - User-specific cursor selection (e.g., `cursor_alice`, `cursor_bob`)

**File Format**: Simple text files containing just the database name
```
staging
```

## Invocation Patterns

### 1. Direct Cursor Command
Set cursor selection without executing other operations:
```bash
prontodb cursor staging               # Sets global cursor → staging database
prontodb cursor prod --user alice     # Sets alice's cursor → prod database
```

### 2. Cursor Flag on Any Command
Set cursor selection while executing a command:
```bash
prontodb set key1 "value" --cursor staging          # Sets global cursor + executes set
prontodb get key1 --cursor prod --user alice        # Sets alice's cursor + executes get
prontodb backup --cursor test --user bob            # Sets bob's cursor + creates backup
```

### 3. Optional Noop Command
Set cursor selection without any operation (useful for scripting):
```bash
prontodb noop --cursor staging                      # Just sets global cursor
prontodb noop --cursor prod --user alice            # Just sets alice's cursor
```

## Auto-Selection Logic

When no explicit database selection is provided:

1. **With --user flag**: Read `cursor_<username>` file
   ```bash
   prontodb --user alice get key1    # Reads cursor_alice → uses alice's selected database
   ```

2. **Without --user flag**: Read `cursor` file (global selection)
   ```bash
   prontodb get key1                 # Reads cursor → uses globally selected database
   ```

3. **Explicit flags override**: `--cursor` or `--database` flags take precedence
   ```bash
   prontodb --database test get key1 # Direct database access, ignores cursor cache
   ```

## User Workflow Examples

### Setting Up Cursors
```bash
# Alice sets her cursor to prod database
prontodb cursor prod --user alice

# Bob sets his cursor to staging database  
prontodb cursor staging --user bob

# Set global cursor to test database
prontodb cursor test
```

### Using Cursor Selections
```bash
# Alice's operations automatically use prod database
prontodb --user alice set project.key "alice-value"
prontodb --user alice get project.key

# Bob's operations automatically use staging database
prontodb --user bob set project.key "bob-value"
prontodb --user bob get project.key

# Global operations use test database
prontodb set global.key "test-value"
prontodb get global.key
```

### Overriding Cursor Selection
```bash
# Alice temporarily uses different database without changing her cursor
prontodb --user alice --database dev get temp.key

# Direct database access without cursor involvement
prontodb --database backup list-keys
```

## Relationship to --database Flag

**--cursor**: Persistent selection with caching
- Updates cursor cache file when used
- Provides auto-selection for subsequent commands
- Works with --user for user-specific contexts

**--database**: Ephemeral direct access  
- No caching or persistence
- Direct database selection for single command
- No --user interaction needed (explicit selection)

## Implementation Requirements

1. **Cursor Cache Management**
   - Create/update cursor cache files in `~/.local/etc/prontodb/`
   - Read cursor cache files for auto-selection
   - Handle missing cache files gracefully

2. **Command Integration**
   - Add `cursor` command to dispatcher
   - Support `--cursor` flag on all commands
   - Implement auto-selection logic in command context parsing

3. **User Context Support**
   - Support `--user` flag with cursor operations
   - Manage separate cursor files per user
   - Default to global cursor when no user specified

4. **Backward Compatibility**
   - Don't break existing `--database` functionality
   - Maintain explicit flag override behavior
   - Support systems without cursor cache files

## Benefits

1. **Improved UX**: Set database context once, use across session
2. **Multi-User Support**: Each user maintains separate database context
3. **Flexibility**: Multiple ways to set and use cursors
4. **Scriptability**: Easy automation with noop command
5. **Lightweight**: Simple text files, no complex state management

## Future Enhancements

1. **Local Cursor Override**: Support cursor files in working directory
2. **Cursor Hierarchy**: Local → global → default fallback
3. **Cursor Metadata**: Store additional context (project, namespace defaults)
4. **Cursor Management**: List, delete, rename cursor selections