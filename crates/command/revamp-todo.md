# Command Caching Revamp Plan

This document outlines the plan to refactor the command caching mechanism to support access timestamp tracking, in-memory caching, and better separation of concerns.

## Goals
1.  **Access Tracking**: Track when cache entries are accessed (not just created) to identify frequently used vs. stale entries.
2.  **In-Memory Cache**: Introduce a fast in-memory cache to reduce disk I/O for repeated commands within the same session.
3.  **Refactoring**: Split the monolithic `command.rs` and introduce a dedicated `CommandCache` and `CommandInput` structure.

## Proposed Changes

### 1. New File: `crates/command/src/command_input.rs`

We will introduce a `CommandInput` struct that captures all inputs determining the output of a command. This struct will serve as the key for the in-memory cache.

```rust
use std::collections::BTreeMap;
use std::ffi::OsString;
use std::path::PathBuf;
use bstr::BString;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct CommandInput {
    pub program: String, // Derived from CommandKind
    pub args: Vec<OsString>, // Resolved arguments
    pub env: BTreeMap<String, String>, // Environment variables
    pub run_dir: Option<PathBuf>,
    pub stdin: Option<String>,
    pub adjacent_files: BTreeMap<PathBuf, BString>, // Adjacent files
}

impl CommandInput {
    // Constructor that takes CommandBuilder fields and normalizes them (resolves args)
}
```

**Notes:**
-   `env` and `adjacent_files` use `BTreeMap` to ensure a consistent sort order for stable hashing.
-   `CommandArgument`s will be resolved to `OsString`s during `CommandInput` construction.

### 2. New File: `crates/command/src/command_cache.rs`

This file will contain the `CommandCache` struct responsible for managing both in-memory and on-disk caching.

```rust
use crate::{CommandInput, CommandOutput};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use std::time::Duration;

// Newtype for type safety
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CacheKey(pub PathBuf);

pub struct CommandCache {
    memory: Arc<Mutex<HashMap<CommandInput, CommandOutput>>>,
}

impl CommandCache {
    pub fn new() -> Self { ... }

    pub async fn get(&self, input: &CommandInput, key: &CacheKey, valid_for: Duration) -> Option<CommandOutput> {
        // 1. Check memory
        // 2. If miss, check disk (using key.0)
        //    - Validate disk cache against input (hash check)
        //    - If hit:
        //      - Update timestamps.txt (debounced)
        //      - Populate memory
        //      - Return
        // 3. Return None
    }

    pub async fn put(&self, input: CommandInput, output: CommandOutput, key: &CacheKey) -> Result<()> {
        // 1. Update memory
        // 2. Update disk (using key.0)
        //    - Write stdout, stderr, status
        //    - Write input hash for validation (input_hash.txt)
        //    - Write human-readable summary (context.txt)
        //    - Write initial timestamps.txt
    }
    
    async fn update_disk_timestamp(key: &CacheKey) -> Result<()> {
        // Append current RFC3339 timestamp to timestamps.txt
        // Debounce: Only write if last timestamp > X minutes ago
    }
}
```

**Note on `key`**: This parameter is required. If `CacheBehaviour::None` is used, the `CommandCache` should not be invoked. All cached commands must have a corresponding cache key.

### 3. Modifications to `crates/command/src/command.rs`

-   Remove `get_cached_output`, `write_output`, and `write_failure` (move logic to `CommandCache` or adapt).
-   `CommandBuilder` will now instantiate a `CommandInput`.
-   `run_raw` will interact with a global or shared `CommandCache` instance.

### 4. Modifications to `crates/command/src/lib.rs`

-   Expose `CommandInput` and `CommandCache`.
-   Define a static instance of `CommandCache` (e.g., `pub static GLOBAL_CACHE: Lazy<CommandCache>`).

## Implementation Details

### Timestamp Logic
-   **File**: Rename `timestamp.txt` (creation time) to `timestamps.txt` (append-only log).
-   **Format**: One RFC3339 timestamp per line.
-   **Creation**: The first line is the creation time.
-   **Access**: Subsequent lines are access times.
-   **Debouncing**: To prevent excessive disk writes, we will only append a new timestamp if the last entry in `timestamps.txt` is older than a threshold (e.g., 15 minutes).

### Cache Validation
-   The current `command.rs` validates `context.txt` (summary) and adjacent files.
-   The new `CommandInput` includes `env` and `run_dir`.
-   **Decision**: Should on-disk cache validation be stricter (check env/run_dir)?
    -   [x] **Yes**, `CommandInput` captures the true state. If `env` changes, it's a different command context.
    -   **Security & Transparency**:
        -   **Validation**: We will store a **hash** of the full `CommandInput` (including env vars) in `input_hash.txt`. This allows strict validation without storing sensitive values in plain text.
        -   **Debugging**: We will continue to store a **human-readable summary** in `context.txt` (e.g., the command string). This ensures the cache is not opaque to users, while the hash ensures integrity. Sensitive environment variables should be excluded from the summary but included in the hash.


### Transition Plan
1.  Create `CommandInput` and ensure it captures all necessary state.
2.  Implement `CommandCache` with in-memory logic.
3.  Port on-disk logic to `CommandCache`, implementing the `timestamps.txt` change.
4.  Update `CommandBuilder` to use the new system.

## Todo List

- [ ] Create `crates/command/src/command_input.rs`
- [ ] Create `crates/command/src/command_cache.rs`
- [ ] Implement `CommandInput` struct and construction logic
- [ ] Implement `CommandCache` in-memory logic
- [ ] Implement `CommandCache` on-disk logic (read/write/validate)
- [ ] Implement `timestamps.txt` logic with debouncing
- [ ] Refactor `CommandBuilder` in `command.rs` to use `CommandCache`
- [ ] Update `lib.rs` exports
