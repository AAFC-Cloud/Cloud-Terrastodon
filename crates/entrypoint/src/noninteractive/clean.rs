use super::CleanStats;
use cloud_terrastodon_cache::WorkCache;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_pathing::AppDir;
use eyre::Context;
use eyre::Result;
use std::io;
use std::path::Path;
use std::path::PathBuf;
use tokio::fs::remove_dir_all;
use tokio::fs::try_exists;
use tracing::warn;

pub async fn clean(cache: &WorkCache, cancellation: &CancellationScope) -> Result<CleanStats> {
    clean_dirs(AppDir::ok_to_clean(), cache, cancellation).await
}

pub async fn clean_dirs(
    dirs: impl IntoIterator<Item = AppDir>,
    cache: &WorkCache,
    cancellation: &CancellationScope,
) -> Result<CleanStats> {
    let dirs = dirs.into_iter().collect::<Vec<_>>();
    let invalidate_cache = dirs.iter().any(|dir| matches!(dir, AppDir::Commands));
    clean_paths(
        dirs.into_iter().map(|dir| dir.as_path_buf()),
        cache,
        &AppDir::Commands.join("work"),
        invalidate_cache,
        cancellation,
    )
    .await
}

async fn clean_paths(
    paths: impl IntoIterator<Item = PathBuf>,
    cache: &WorkCache,
    default_cache_root: &Path,
    invalidate_cache: bool,
    cancellation: &CancellationScope,
) -> Result<CleanStats> {
    if invalidate_cache {
        cache.invalidate_all(cancellation).await?;
        tracing::info!(
            cache_root = %cache.root().display(),
            "Invalidated configured cache results; retained immutable artifacts and cache coordination files"
        );
    }
    // A configured root may differ from the application's default. Preserve
    // both from recursive removal, but only invalidate the supplied owner;
    // acquiring another root while retaining this one can invert lock order.
    let protected_roots = [
        resolve_existing_ancestor(cache.root())?,
        resolve_existing_ancestor(default_cache_root)?,
    ];
    let mut total = CleanStats::default();

    for path in paths {
        cancellation.bail_if_cancelled()?;
        if !try_exists(&path).await? {
            continue;
        }
        let resolved = resolve_existing_ancestor(&path)?;
        if protected_roots
            .iter()
            .any(|root| root.starts_with(&resolved) || resolved.starts_with(root))
        {
            tracing::info!(
                directory = %path.display(),
                "Retaining generated directory because it overlaps a managed work cache"
            );
            continue;
        }

        let stats = match directory_stats(path.clone()).await {
            Ok(stats) => stats,
            Err(error) => {
                warn!(
                    directory = %path.display(),
                    %error,
                    "Unable to calculate cleanup size"
                );
                CleanStats::default()
            }
        };

        if let Err(error) = remove_dir_all(&path).await {
            warn!(directory = %path.display(), %error, "Unable to remove generated directory");
        } else {
            total.add_assign(stats);
        }
    }

    tracing::info!(
        "Removed {} files, {} total",
        total.files,
        format_bytes(total.bytes)
    );
    Ok(total)
}

// Resolve existing ancestors so aliases and platform path spelling cannot make
// an active cache look unrelated to a directory selected for recursive removal.
// Missing descendants still matter: another operation may create them later.
fn resolve_existing_ancestor(path: &Path) -> Result<PathBuf> {
    let mut existing = path;
    let mut missing = Vec::new();
    loop {
        match std::fs::canonicalize(existing) {
            Ok(mut resolved) => {
                for component in missing.into_iter().rev() {
                    resolved.push(component);
                }
                return Ok(resolved);
            }
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let Some(name) = existing.file_name() else {
                    return Err(error).wrap_err("resolving generated-directory cleanup boundary");
                };
                missing.push(name.to_owned());
                let Some(parent) = existing.parent() else {
                    return Err(error).wrap_err("resolving generated-directory cleanup boundary");
                };
                existing = parent;
            }
            Err(error) => {
                return Err(error).wrap_err("resolving generated-directory cleanup boundary");
            }
        }
    }
}

async fn directory_stats(path: PathBuf) -> io::Result<CleanStats> {
    tokio::task::spawn_blocking(move || collect_directory_stats(&path))
        .await
        .map_err(io::Error::other)?
}

