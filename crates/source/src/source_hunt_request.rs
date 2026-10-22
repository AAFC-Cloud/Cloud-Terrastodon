use crate::SourceFileExtension;
use crate::SourceHuntProgress;
use crate::SourceHuntReport;
use crate::SourceRepositoryProgress;
use crate::source_directory_scan::SourceDirectoryScan;
use crate::source_directory_scan::prepare_root;
use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use eyre::Result;
use eyre::WrapErr;
use futures::StreamExt;
use futures::stream::FuturesUnordered;
use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::future::Future;
use std::future::IntoFuture;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use teamy_cancellation::CancellationScope;
use tokio::sync::watch;
use tokio::task::spawn_blocking;
use tokio::time::MissedTickBehavior;

/// Discover Git working trees and summarize their regular files offline.
///
/// All files are included, including hidden, untracked, and ignored files. Links
/// and Windows reparse points discovered beneath the root are skipped, along
/// with `.git` entries and explicitly excluded directory names. The explicitly
/// supplied root is resolved once, including any symlink or junction aliases.
/// Nested repositories own their files exclusively. Bare
/// repositories are not source working trees and are not discovered.
///
/// The extension filter selects repositories with any requested suffix; every
/// selected repository still includes its complete extension histogram. Global
/// totals cover every scanned file, including files outside repositories.
///
/// Filesystem, Git configuration, and history reads run on Tokio's blocking pool, with at
/// most `jobs` directory reads outstanding. Cancelling or dropping the future
/// closes its linked scopes so blocking directory loops stop at checkpoints.
#[must_use = "Await the request to scan local source repositories"]
#[derive(Debug, Clone, facet::Facet)]
pub struct SourceHuntRequest {
    pub root: PathBuf,
    pub extensions: Vec<SourceFileExtension>,
    /// Exact directory basenames to skip, in addition to the mandatory `.git`.
    pub exclude_dirs: Vec<String>,
    /// Maximum simultaneous blocking directory jobs; must be greater than zero.
    pub jobs: usize,
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
    #[facet(opaque, skip_serializing)]
    pub progress: Option<watch::Sender<SourceHuntProgress>>,
}

impl<'a> Arbitrary<'a> for SourceHuntRequest {
    fn arbitrary(input: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            root: PathBuf::arbitrary(input)?,
            extensions: Vec::arbitrary(input)?,
            exclude_dirs: Vec::arbitrary(input)?,
            jobs: input.int_in_range(1..=64)?,
            cancellation: CancellationScope::arbitrary(input)?,
            progress: None,
        })
    }
}

