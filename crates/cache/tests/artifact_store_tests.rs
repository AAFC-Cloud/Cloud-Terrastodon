#![cfg(test)]

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
use cloud_terrastodon_cache::StoredArtifact;
use cloud_terrastodon_cache::WorkCache;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::fs;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use teamy_cancellation::CancellationToken;

struct SyntheticRead;
struct OtherSyntheticRead;

fn entry(
    cache: &WorkCache,
    kind: ExecutorKind,
    components: &[&str],
    partition: &str,
    fingerprint: &str,
    retention: ArtifactRetention,
) -> eyre::Result<ArtifactEntry> {
    cache.artifacts::<String>(
        kind,
        CacheKey {
            scope: CacheScope::try_from_components(components.iter().copied())?,
            valid_for: Duration::from_secs(3600),
        },
        CacheOperationOwner::of::<SyntheticRead>(),
        AuthPartition::new(partition)?,
        NonEmptyString::new(fingerprint)?,
        retention,
    )
}

fn read_entry(cache: &WorkCache, fingerprint: &str) -> eyre::Result<ArtifactEntry> {
    entry(
        cache,
        ExecutorKind::Rest,
        &["projects", "synthetic", "list"],
        "synthetic-session",
        fingerprint,
        ArtifactRetention::Full,
    )
}

async fn finish(
    entry: &ArtifactEntry,
    payload: &[u8],
    scope: &CancellationScope,
) -> eyre::Result<StoredArtifact> {
    entry
        .begin_attempt(BTreeMap::new(), scope)
        .await?
        .expect("full retention admits an artifact attempt")
        .finish(
            ArtifactSet {
                payload: payload.to_vec(),
                diagnostics: BTreeMap::new(),
            },
            ArtifactObservation::now(),
            scope,
        )
        .await
}

#[tokio::test]
async fn exact_variants_namespaces_and_scope_components_keep_distinct_artifacts() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let variants = [
                (
                    ExecutorKind::Rest,
                    vec!["projects", "Équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects", "Équipe", "list"],
                    "session-a",
                    "query-b",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects", "Équipe", "list"],
                    "session-b",
                    "query-a",
                ),
                (
                    ExecutorKind::Command,
                    vec!["projects", "Équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::InProcess,
                    vec!["projects", "Équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Other("Rest".to_owned()),
                    vec!["projects", "Équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects", "équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects/Équipe", "list"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects", "Équipe", "listing"],
                    "session-a",
                    "query-a",
                ),
                (
                    ExecutorKind::Rest,
                    vec!["projects", "Équipe", "list", "page-1"],
                    "session-a",
                    "query-a",
                ),
            ];
            let mut entries = Vec::new();
            let mut directories = HashSet::new();
            for (index, (kind, components, partition, fingerprint)) in
                variants.into_iter().enumerate()
            {
                let entry = entry(
                    &cache,
                    kind,
                    &components,
                    partition,
                    fingerprint,
                    ArtifactRetention::Full,
                )?;
                assert!(directories.insert(entry.directory()?));
                let payload = format!("synthetic variant {index}").into_bytes();
                let stored = finish(&entry, &payload, &scope).await?;
                assert!(entry.publish(&stored, &scope).await?);
                entries.push((entry, payload));
            }
            // Read only after all writes: a physical alias would now return
            // another variant's payload or report an identity mismatch.
            for (entry, expected) in entries {
                assert_eq!(
                    entry.load(&scope).await?.unwrap().artifacts.payload,
                    expected
                );
            }
            Ok(())
        })
        .await
}

#[test]
fn conflicting_contracts_fail_before_creating_an_artifact_directory() -> eyre::Result<()> {
    let temporary = tempfile::tempdir()?;
    let missing = temporary.path().join("not-created");
    let cache = WorkCache::new(&missing)?;
    let first = read_entry(&cache, "original")?;
    let key = CacheKey {
        scope: CacheScope::try_from_components(["projects", "synthetic", "list"])?,
        valid_for: Duration::ZERO,
    };
    assert!(
        cache
            .artifacts::<String>(
                ExecutorKind::Rest,
                key.clone(),
                CacheOperationOwner::of::<OtherSyntheticRead>(),
                AuthPartition::new("another-session")?,
                NonEmptyString::new("another-fingerprint")?,
                ArtifactRetention::Full,
            )
            .is_err()
    );
    assert!(
        cache
            .artifacts::<u64>(
                ExecutorKind::Rest,
                key,
                CacheOperationOwner::of::<SyntheticRead>(),
                AuthPartition::new("another-session")?,
                NonEmptyString::new("another-fingerprint")?,
                ArtifactRetention::Full,
            )
            .is_err()
    );
    assert!(!missing.exists());
    assert!(!first.directory()?.exists());
    Ok(())
}

