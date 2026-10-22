use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_source::SourceFileExtension;
use cloud_terrastodon_source::SourceHuntProgress;
use cloud_terrastodon_source::SourceHuntReport;
use cloud_terrastodon_source::SourceHuntRequest;
use eyre::Result;
use eyre::WrapErr;
use std::fs;
use std::future::IntoFuture;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use teamy_cancellation::CancellationToken;
use tempfile::TempDir;
use tokio::sync::watch;

// Fixtures create their own working trees. No service access, existing Git
// repositories, credentials, or production source files are used by these tests.
struct Fixture {
    temporary: TempDir,
}

impl Fixture {
    fn new() -> Result<Self> {
        Ok(Self {
            temporary: tempfile::tempdir()?,
        })
    }

    fn root(&self) -> &Path {
        self.temporary.path()
    }

    fn path(&self, relative: &str) -> PathBuf {
        self.root().join(relative)
    }

    fn repository(&self, relative: &str) -> Result<PathBuf> {
        let path = self.path(relative);
        fs::create_dir_all(&path)?;
        self.git(&path, &["init", "--quiet", "--template="])?;
        Ok(path)
    }

    fn write(&self, relative: &str, contents: &str) -> Result<()> {
        let path = self.path(relative);
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, contents)?;
        Ok(())
    }

    fn git(&self, path: &Path, args: &[&str]) -> Result<()> {
        self.git_with_env(path, args, &[])
    }

    fn commit(&self, path: &Path, name: &str, email: &str, authored_at: &str) -> Result<()> {
        self.git_with_env(
            path,
            &[
                "commit",
                "--quiet",
                "--allow-empty",
                "--no-gpg-sign",
                "--no-verify",
                "-m",
                "fixture commit",
            ],
            &[
                ("GIT_AUTHOR_NAME", name),
                ("GIT_AUTHOR_EMAIL", email),
                ("GIT_AUTHOR_DATE", authored_at),
            ],
        )
    }

    fn git_with_env(&self, path: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<()> {
        let mut command = Command::new("git");
        command
            .arg("-C")
            .arg(path)
            .args(["-c", "init.defaultBranch=main"])
            .args(args)
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", self.path("nonexistent-global-config"))
            .env("GIT_AUTHOR_NAME", "Fixture")
            .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
            .env("GIT_AUTHOR_DATE", "2030-01-01T00:00:00+00:00")
            .env("GIT_COMMITTER_NAME", "Fixture committer")
            .env("GIT_COMMITTER_EMAIL", "committer@example.invalid")
            .env("GIT_COMMITTER_DATE", "2030-01-01T00:00:00+00:00");
        for name in [
            "GIT_CONFIG_PARAMETERS",
            "GIT_CONFIG_COUNT",
            "GIT_CONFIG",
            "GIT_COMMON_DIR",
            "GIT_DIR",
            "GIT_INDEX_FILE",
            "GIT_WORK_TREE",
            "GIT_NAMESPACE",
            "GIT_OBJECT_DIRECTORY",
            "GIT_ALTERNATE_OBJECT_DIRECTORIES",
        ] {
            command.env_remove(name);
        }
        command.envs(env.iter().copied());
        let output = command
            .output()
            .wrap_err("Running local Git fixture setup")?;
        eyre::ensure!(
            output.status.success(),
            "Local Git fixture setup failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(())
    }
}

async fn hunt(
    root: &Path,
    extensions: &[&str],
    exclude_dirs: &[&str],
    jobs: usize,
) -> Result<SourceHuntReport> {
    let extensions = extensions
        .iter()
        .map(SourceFileExtension::try_new)
        .collect::<Result<Vec<_>>>()?;
    CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            SourceHuntRequest {
                root: root.to_owned(),
                extensions,
                exclude_dirs: exclude_dirs
                    .iter()
                    .map(|value| (*value).to_owned())
                    .collect(),
                jobs,
                cancellation: cancellation.clone(),
                progress: None,
            }
            .await
        })
        .await
}

