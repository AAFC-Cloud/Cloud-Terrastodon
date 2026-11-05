#![allow(dead_code)]

use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_execution::AnonymousAuthenticationProvider;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

pub fn executor(scope: CancellationScope) -> Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::temporary()?,
        Arc::new(AnonymousAuthenticationProvider),
        scope,
    ))
}

pub fn executor_in(scope: CancellationScope, root: &Path) -> Result<CloudTerrastodonExecutor> {
    Ok(CloudTerrastodonExecutor::new(
        WorkCache::new(root)?,
        Arc::new(AnonymousAuthenticationProvider),
        scope,
    ))
}

pub fn key(valid_for: Duration) -> CacheKey {
    CacheKey {
        scope: CacheScope::try_from_components(["synthetic", "process"]).unwrap(),
        valid_for,
    }
}

/// Inspect only the explicit synthetic cache root supplied by the test.
pub fn files_named(root: &Path, name: &str) -> Result<Vec<PathBuf>> {
    if !root.exists() {
        return Ok(Vec::new());
    }
    let mut directories = vec![root.to_path_buf()];
    let mut matches = Vec::new();
    while let Some(directory) = directories.pop() {
        for entry in std::fs::read_dir(directory)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                directories.push(entry.path());
            } else if entry.file_name() == name {
                matches.push(entry.path());
            }
        }
    }
    Ok(matches)
}