fn collect_directory_stats(path: &Path) -> io::Result<CleanStats> {
    let mut stats = CleanStats::default();

    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            stats.add_assign(collect_directory_stats(&entry.path())?);
        } else {
            stats.files = stats.files.saturating_add(1);
            stats.bytes = stats.bytes.saturating_add(entry.metadata()?.len());
        }
    }

    Ok(stats)
}

fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{bytes}B")
    } else {
        format!("{value:.1}{}", UNITS[unit])
    }
}

#[cfg(test)]
mod tests {
    use super::clean_paths;
    use super::format_bytes;
    use cloud_terrastodon_cache::ArtifactObservation;
    use cloud_terrastodon_cache::ArtifactRetention;
    use cloud_terrastodon_cache::ArtifactSet;
    use cloud_terrastodon_cache::AuthPartition;
    use cloud_terrastodon_cache::CacheKey;
    use cloud_terrastodon_cache::CacheOperationOwner;
    use cloud_terrastodon_cache::CacheScope;
    use cloud_terrastodon_cache::ExecutorKind;
    use cloud_terrastodon_cache::NonEmptyString;
    use cloud_terrastodon_cache::WorkCache;
    use cloud_terrastodon_command::CancellationToken;
    use std::collections::BTreeMap;
    use std::fs;
    use std::time::Duration;

    struct SyntheticRead;

    #[tokio::test]
    async fn clean_invalidates_managed_cache_without_deleting_its_ancestor() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let temporary = tempfile::tempdir()?;
                let commands = temporary.path().join("commands");
                let cache = WorkCache::new(commands.join("work"))?;
                let entry = cache.artifacts::<String>(
                    ExecutorKind::Rest,
                    CacheKey {
                        scope: CacheScope::try_from_components(["synthetic", "cleanup"])?,
                        valid_for: Duration::from_secs(60),
                    },
                    CacheOperationOwner::of::<SyntheticRead>(),
                    AuthPartition::new("synthetic")?,
                    NonEmptyString::new("input")?,
                    ArtifactRetention::Full,
                )?;
                let stored = entry
                    .begin_attempt(BTreeMap::new(), &scope)
                    .await?
                    .unwrap()
                    .finish(
                        ArtifactSet {
                            payload: b"synthetic response".to_vec(),
                            diagnostics: BTreeMap::new(),
                        },
                        ArtifactObservation::now(),
                        &scope,
                    )
                    .await?;
                assert!(entry.publish(&stored, &scope).await?);
                let generated = temporary.path().join("imports");
                fs::create_dir(&generated)?;
                fs::write(generated.join("generated.txt"), b"generated")?;
                let stats = clean_paths(
                    [commands.clone(), generated.clone()],
                    &cache,
                    cache.root(),
                    true,
                    &scope,
                )
                .await?;
                assert!(entry.load(&scope).await?.is_none());
                assert!(stored.directory().join("payload.bin").is_file());
                assert!(commands.is_dir());
                assert!(!generated.exists());
                assert_eq!(stats.files, 1);
                assert_eq!(stats.bytes, 9);
                Ok(())
            })
            .await
    }

    #[test]
    fn formats_bytes_like_cargo() {
        assert_eq!(format_bytes(0), "0B");
        assert_eq!(format_bytes(1024), "1.0KiB");
        assert_eq!(format_bytes(1024 * 1024), "1.0MiB");
    }

    #[tokio::test]
    async fn cleaning_a_configured_cache_does_not_wait_for_or_remove_the_default_cache()
    -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(async |scope| {
                let temporary = tempfile::tempdir()?;
                let commands = temporary.path().join("commands");
                let default_cache = WorkCache::new(commands.join("work"))?;
                default_cache.acquire(&scope).await?;
                let retained = default_cache.root().join("retained-artifact");
                fs::write(&retained, b"synthetic retained content")?;
                let configured = temporary.path().join("configured");
                let cache = WorkCache::new(configured.join("work"))?;

                let stats = tokio::time::timeout(
                    Duration::from_secs(10),
                    clean_paths(
                        [commands.clone(), configured.clone()],
                        &cache,
                        default_cache.root(),
                        true,
                        &scope,
                    ),
                )
                .await??;
                assert_eq!(stats.files, 0);
                assert!(commands.is_dir());
                assert!(configured.is_dir());
                assert_eq!(fs::read(&retained)?, b"synthetic retained content");
                // Keep the separate default owner alive through cleanup; the
                // configured owner must never try to acquire its root as well.
                drop(default_cache);
                Ok(())
            })
            .await
    }
}