#[tokio::test]
async fn partitions_nested_repositories_and_filters_without_losing_other_extensions() -> Result<()>
{
    let fixture = Fixture::new()?;
    let parent = fixture.repository("parent")?;
    let nested = fixture.repository("parent/nested")?;
    fixture.repository("other")?;
    fixture.write("outside.txt", "outside")?;
    fixture.write("parent/app.PHP", "php")?;
    fixture.write("parent/ui.js", "js")?;
    fixture.write("parent/.env", "hidden, extensionless")?;
    fixture.write("parent/.gitignore", "vendor/\n")?;
    fixture.write("parent/vendor/ignored.ts", "ignored files still count")?;
    fixture.write("parent/node_modules/excluded.js", "explicit exclusion")?;
    fixture.write("parent/nested/widget.ts", "ts")?;
    fixture.write("parent/nested/api.php", "php")?;
    fixture.write("other/readme.md", "md")?;
    fixture.git(
        &parent,
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/team/php.git",
        ],
    )?;
    fixture.git(
        &parent,
        &[
            "config",
            "remote.origin.pushurl",
            "git@example.invalid:team/php.git",
        ],
    )?;

    let serial = hunt(fixture.root(), &[".PHP"], &["node_modules"], 1).await?;
    let concurrent = hunt(fixture.root(), &["php"], &["node_modules"], 4).await?;
    assert_eq!(serial, concurrent);
    assert!(serial.errors.is_empty(), "{:?}", serial.errors);
    assert_eq!(serial.files_scanned, 9);
    assert_eq!(serial.files_outside_repositories, 1);
    assert_eq!(serial.extension_counts.get("php"), Some(&2));
    assert_eq!(serial.extension_counts.get("ts"), Some(&2));
    assert_eq!(serial.extension_counts.get("."), Some(&2));
    assert!(!serial.extension_counts.contains_key(""));
    assert!(
        serial
            .repositories
            .iter()
            .all(|repository| !repository.extension_counts.contains_key(""))
    );
    assert_eq!(serial.repositories.len(), 2);
    let parent = serial
        .repositories
        .iter()
        .find(|repository| repository.path == fs::canonicalize(&parent).unwrap())
        .unwrap();
    let nested = serial
        .repositories
        .iter()
        .find(|repository| repository.path == fs::canonicalize(&nested).unwrap())
        .unwrap();
    assert_eq!(parent.total_files, 5);
    assert_eq!(parent.matched_files, 1);
    assert_eq!(parent.extension_counts.get("ts"), Some(&1));
    let history = parent.history.as_ref().unwrap();
    assert_eq!(history.total_commits, 0);
    assert!(history.authors.is_empty());
    assert!(!history.is_shallow);
    assert_eq!(nested.total_files, 2);
    assert_eq!(nested.matched_files, 1);
    assert_eq!(parent.remotes.len(), 1);
    assert_eq!(parent.remotes[0].name, "origin");
    assert_eq!(
        parent.remotes[0].fetch_url.as_deref(),
        Some("https://example.invalid/team/php.git")
    );
    assert_eq!(
        parent.remotes[0].push_url.as_deref(),
        Some("git@example.invalid:team/php.git")
    );
    Ok(())
}

#[tokio::test]
async fn summarizes_exact_author_identities_and_authored_time_extrema() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/app.php", "php")?;
    fixture.write(
        "repo/.mailmap",
        "Canonical Author <canonical@example.invalid> Alice <alice@example.invalid>\n",
    )?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-03-20T10:00:00+02:00",
    )?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-01-15T12:30:00-05:00",
    )?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-03-20T03:00:00-05:00",
    )?;
    fixture.commit(
        &repository,
        "Alice",
        "other@example.invalid",
        "2024-02-01T00:00:00+00:00",
    )?;
    fixture.commit(
        &repository,
        "Bob",
        "bob@example.invalid",
        "2024-04-01T00:00:00+00:00",
    )?;

    let report = hunt(&repository, &[], &[], 2).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let history = report.repositories[0].history.as_ref().unwrap();
    assert_eq!(history.total_commits, 5);
    assert_eq!(history.authors.len(), 3);
    assert!(!history.is_shallow);
    let alice = history
        .authors
        .iter()
        .find(|author| author.name == "Alice" && author.email == "alice@example.invalid")
        .unwrap();
    assert_eq!(alice.commit_count, 3);
    assert_eq!(
        alice.first_commit_at,
        DateTime::parse_from_rfc3339("2024-01-15T17:30:00Z")?.with_timezone(&Utc)
    );
    assert_eq!(
        alice.last_commit_at,
        DateTime::parse_from_rfc3339("2024-03-20T08:00:00Z")?.with_timezone(&Utc)
    );
    let other_alice = history
        .authors
        .iter()
        .find(|author| author.name == "Alice" && author.email == "other@example.invalid")
        .unwrap();
    assert_eq!(other_alice.commit_count, 1);
    assert_eq!(other_alice.first_commit_at, other_alice.last_commit_at);
    assert!(history.authors.iter().any(|author| {
        author.name == "Bob" && author.email == "bob@example.invalid" && author.commit_count == 1
    }));
    Ok(())
}