#[tokio::test]
async fn simultaneous_attempts_have_separate_input_workspaces_and_diagnostic_files()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = read_entry(&cache, "same-input-name")?;
            let input_name = ArtifactFilePath::new("nested/input.txt")?;
            let stage = |contents: &'static [u8]| {
                let entry = &entry;
                let input_name = &input_name;
                scope.cancellable_scope(move |child| async move {
                    entry
                        .begin_attempt(
                            BTreeMap::from([(input_name.clone(), contents.to_vec())]),
                            &child,
                        )
                        .await
                })
            };
            let (first, second) = tokio::join!(
                stage(b"first synthetic input"),
                stage(b"second synthetic input"),
            );
            let first = first?.unwrap();
            let second = second?.unwrap();
            assert_ne!(first.inputs_directory(), second.inputs_directory());
            assert_ne!(
                first.input_path(&input_name)?,
                second.input_path(&input_name)?
            );
            assert_eq!(
                fs::read(first.input_path(&input_name)?)?,
                b"first synthetic input"
            );
            assert_eq!(
                fs::read(second.input_path(&input_name)?)?,
                b"second synthetic input"
            );
            let diagnostic = ArtifactFilePath::new("payload.bin")?;
            let stored = first
                .finish(
                    ArtifactSet {
                        payload: b"canonical raw payload".to_vec(),
                        diagnostics: BTreeMap::from([(
                            diagnostic.clone(),
                            b"diagnostic payload".to_vec(),
                        )]),
                    },
                    ArtifactObservation::now(),
                    &scope,
                )
                .await?;
            assert!(entry.publish(&stored, &scope).await?);
            let loaded = entry.load(&scope).await?.unwrap();
            assert_eq!(loaded.artifacts.payload, b"canonical raw payload");
            assert_eq!(
                loaded.artifacts.diagnostics[&diagnostic],
                b"diagnostic payload"
            );
            assert_eq!(
                fs::read(second.input_path(&input_name)?)?,
                b"second synthetic input"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn only_explicitly_published_complete_attempts_are_reusable() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = read_entry(&cache, "publication")?;
            let unfinished = entry.begin_attempt(BTreeMap::new(), &scope).await?.unwrap();
            assert!(entry.load(&scope).await?.is_none());
            drop(unfinished);
            let first = finish(&entry, b"first result", &scope).await?;
            assert!(first.directory().join("manifest.json").exists());
            assert!(entry.load(&scope).await?.is_none());
            assert!(entry.publish(&first, &scope).await?);
            let initial = entry.load(&scope).await?.unwrap();
            assert_eq!(initial.artifacts.payload, b"first result");
            let second = finish(&entry, b"second result", &scope).await?;
            let still_initial = entry.load(&scope).await?.unwrap();
            assert_eq!(still_initial.artifacts.payload, b"first result");
            assert_eq!(still_initial.freshness, initial.freshness);
            assert!(entry.publish(&second, &scope).await?);
            let replaced = entry.load(&scope).await?.unwrap();
            assert_eq!(replaced.artifacts.payload, b"second result");
            assert!(replaced.freshness.observed_at >= initial.freshness.observed_at);
            assert_eq!(
                entry.load(&scope).await?.unwrap().freshness,
                replaced.freshness
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn caller_lifetime_narrows_loaded_freshness_without_reanchoring_observation()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let original = read_entry(&cache, "caller-lifetime")?;
            let stored = finish(&original, b"synthetic payload", &scope).await?;
            assert!(original.publish(&stored, &scope).await?);
            let first = original.load(&scope).await?.unwrap();
            // A separately opened owner exercises disk admission without borrowing
            // any in-memory observation from the writer.
            let reopened = WorkCache::new(temporary.path())?;
            for validity in [Duration::from_secs(30), Duration::MAX] {
                let reader = reopened.artifacts::<String>(
                    ExecutorKind::Rest,
                    CacheKey {
                        scope: CacheScope::try_from_components(["projects", "synthetic", "list"])?,
                        valid_for: validity,
                    },
                    CacheOperationOwner::of::<SyntheticRead>(),
                    AuthPartition::new("synthetic-session")?,
                    NonEmptyString::new("caller-lifetime")?,
                    ArtifactRetention::Full,
                )?;
                let loaded = reader.load(&scope).await?.unwrap();
                assert_eq!(loaded.freshness.observed_at, first.freshness.observed_at);
                assert_eq!(
                    loaded.freshness.expires_at,
                    first.freshness.effective_expires_at(validity)?
                );
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn missing_corrupt_or_partial_artifacts_are_misses() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            for (fingerprint, file, replacement) in [
                ("missing-payload", "payload.bin", None),
                (
                    "changed-payload",
                    "payload.bin",
                    Some(b"changed".as_slice()),
                ),
                ("missing-diagnostic", "diagnostics/details.txt", None),
                (
                    "changed-diagnostic",
                    "diagnostics/details.txt",
                    Some(b"changed".as_slice()),
                ),
                ("missing-manifest", "manifest.json", None),
                ("partial-manifest", "manifest.json", Some(b"{".as_slice())),
            ] {
                let entry = read_entry(&cache, fingerprint)?;
                let stored = entry
                    .begin_attempt(BTreeMap::new(), &scope)
                    .await?
                    .unwrap()
                    .finish(
                        ArtifactSet {
                            payload: b"intact synthetic payload".to_vec(),
                            diagnostics: BTreeMap::from([(
                                ArtifactFilePath::new("details.txt")?,
                                b"synthetic detail".to_vec(),
                            )]),
                        },
                        ArtifactObservation::now(),
                        &scope,
                    )
                    .await?;
                assert!(entry.publish(&stored, &scope).await?);
                assert!(entry.load(&scope).await?.is_some());
                let target = stored.directory().join(file);
                match replacement {
                    Some(bytes) => fs::write(target, bytes)?,
                    None => fs::remove_file(target)?,
                }
                assert!(
                    entry.load(&scope).await?.is_none(),
                    "accepted {fingerprint}"
                );
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn legacy_files_without_new_identity_or_freshness_are_preserved_and_ignored()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let legacy = temporary.path().join("projects/synthetic/list");
            fs::create_dir_all(&legacy)?;
            fs::write(legacy.join("stdout.json"), b"synthetic legacy payload")?;
            fs::write(legacy.join("status.txt"), b"0")?;
            fs::write(legacy.join("timestamp.txt"), b"synthetic legacy timestamp")?;
            fs::write(legacy.join("metadata.json"), b"synthetic legacy metadata")?;
            let entry = read_entry(&cache, "legacy")?;
            assert!(entry.load(&scope).await?.is_none());
            let stored = finish(&entry, b"new verified artifact", &scope).await?;
            assert!(entry.publish(&stored, &scope).await?);
            assert_eq!(
                entry.load(&scope).await?.unwrap().artifacts.payload,
                b"new verified artifact"
            );
            assert_eq!(
                fs::read(legacy.join("stdout.json"))?,
                b"synthetic legacy payload"
            );
            assert_eq!(
                fs::read(legacy.join("metadata.json"))?,
                b"synthetic legacy metadata"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn persisted_type_names_are_diagnostic_and_never_select_a_decoder() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = read_entry(&cache, "diagnostic-type")?;
            let stored = finish(&entry, b"raw payload for the current decoder", &scope).await?;
            assert!(entry.publish(&stored, &scope).await?);
            let initial = entry.load(&scope).await?.unwrap();
            let manifest_path = stored.directory().join("manifest.json");
            let mut manifest = fs::read_to_string(&manifest_path)?;
            for (old_name, new_name) in [
                (
                    &initial.metadata.output_type_name,
                    "old_release::UnrelatedDiagnosticType",
                ),
                (
                    &initial.metadata.operation_owner_type_name,
                    "old_release::UnrelatedOperationOwner",
                ),
            ] {
                let old_name = facet_json::to_string(old_name)?;
                let new_name = facet_json::to_string(&new_name)?;
                assert_eq!(manifest.matches(&old_name).count(), 1);
                manifest = manifest.replace(&old_name, &new_name);
            }
            fs::write(&manifest_path, manifest)?;
            let loaded = entry.load(&scope).await?.unwrap();
            assert_eq!(
                loaded.metadata.output_type_name,
                "old_release::UnrelatedDiagnosticType"
            );
            assert_eq!(
                loaded.metadata.operation_owner_type_name,
                "old_release::UnrelatedOperationOwner"
            );
            assert_eq!(
                loaded.artifacts.payload,
                b"raw payload for the current decoder"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn a_complete_record_with_another_exact_identity_is_an_error() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = read_entry(&cache, "fingerprint-original")?;
            let stored = finish(&entry, b"synthetic data", &scope).await?;
            assert!(entry.publish(&stored, &scope).await?);
            let manifest_path = stored.directory().join("manifest.json");
            let manifest = fs::read_to_string(&manifest_path)?;
            let old_identity = facet_json::to_string(&"fingerprint-original")?;
            let new_identity = facet_json::to_string(&"fingerprint-another")?;
            assert_eq!(manifest.matches(&old_identity).count(), 1);
            fs::write(
                manifest_path,
                manifest.replace(&old_identity, &new_identity),
            )?;
            assert!(entry.load(&scope).await.is_err());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn disabled_retention_never_creates_inputs_or_failure_diagnostics() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let absent_root = temporary.path().join("sensitive-root");
            let cache = WorkCache::new(&absent_root)?;
            for (fingerprint, outcome) in [
                (
                    "credential-success",
                    ArtifactSet {
                        payload: b"SYNTHETIC_SUCCESS_SECRET".to_vec(),
                        diagnostics: BTreeMap::new(),
                    },
                ),
                (
                    "credential-failure",
                    ArtifactSet {
                        payload: Vec::new(),
                        diagnostics: BTreeMap::from([(
                            ArtifactFilePath::new("failure-diagnostic.txt")?,
                            b"SYNTHETIC_FAILURE_SECRET".to_vec(),
                        )]),
                    },
                ),
            ] {
                let entry = entry(
                    &cache,
                    ExecutorKind::Command,
                    &["credentials", "synthetic"],
                    "synthetic-provider",
                    fingerprint,
                    ArtifactRetention::Disabled,
                )?;
                assert!(entry.load(&scope).await?.is_none());
                let retained = match entry
                    .begin_attempt(
                        BTreeMap::from([(
                            ArtifactFilePath::new("credential-input.txt")?,
                            b"SYNTHETIC_INPUT_SECRET".to_vec(),
                        )]),
                        &scope,
                    )
                    .await?
                {
                    Some(attempt) => Some(
                        attempt
                            .finish(outcome, ArtifactObservation::now(), &scope)
                            .await?,
                    ),
                    None => None,
                };
                assert!(retained.is_none());
                assert!(!absent_root.exists());
            }
            Ok(())
        })
        .await
}

#[tokio::test]
async fn disabled_retention_bypasses_existing_artifacts_without_deleting_them() -> eyre::Result<()>
{
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let regular = read_entry(&cache, "policy-change")?;
            let stored = finish(&regular, b"synthetic prior retained result", &scope).await?;
            assert!(regular.publish(&stored, &scope).await?);
            let disabled = entry(
                &cache,
                ExecutorKind::Rest,
                &["projects", "synthetic", "list"],
                "synthetic-session",
                "policy-change",
                ArtifactRetention::Disabled,
            )?;
            assert_eq!(regular.directory()?, disabled.directory()?);
            assert!(disabled.load(&scope).await?.is_none());
            assert!(!disabled.publish(&stored, &scope).await?);
            assert!(
                disabled
                    .begin_attempt(BTreeMap::new(), &scope)
                    .await?
                    .is_none()
            );
            assert_eq!(
                regular.load(&scope).await?.unwrap().artifacts.payload,
                b"synthetic prior retained result"
            );
            Ok(())
        })
        .await
}

#[tokio::test]
async fn zero_validity_retains_inspectable_artifacts_without_reuse() -> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = cache.artifacts::<String>(
                ExecutorKind::Rest,
                CacheKey {
                    scope: CacheScope::try_from_components(["synthetic", "mutation"])?,
                    valid_for: Duration::ZERO,
                },
                CacheOperationOwner::of::<SyntheticRead>(),
                AuthPartition::new("synthetic-session")?,
                NonEmptyString::new("mutation")?,
                ArtifactRetention::Full,
            )?;
            let stored = finish(&entry, b"synthetic mutation response", &scope).await?;
            assert_eq!(
                fs::read(stored.directory().join("payload.bin"))?,
                b"synthetic mutation response"
            );
            assert!(!entry.publish(&stored, &scope).await?);
            assert!(entry.load(&scope).await?.is_none());
            assert!(stored.directory().join("manifest.json").is_file());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn failed_publication_quarantines_even_a_restored_old_selection_until_recovery()
-> eyre::Result<()> {
    CancellationToken::new()
        .with_async_scope(|scope| async move {
            let temporary = tempfile::tempdir()?;
            let cache = WorkCache::new(temporary.path())?;
            let entry = read_entry(&cache, "publication-failure")?;
            let original = finish(&entry, b"original selection", &scope).await?;
            assert!(entry.publish(&original, &scope).await?);
            let selection = entry.directory()?.join("selection.json");
            let original_selection = fs::read(&selection)?;
            fs::remove_file(&selection)?;
            fs::create_dir(&selection)?;
            let failed = finish(&entry, b"failed publication candidate", &scope).await?;
            assert!(entry.publish(&failed, &scope).await.is_err());
            fs::remove_dir(&selection)?;
            fs::write(&selection, original_selection)?;
            assert!(entry.load(&scope).await?.is_none());
            let recovered = finish(&entry, b"fresh successful selection", &scope).await?;
            assert!(entry.publish(&recovered, &scope).await?);
            assert_eq!(
                entry.load(&scope).await?.unwrap().artifacts.payload,
                b"fresh successful selection"
            );
            Ok(())
        })
        .await
}
