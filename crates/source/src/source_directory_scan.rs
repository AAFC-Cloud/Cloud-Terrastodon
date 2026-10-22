use crate::SourceHuntIssue;
use crate::SourceRepositoryRemote;
use crate::SourceRepositorySummary;
use crate::read_repository_history::read_repository_history;
use eyre::Result;
use eyre::WrapErr;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::fs::Metadata;
use std::io::ErrorKind;
use std::path::Path;
use std::path::PathBuf;
use teamy_cancellation::CancellationReportExt;
use teamy_cancellation::CancellationScope;

/// The bounded unit of blocking filesystem work and its aggregated findings.
/// Only directory paths and extension totals survive enumeration, never a list
/// of individual files or an open directory iterator.
#[derive(Debug, Default)]
pub(crate) struct SourceDirectoryScan {
    pub directories: Vec<PathBuf>,
    pub extension_counts: BTreeMap<String, u64>,
    /// A `.git` marker stops inheritance even when its metadata is unreadable.
    pub repository_boundary: bool,
    pub repository: Option<SourceRepositorySummary>,
    pub issues: Vec<SourceHuntIssue>,
}

impl SourceDirectoryScan {
    pub fn read(
        path: &Path,
        exclude_dirs: &HashSet<String>,
        cancellation: &CancellationScope,
    ) -> Result<Self> {
        let mut scan = Self::default();
        // Recheck after scheduling: a directory may have become a link while it
        // was queued. Windows junctions are reparse points, not always symlinks.
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                scan.issues.push(SourceHuntIssue {
                    path: path.to_owned(),
                    message: format!("Reading directory metadata: {error}"),
                });
                return Ok(scan);
            }
        };
        if metadata_is_link(&metadata) {
            return Ok(scan);
        }
        (scan.repository_boundary, scan.repository) =
            repository_at(path, &mut scan.issues, cancellation)?;
        cancellation.bail_if_cancelled()?;
        let entries = match fs::read_dir(path) {
            Ok(entries) => entries,
            Err(error) => {
                scan.issues.push(SourceHuntIssue {
                    path: path.to_owned(),
                    message: format!("Reading directory: {error}"),
                });
                return Ok(scan);
            }
        };
        for entry in entries {
            cancellation.bail_if_cancelled()?;
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    scan.issues.push(SourceHuntIssue {
                        path: path.to_owned(),
                        message: format!("Reading directory entry: {error}"),
                    });
                    continue;
                }
            };
            if is_git_entry(&entry.file_name()) {
                continue;
            }
            // DirEntry::metadata does not traverse symlinks, and its Windows
            // attributes include FILE_ATTRIBUTE_REPARSE_POINT for junctions.
            let metadata = match entry.metadata() {
                Ok(metadata) => metadata,
                Err(error) => {
                    scan.issues.push(SourceHuntIssue {
                        path: entry.path(),
                        message: format!("Reading entry metadata: {error}"),
                    });
                    continue;
                }
            };
            if metadata_is_link(&metadata) {
                continue;
            }
            if metadata.is_dir() {
                if !exclude_dirs.contains(entry.file_name().to_string_lossy().as_ref()) {
                    scan.directories.push(entry.path());
                }
            } else if metadata.is_file() {
                let extension = entry
                    .path()
                    .extension()
                    .filter(|extension| !extension.is_empty())
                    .map(|extension| extension.to_string_lossy().to_lowercase())
                    // A suffix cannot contain a dot, making this extensionless
                    // bucket collision-free and usable by PowerShell JSON objects.
                    .unwrap_or_else(|| ".".to_owned());
                *scan.extension_counts.entry(extension).or_default() += 1;
            }
        }
        Ok(scan)
    }
}