#[tokio::test]
async fn includes_all_refs_and_detached_head_while_deduplicating_merged_ancestry() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/app.ts", "ts")?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-01-01T00:00:00+00:00",
    )?;
    fixture.git(&repository, &["checkout", "--quiet", "-b", "feature"])?;
    fixture.commit(
        &repository,
        "Bob",
        "bob@example.invalid",
        "2024-01-02T00:00:00+00:00",
    )?;
    fixture.git(&repository, &["branch", "feature-copy"])?;
    fixture.git(
        &repository,
        &["update-ref", "refs/remotes/origin/feature", "feature"],
    )?;
    fixture.git(
        &repository,
        &["tag", "--annotate", "feature-tag", "-m", "fixture tag"],
    )?;
    fixture.git(&repository, &["checkout", "--quiet", "main"])?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-01-03T00:00:00+00:00",
    )?;
    fixture.git(
        &repository,
        &[
            "merge",
            "--quiet",
            "--no-ff",
            "--no-gpg-sign",
            "--no-verify",
            "feature",
            "-m",
            "fixture merge",
        ],
    )?;
    fixture.git(&repository, &["checkout", "--quiet", "-b", "unmerged"])?;
    fixture.commit(
        &repository,
        "Branch Author",
        "branch@example.invalid",
        "2024-01-04T00:00:00+00:00",
    )?;
    fixture.git(&repository, &["checkout", "--quiet", "--detach", "main"])?;
    fixture.commit(
        &repository,
        "Detached Author",
        "detached@example.invalid",
        "2024-01-05T00:00:00+00:00",
    )?;

    let report = hunt(&repository, &[], &[], 4).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    let history = report.repositories[0].history.as_ref().unwrap();
    assert_eq!(history.total_commits, 6);
    assert_eq!(
        history
            .authors
            .iter()
            .map(|author| author.commit_count)
            .sum::<u64>(),
        6
    );
    for (name, expected) in [
        ("Alice", 2),
        ("Bob", 1),
        ("Fixture", 1),
        ("Branch Author", 1),
        ("Detached Author", 1),
    ] {
        assert_eq!(
            history
                .authors
                .iter()
                .find(|author| author.name == name)
                .unwrap()
                .commit_count,
            expected,
            "commit count for {name}"
        );
    }
    Ok(())
}

#[tokio::test]
async fn shallow_history_reports_only_locally_available_commits() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("origin")?;
    fixture.write("origin/app.php", "php")?;
    fixture.git(&repository, &["add", "app.php"])?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-01-01T00:00:00+00:00",
    )?;
    fixture.commit(
        &repository,
        "Bob",
        "bob@example.invalid",
        "2024-02-01T00:00:00+00:00",
    )?;
    // Declare the tip a shallow boundary while keeping its parent object. This
    // proves traversal respects Git's boundary even when older objects exist,
    // without spawning a local clone transport or shell on Windows.
    let tip = fs::read_to_string(repository.join(".git/refs/heads/main"))?;
    fixture.write("origin/.git/shallow", &tip)?;

    let report = hunt(&repository, &[], &[], 2).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.files_scanned, 1);
    let history = report.repositories[0].history.as_ref().unwrap();
    assert!(history.is_shallow);
    assert_eq!(history.total_commits, 1);
    assert_eq!(history.authors.len(), 1);
    assert_eq!(history.authors[0].name, "Bob");
    assert_eq!(history.authors[0].commit_count, 1);
    Ok(())
}

