use cloud_terrastodon_cache::ArtifactEntry;
use cloud_terrastodon_cache::ArtifactFilePath;
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
use std::collections::BTreeMap;
use std::fs;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

struct SyntheticRead;

fn entry(cache: &WorkCache) -> eyre::Result<ArtifactEntry> {
    cache.artifacts::<String>(
        ExecutorKind::Rest,
        CacheKey {
            scope: CacheScope::try_from_components(["synthetic", "input-integrity"])?,
            valid_for: Duration::from_secs(3600),
        },
        CacheOperationOwner::of::<SyntheticRead>(),
        AuthPartition::new("synthetic-session")?,
        NonEmptyString::new("synthetic-query")?,
        ArtifactRetention::Full,
    )
}

async fn publish_original(entry: &ArtifactEntry, scope: &CancellationScope) -> eyre::Result<()> {
    let original = entry
        .begin_attempt(BTreeMap::new(), scope)
        .await?
        .unwrap()
        .finish(
            ArtifactSet {
                payload: b"original selected result".to_vec(),
                diagnostics: BTreeMap::new(),
            },
            ArtifactObservation::now(),
            scope,
        )
        .await?;
    assert!(entry.publish(&original, scope).await?);
    Ok(())
}

#[tokio::test]
async fn mutated_staged_input_prevents_publication_without_replacing_the_previous_result()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = entry(&cache)?;
            publish_original(&entry, &scope).await?;
            let selected = entry.load(&scope).await?.unwrap();
            let input = ArtifactFilePath::new("nested/input.txt")?;
            let attempt = entry
                .begin_attempt(
                    BTreeMap::from([(input.clone(), b"declared input".to_vec())]),
                    &scope,
                )
                .await?
                .unwrap();
            fs::write(attempt.input_path(&input)?, b"modified during execution")?;
            let candidate = attempt
                .finish(
                    ArtifactSet {
                        payload: b"result using modified input".to_vec(),
                        diagnostics: BTreeMap::new(),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert!(entry.publish(&candidate, &scope).await.is_err());
            let still_selected = entry.load(&scope).await?.unwrap();
            assert_eq!(still_selected.artifacts.payload, selected.artifacts.payload);
            assert_eq!(still_selected.freshness, selected.freshness);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn conflicting_input_file_and_directory_leave_previous_selection_reusable() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = entry(&cache)?;
            publish_original(&entry, &scope).await?;
            let selected = entry.load(&scope).await?.unwrap();
            let failed = entry
                .begin_attempt(
                    BTreeMap::from([
                        (ArtifactFilePath::new("a")?, b"file contents".to_vec()),
                        (ArtifactFilePath::new("a/b")?, b"nested contents".to_vec()),
                    ]),
                    &scope,
                )
                .await;
            assert!(failed.is_err());
            let still_selected = entry.load(&scope).await?.unwrap();
            assert_eq!(still_selected.artifacts.payload, selected.artifacts.payload);
            assert_eq!(still_selected.freshness, selected.freshness);
            Ok(())
        })
        .await
}

#[cfg(any(unix, windows))]
#[tokio::test]
async fn symlinked_store_directory_rejects_reads_and_writes() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let outside = tempfile::tempdir()?;
            let sentinel = outside.path().join("sentinel.txt");
            fs::write(&sentinel, b"synthetic external sentinel")?;
            let link = temporary.path().join("artifacts-v2");
            #[cfg(unix)]
            let linked = std::os::unix::fs::symlink(outside.path(), &link);
            #[cfg(windows)]
            let linked = std::os::windows::fs::symlink_dir(outside.path(), &link);
            #[cfg(windows)]
            if linked
                .as_ref()
                .is_err_and(|error| error.raw_os_error() == Some(1314))
            {
                eprintln!("Skipping symlink policy test: Windows symlink privilege is unavailable");
                return Ok(());
            }
            linked?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = entry(&cache)?;
            assert!(entry.load(&scope).await.is_err());
            assert!(
                entry
                    .begin_attempt(
                        BTreeMap::from([(
                            ArtifactFilePath::new("input.txt")?,
                            b"must remain inside cache root".to_vec(),
                        )]),
                        &scope,
                    )
                    .await
                    .is_err()
            );
            assert_eq!(fs::read(&sentinel)?, b"synthetic external sentinel");
            assert_eq!(fs::read_dir(outside.path())?.count(), 1);
            Ok(())
        })
        .await
}

#[tokio::test]
async fn missing_required_file_lists_make_selected_artifacts_unusable() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = entry(&cache)?;
            let stored = entry
                .begin_attempt(
                    BTreeMap::from([(
                        ArtifactFilePath::new("input.txt")?,
                        b"required synthetic input".to_vec(),
                    )]),
                    &scope,
                )
                .await?
                .unwrap()
                .finish(
                    ArtifactSet {
                        payload: b"synthetic payload".to_vec(),
                        diagnostics: BTreeMap::from([(
                            ArtifactFilePath::new("details.txt")?,
                            b"required synthetic diagnostic".to_vec(),
                        )]),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert!(entry.publish(&stored, &scope).await?);
            assert!(entry.load(&scope).await?.is_some());
            let manifest_path = stored.directory().join("manifest.json");
            let manifest = fs::read_to_string(&manifest_path)?;
            for field in ["inputs", "diagnostics"] {
                // These real manifest arrays contain only the simple filenames
                // above and tagged digests, so the first closing bracket ends the
                // whole field. Remove its adjacent comma to retain valid JSON.
                let field_start = format!("\"{field}\":[");
                assert_eq!(manifest.matches(&field_start).count(), 1);
                let start = manifest.find(&field_start).unwrap();
                let end = start + manifest[start..].find(']').unwrap() + 1;
                let mut incomplete = manifest.clone();
                if manifest.as_bytes().get(end) == Some(&b',') {
                    incomplete.replace_range(start..end + 1, "");
                } else {
                    assert_eq!(manifest.as_bytes().get(start - 1), Some(&b','));
                    incomplete.replace_range(start - 1..end, "");
                }
                fs::write(&manifest_path, incomplete)?;
                assert!(
                    entry.load(&scope).await?.is_none(),
                    "a manifest missing its {field} list must not bypass file integrity checks",
                );
                fs::write(&manifest_path, &manifest)?;
                assert!(entry.load(&scope).await?.is_some());
            }
            Ok(())
        })
        .await
}
