# 🦊 RSB VIOLATION YAP
**Date**: 2025-09-09
**Target**: /home/xnull/repos/code/rust/oodx/prontodb/src/cursor.rs
**Violation Type**: Missing RSB Prelude Import

## VIOLATION DETECTED 🚨
The cursor.rs module is completely missing the required RSB prelude import:

```rust
// cursor.rs - Line 1-11 - NO RSB IMPORT FOUND
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::xdg::XdgPaths;
```

## CANONICAL RSB PATTERN 📚
Every RSB-compliant module MUST start with the prelude import:

```rust
use rsb::prelude::*;
```

## CORRECTIVE ACTION ⚡
Add RSB prelude import as the FIRST import:

```rust
// Cursor Management for ProntoDB
// Provides database context switching and multi-user support following RSB patterns

use rsb::prelude::*;  // RSB REQUIRED FIRST IMPORT

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::xdg::XdgPaths;
```

## REFERENCE 📖
RSB Architecture Guide: "Every module MUST import 'use rsb::prelude::*'"