#[tokio::test]
async fn unavailable_reachable_commits_leave_file_inventory_usable_and_history_unknown()
-> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/app.php", "php")?;
    fixture.commit(
        &repository,
        "Alice",
        "alice@example.invalid",
        "2024-01-01T00:00:00+00:00",
    )?;
    fixture.write(
        "repo/.git/refs/heads/unavailable",
        "1111111111111111111111111111111111111111\n",
    )?;

    let report = hunt(&repository, &[], &[], 2).await?;
    assert!(!report.errors.is_empty());
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.repositories.len(), 1);
    assert_eq!(report.repositories[0].extension_counts.get("php"), Some(&1));
    assert!(report.repositories[0].history.is_none());
    Ok(())
}

#[tokio::test]
async fn a_nested_repository_scan_root_does_not_add_an_empty_ancestor() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.repository("parent")?;
    let nested = fixture.repository("parent/nested")?;
    fixture.write("parent/outside.php", "outside the scan")?;
    fixture.write("parent/nested/inside.ts", "inside the scan")?;

    let report = hunt(&nested, &[], &[], 2).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.repositories.len(), 1);
    assert_eq!(report.repositories[0].path, fs::canonicalize(nested)?);
    assert_eq!(report.repositories[0].total_files, 1);
    assert_eq!(report.repositories[0].matched_files, 1);
    Ok(())
}

#[tokio::test]
async fn scanning_a_subtree_retains_ancestor_identity_but_counts_only_that_subtree() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/outside.php", "outside the scan")?;
    fixture.write("repo/src/deep/inside.ts", "inside the scan")?;

    let report = hunt(&fixture.path("repo/src"), &[], &[], 2).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.repositories.len(), 1);
    assert_eq!(report.repositories[0].path, fs::canonicalize(repository)?);
    assert_eq!(report.repositories[0].total_files, 1);
    assert_eq!(report.files_outside_repositories, 0);
    assert_eq!(report.extension_counts.get("php"), None);
    Ok(())
}

#[tokio::test]
async fn a_broken_nested_repository_reports_an_issue_without_contaminating_its_parent() -> Result<()>
{
    let fixture = Fixture::new()?;
    let parent = fixture.repository("parent")?;
    fixture.repository("sibling")?;
    fixture.write("parent/good.php", "php")?;
    fixture.write("parent/broken/.git", "not a valid Git file")?;
    fixture.write("parent/broken/unknown.php", "unattributed")?;
    fixture.write("sibling/good.ts", "ts")?;

    let report = hunt(fixture.root(), &[], &[], 4).await?;
    assert_eq!(report.errors.len(), 1);
    assert_eq!(report.files_scanned, 3);
    assert_eq!(report.files_outside_repositories, 1);
    assert_eq!(report.repositories.len(), 2);
    assert_eq!(
        report
            .repositories
            .iter()
            .find(|repository| repository.path == fs::canonicalize(&parent).unwrap())
            .unwrap()
            .total_files,
        1
    );
    Ok(())
}

#[tokio::test]
async fn discovers_git_file_worktrees_and_reads_their_common_remote_configuration() -> Result<()> {
    let fixture = Fixture::new()?;
    let main = fixture.repository("main")?;
    fixture.git(&main, &["config", "user.name", "Fixture"])?;
    fixture.git(&main, &["config", "user.email", "fixture@example.invalid"])?;
    fixture.git(
        &main,
        &[
            "commit",
            "--quiet",
            "--allow-empty",
            "--no-gpg-sign",
            "--no-verify",
            "-m",
            "fixture",
        ],
    )?;
    fixture.git(
        &main,
        &[
            "config",
            "remote.origin.url",
            "https://example.invalid/worktree.git",
        ],
    )?;
    let worktree = fixture.path("linked");
    fixture.git(
        &main,
        &[
            "worktree",
            "add",
            "--quiet",
            "--detach",
            worktree.to_str().unwrap(),
        ],
    )?;
    fixture.write("main/main.php", "php")?;
    fixture.write("linked/linked.ts", "ts")?;

    let report = hunt(fixture.root(), &[], &[], 4).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.repositories.len(), 2);
    assert_eq!(report.files_scanned, 2);
    let linked = report
        .repositories
        .iter()
        .find(|repository| repository.path == fs::canonicalize(&worktree).unwrap())
        .unwrap();
    assert_eq!(linked.total_files, 1);
    assert_eq!(
        linked.remotes[0].fetch_url.as_deref(),
        Some("https://example.invalid/worktree.git")
    );
    Ok(())
}