impl IntoFuture for SourceHuntRequest {
    type Output = Result<SourceHuntReport>;
    type IntoFuture = Pin<Box<dyn Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(self.cancellation.cancellable_scope(move |scan| async move {
            eyre::ensure!(self.jobs > 0, "Source hunt requires at least one directory job");
            if let Some(progress) = &self.progress {
                progress.send_replace(SourceHuntProgress {
                    active_paths: vec![self.root.clone()],
                    ..SourceHuntProgress::default()
                });
            }
            let root_scope = scan.clone();
            let (root, enclosing_repository, issues) =
                spawn_blocking(move || prepare_root(&self.root, &root_scope))
                    .await
                    .wrap_err("Joining source-hunt root preparation")??;
            scan.bail_if_cancelled()?;
            let mut report = SourceHuntReport {
                root: root.clone(),
                repositories: enclosing_repository.into_iter().collect(),
                extension_counts: BTreeMap::new(),
                files_scanned: 0,
                files_outside_repositories: 0,
                directories_scanned: 0,
                errors: issues,
            };
            let selected_extensions: HashSet<_> = self
                .extensions
                .iter()
                .map(|extension| extension.as_str().to_owned())
                .collect();
            let exclude_dirs = Arc::new(self.exclude_dirs.into_iter().collect::<HashSet<_>>());
            let mut pending = VecDeque::from([(
                root,
                (!report.repositories.is_empty()).then_some(0),
            )]);
            let mut active = FuturesUnordered::new();
            let mut active_paths = BTreeSet::new();
            let mut refresh = tokio::time::interval(Duration::from_millis(100));
            refresh.set_missed_tick_behavior(MissedTickBehavior::Skip);
            while !pending.is_empty() || !active.is_empty() {
                scan.bail_if_cancelled()?;
                while active.len() < self.jobs {
                    let Some((path, inherited_repository)) = pending.pop_front() else {
                        break;
                    };
                    active_paths.insert(path.clone());
                    let exclude_dirs = Arc::clone(&exclude_dirs);
                    active.push(scan.cancellable_scope(move |directory| async move {
                        let directory_scope = directory.clone();
                        let result = spawn_blocking(move || {
                            SourceDirectoryScan::read(&path, &exclude_dirs, &directory_scope)
                                .map(|result| (path, inherited_repository, result))
                        })
                        .await
                        .wrap_err("Joining source-hunt directory job")??;
                        directory.bail_if_cancelled()?;
                        Ok::<_, eyre::Report>(result)
                    }));
                }
                tokio::select! {
                    result = active.next(), if !active.is_empty() => {
                        let Some(result) = result else { continue };
                        let (path, mut repository_index, directory) = result?;
                        scan.bail_if_cancelled()?;
                        active_paths.remove(&path);
                        report.directories_scanned += 1;
                        report.errors.extend(directory.issues);
                        if directory.repository_boundary {
                            repository_index = None;
                        }
                        if let Some(repository) = directory.repository {
                            repository_index = Some(report.repositories.len());
                            report.repositories.push(repository);
                        }
                        for (extension, count) in directory.extension_counts {
                            scan.bail_if_cancelled()?;
                            report.files_scanned += count;
                            *report.extension_counts.entry(extension.clone()).or_default() += count;
                            if let Some(index) = repository_index {
                                let repository = &mut report.repositories[index];
                                repository.total_files += count;
                                if selected_extensions.is_empty() || selected_extensions.contains(&extension) {
                                    repository.matched_files += count;
                                }
                                *repository.extension_counts.entry(extension).or_default() += count;
                            } else {
                                report.files_outside_repositories += count;
                            }
                        }
                        pending.extend(directory.directories.into_iter().map(|path| (path, repository_index)));
                    }
                    _ = refresh.tick(), if self.progress.is_some() => {
                        if let Some(progress) = &self.progress {
                            progress.send_replace(progress_snapshot(&report, &active_paths));
                        }
                    }
                }
            }
            scan.bail_if_cancelled()?;
            if let Some(progress) = &self.progress {
                progress.send_replace(progress_snapshot(&report, &active_paths));
            }
            if !selected_extensions.is_empty() {
                report.repositories.retain(|repository| repository.matched_files > 0);
            }
            report.repositories.sort_unstable_by(|left, right| left.path.cmp(&right.path));
            report.errors.sort_unstable_by(|left, right| left.path.cmp(&right.path).then_with(|| left.message.cmp(&right.message)));
            Ok(report)
        }))
    }
}

cloud_terrastodon_registry::register_thing!(SourceHuntRequest);
cloud_terrastodon_registry::register_arbitrary!(SourceHuntRequest);
cloud_terrastodon_registry::register_into_future!(SourceHuntRequest => SourceHuntReport, effects = [Read]);

fn progress_snapshot(
    report: &SourceHuntReport,
    active_paths: &BTreeSet<PathBuf>,
) -> SourceHuntProgress {
    // Sort references, then copy only the ten visible file-count snapshots.
    // Author histories never cross the watch channel.
    let mut top_repositories: Vec<_> = report
        .repositories
        .iter()
        .filter(|repository| repository.matched_files > 0)
        .collect();
    top_repositories.sort_unstable_by(|left, right| {
        right
            .matched_files
            .cmp(&left.matched_files)
            .then_with(|| left.path.cmp(&right.path))
    });
    SourceHuntProgress {
        directories_scanned: report.directories_scanned,
        files_scanned: report.files_scanned,
        repositories_found: report.repositories.len(),
        active_paths: active_paths.iter().cloned().collect(),
        extension_counts: report.extension_counts.clone(),
        top_repositories: top_repositories
            .into_iter()
            .take(10)
            .map(|repository| SourceRepositoryProgress {
                path: repository.path.clone(),
                total_files: repository.total_files,
                matched_files: repository.matched_files,
            })
            .collect(),
        errors: report.errors.len(),
    }
}