pub(crate) fn prepare_root(
    root: &Path,
    cancellation: &CancellationScope,
) -> Result<(
    PathBuf,
    Option<SourceRepositorySummary>,
    Vec<SourceHuntIssue>,
)> {
    let absolute = std::path::absolute(root)
        .wrap_err_with(|| format!("Resolving scan root {}", root.display()))?;
    for ancestor in absolute.ancestors() {
        eyre::ensure!(
            !ancestor.file_name().is_some_and(is_git_entry),
            "Scan root is inside excluded Git metadata: {}",
            ancestor.display()
        );
        cancellation.bail_if_cancelled()?;
    }
    // The caller explicitly selected this path, including any aliases in its
    // ancestry. Resolve that selection once; links discovered below the resolved
    // root are still excluded by ordinary directory jobs.
    let root = fs::canonicalize(root)
        .wrap_err_with(|| format!("Resolving scan root {}", root.display()))?;
    for ancestor in root.ancestors() {
        eyre::ensure!(
            !ancestor.file_name().is_some_and(is_git_entry),
            "Scan root resolves inside excluded Git metadata: {}",
            ancestor.display()
        );
        cancellation.bail_if_cancelled()?;
    }
    // Validate the requested root before admitting descendant work. An unreadable
    // root is fatal; descendant failures are reported alongside useful siblings.
    fs::read_dir(&root)
        .wrap_err_with(|| format!("Opening scan-root directory {}", root.display()))?;
    let mut issues = Vec::new();
    let mut repository = None;
    // A requested repository root belongs to its own ordinary job. Do not seed
    // an ancestor repository that would otherwise appear as a spurious empty row.
    if !matches!(fs::symlink_metadata(root.join(".git")), Err(error) if error.kind() == ErrorKind::NotFound)
    {
        return Ok((root, None, issues));
    }
    // The root itself is inspected by its ordinary directory job. Its closest
    // ancestor may already own the files even when the scan starts deep inside it.
    for ancestor in root.ancestors().skip(1) {
        cancellation.bail_if_cancelled()?;
        let (boundary, found) = repository_at(ancestor, &mut issues, cancellation)?;
        if boundary {
            repository = found;
            break;
        }
    }
    Ok((root, repository, issues))
}

fn is_git_entry(name: &OsStr) -> bool {
    #[cfg(windows)]
    {
        name.to_string_lossy().eq_ignore_ascii_case(".git")
    }
    #[cfg(not(windows))]
    {
        name == ".git"
    }
}

pub(crate) fn metadata_is_link(metadata: &Metadata) -> bool {
    if metadata.file_type().is_symlink() {
        return true;
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_REPARSE_POINT: u32 = 0x400;
        metadata.file_attributes() & FILE_ATTRIBUTE_REPARSE_POINT != 0
    }
    #[cfg(not(windows))]
    {
        false
    }
}

fn repository_at(
    path: &Path,
    issues: &mut Vec<SourceHuntIssue>,
    cancellation: &CancellationScope,
) -> Result<(bool, Option<SourceRepositorySummary>)> {
    let marker = path.join(".git");
    match fs::symlink_metadata(&marker) {
        Ok(metadata) if metadata_is_link(&metadata) => return Ok((true, None)),
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => return Ok((false, None)),
        Err(error) => {
            issues.push(SourceHuntIssue {
                path: marker,
                message: format!("Inspecting Git repository marker: {error}"),
            });
            return Ok((true, None));
        }
    }
    let repository = match gix::open::Options::isolated()
        .strict_config(true)
        .open(path)
    {
        Ok(repository) => repository.to_thread_local(),
        Err(error) => {
            issues.push(SourceHuntIssue {
                path: path.to_owned(),
                message: format!("Opening Git repository: {error:#}"),
            });
            return Ok((true, None));
        }
    };
    cancellation.bail_if_cancelled()?;
    // Isolated opening reads repository-local configuration only. Snapshot
    // lookups neither invoke credential helpers nor contact remote transports.
    // They retain opaque configured strings and avoid insteadOf rewriting.
    // https://docs.rs/gix/0.84.0/gix/open/struct.Options.html#method.isolated
    let config = repository.config_snapshot();
    let mut remotes = BTreeMap::new();
    if let Some(sections) = config.sections_by_name("remote") {
        for section in sections {
            cancellation.bail_if_cancelled()?;
            let Some(name) = section.header().subsection_name() else {
                continue;
            };
            remotes.insert(
                name.to_string(),
                SourceRepositoryRemote {
                    name: name.to_string(),
                    fetch_url: config
                        .string_by("remote", Some(name), "url")
                        .map(|value| value.to_string()),
                    push_url: config
                        .string_by("remote", Some(name), "pushurl")
                        .map(|value| value.to_string()),
                },
            );
        }
    }
    let history = match read_repository_history(&repository, cancellation) {
        Ok(history) => Some(history),
        Err(error) if error.is_cancellation() => return Err(error),
        Err(error) => {
            issues.push(SourceHuntIssue {
                path: path.to_owned(),
                message: format!("Reading Git author history: {error:#}"),
            });
            None
        }
    };
    Ok((
        true,
        Some(SourceRepositorySummary {
            path: path.to_owned(),
            remotes: remotes.into_values().collect(),
            history,
            extension_counts: BTreeMap::new(),
            total_files: 0,
            matched_files: 0,
        }),
    ))
}