#[tokio::test]
async fn publishes_complete_final_progress_and_rejects_invalid_roots_and_job_limits() -> Result<()>
{
    let fixture = Fixture::new()?;
    fixture.repository("repo")?;
    fixture.write("repo/app.php", "php")?;
    let (progress, receiver) = watch::channel(SourceHuntProgress::default());
    let report = CancellationToken::new()
        .with_async_scope(|cancellation| async move {
            SourceHuntRequest {
                root: fixture.root().to_owned(),
                extensions: Vec::new(),
                exclude_dirs: Vec::new(),
                jobs: 2,
                cancellation: cancellation.clone(),
                progress: Some(progress),
            }
            .await
        })
        .await?;
    let final_progress = receiver.borrow();
    assert_eq!(final_progress.files_scanned, report.files_scanned);
    assert_eq!(
        final_progress.directories_scanned,
        report.directories_scanned
    );
    assert_eq!(final_progress.extension_counts, report.extension_counts);
    assert_eq!(final_progress.repositories_found, 1);
    assert!(final_progress.active_paths.is_empty());
    drop(final_progress);

    let fixture = Fixture::new()?;
    fixture.write("file.txt", "not a directory")?;
    let repository = fixture.repository("repo")?;
    assert!(hunt(&fixture.path("missing"), &[], &[], 1).await.is_err());
    assert!(hunt(&fixture.path("file.txt"), &[], &[], 1).await.is_err());
    assert!(hunt(fixture.root(), &[], &[], 0).await.is_err());
    assert!(hunt(&repository.join(".git"), &[], &[], 1).await.is_err());
    assert!(
        hunt(&repository.join(".git/objects"), &[], &[], 1)
            .await
            .is_err()
    );
    Ok(())
}

#[tokio::test]
async fn cancellation_stops_a_scan_with_queued_descendants() -> Result<()> {
    let fixture = Fixture::new()?;
    fixture.repository("repo")?;
    for index in 0..300 {
        fixture.write(&format!("repo/dir{index}/code.php"), "php")?;
    }
    let token = CancellationToken::new();
    token
        .with_async_scope(|cancellation| async move {
            let (progress, mut receiver) = watch::channel(SourceHuntProgress::default());
            let hunt = SourceHuntRequest {
                root: fixture.root().to_owned(),
                extensions: Vec::new(),
                exclude_dirs: Vec::new(),
                jobs: 1,
                cancellation: cancellation.clone(),
                progress: Some(progress),
            }
            .into_future();
            tokio::pin!(hunt);
            tokio::select! {
                report = &mut hunt => eyre::bail!("Scan completed before a cancellation checkpoint: {report:?}"),
                changed = receiver.changed() => changed?,
            }
            cancellation.request_cancel("fixture cancellation");
            let result = hunt.await;
            assert!(result.is_err());
            Ok(())
        })
        .await
}

#[tokio::test]
async fn traverses_deep_directory_trees_without_recursive_futures() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    let mut deepest = repository.clone();
    for _ in 0..180 {
        deepest.push("d");
    }
    fs::create_dir_all(&deepest)?;
    fs::write(deepest.join("deep.TS"), "ts")?;

    let report = hunt(&repository, &["ts"], &[], 4).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.repositories[0].extension_counts.get("ts"), Some(&1));
    Ok(())
}

#[cfg(unix)]
#[tokio::test]
async fn does_not_follow_directory_links_or_link_cycles() -> Result<()> {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/inside.php", "php")?;
    fixture.write("outside/external.js", "outside")?;
    symlink(fixture.path("outside"), repository.join("linked"))?;
    symlink(&repository, repository.join("cycle"))?;
    symlink(repository.join("inside.php"), repository.join("linked.php"))?;

    let report = hunt(&repository, &[], &[], 4).await?;
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.repositories[0].total_files, 1);
    let explicit_root = hunt(&repository.join("linked"), &[], &[], 1).await?;
    assert_eq!(
        explicit_root.root,
        fs::canonicalize(fixture.path("outside"))?
    );
    assert_eq!(explicit_root.files_scanned, 1);
    assert_eq!(explicit_root.files_outside_repositories, 1);
    assert!(explicit_root.repositories.is_empty());
    fixture.write("outside/subdir/nested.ts", "ts")?;
    let aliased_parent = hunt(&repository.join("linked/subdir"), &[], &[], 1).await?;
    assert_eq!(
        aliased_parent.root,
        fs::canonicalize(fixture.path("outside/subdir"))?
    );
    assert_eq!(aliased_parent.files_scanned, 1);
    assert_eq!(aliased_parent.files_outside_repositories, 1);
    assert_eq!(aliased_parent.extension_counts.get("ts"), Some(&1));
    let metadata_alias = fixture.path("metadata-alias");
    symlink(repository.join(".git"), &metadata_alias)?;
    assert!(hunt(&metadata_alias, &[], &[], 1).await.is_err());
    Ok(())
}

#[cfg(windows)]
#[tokio::test]
async fn does_not_follow_junctions_or_junction_cycles() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/inside.php", "php")?;
    fixture.write("outside/external.js", "outside")?;
    for (link, target) in [
        (repository.join("linked"), fixture.path("outside")),
        (repository.join("cycle"), repository.clone()),
    ] {
        let output = Command::new("cmd")
            .args(["/C", "mklink", "/J"])
            .arg(&link)
            .arg(&target)
            .output()?;
        eyre::ensure!(
            output.status.success(),
            "Creating local fixture junction failed"
        );
    }
    let report = hunt(&repository, &[], &[], 4).await?;
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.repositories[0].total_files, 1);
    let explicit_root = hunt(&repository.join("linked"), &[], &[], 1).await?;
    assert_eq!(
        explicit_root.root,
        fs::canonicalize(fixture.path("outside"))?
    );
    assert_eq!(explicit_root.files_scanned, 1);
    assert_eq!(explicit_root.files_outside_repositories, 1);
    assert!(explicit_root.repositories.is_empty());
    fixture.write("outside/subdir/nested.ts", "ts")?;
    let aliased_parent = hunt(&repository.join("linked/subdir"), &[], &[], 1).await?;
    assert_eq!(
        aliased_parent.root,
        fs::canonicalize(fixture.path("outside/subdir"))?
    );
    assert_eq!(aliased_parent.files_scanned, 1);
    assert_eq!(aliased_parent.files_outside_repositories, 1);
    assert_eq!(aliased_parent.extension_counts.get("ts"), Some(&1));
    let metadata_alias = fixture.path("metadata-alias");
    let output = Command::new("cmd")
        .args(["/C", "mklink", "/J"])
        .arg(&metadata_alias)
        .arg(repository.join(".git"))
        .output()?;
    eyre::ensure!(
        output.status.success(),
        "Creating local fixture junction failed"
    );
    assert!(hunt(&metadata_alias, &[], &[], 1).await.is_err());
    Ok(())
}

#[cfg(windows)]
#[tokio::test]
async fn skips_git_metadata_when_its_name_uses_different_case() -> Result<()> {
    let fixture = Fixture::new()?;
    let repository = fixture.repository("repo")?;
    fixture.write("repo/inside.php", "php")?;
    fs::rename(repository.join(".git"), repository.join("git-temporary"))?;
    fs::rename(repository.join("git-temporary"), repository.join(".GIT"))?;

    let report = hunt(&repository, &[], &[], 2).await?;
    assert!(report.errors.is_empty(), "{:?}", report.errors);
    assert_eq!(report.files_scanned, 1);
    assert_eq!(report.repositories.len(), 1);
    assert!(
        hunt(&repository.join(".GIT/objects"), &[], &[], 1)
            .await
            .is_err()
    );
    Ok(())
}
