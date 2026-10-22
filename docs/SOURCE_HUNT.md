# Source hunt

`ct source hunt PATH` scans a directory tree and summarizes file extensions and
commit authors by Git repository. It finds nested repositories and reads their
configured remotes and commit history with `gix` locally, without contacting a
remote or reading source-file contents. The command accepts linked Git worktrees
as well as ordinary repositories.

```pwsh
ct source hunt 'C:\Users\phillipsdo\source\repos'
ct source hunt 'C:\Users\phillipsdo\source\repos' --extension php --extension js --extension ts
```

Repeat `--extension` to select repositories containing any requested extension.
Each selected repository still includes counts for **all** its file extensions;
`matched_files` counts only files matching the requested extensions. Without an
extension filter, all discovered repositories are included and all their files
match.

Extensions use the final filename suffix, folded to lowercase: `app.JS` counts
as `js`, and `types.d.ts` counts as `ts`. The extension key `"."` represents
files without an extension. PHP, JS, and TS counts identify filename extensions;
they do not establish use of a PHP runtime or Node.js. The command does not infer
runtime or package-manager use from `package.json`.

## What is scanned

The scan includes hidden, untracked, and Git-ignored files. The explicitly
supplied root resolves symbolic links and directory junctions to their target
once. Symbolic links and junctions encountered inside the scanned tree are not
followed. Git metadata named `.git` is always skipped. There are no other default
directory exclusions. Exclude generated or dependency directories explicitly
when they should not contribute to counts:

```pwsh
ct source hunt 'C:\Users\phillipsdo\source\repos' --extension php --extension js --extension ts --exclude-dir node_modules --exclude-dir vendor
```

Repeat `--exclude-dir` to skip directories with those names throughout the tree.
Files belong to their closest enclosing repository, so a nested repository's
files are counted separately from its parent. Starting inside a repository scans
only the requested subtree, while the report identifies the owning repository's
root. Files encountered outside repositories are counted separately.

## Commit authors

Each repository includes an author summary with the number of commits and the
first and last commit datetimes for each author. Authors are identified by their
exact recorded name and email pair; Git mailmap aliases are not applied. The
datetimes use the commit's author date, rather than its committer date, and are
reported in UTC. First and last are the minimum and maximum author datetimes,
regardless of commit traversal order.

The summary covers commits reachable from `HEAD` and all local references,
including tags and remote-tracking references already present locally. Each
commit is counted once, even when several references reach it. No fetch is
performed. Only available local history contributes; shallow repositories are
flagged with `is_shallow`, and their summaries may omit earlier commits.

Author summaries cover the whole repository, including when the scan starts in
a subtree, excludes directories, or filters by extension. They describe who
authored the repository's commits, not who authored files with particular
extensions. A repository with no commits has an empty author list and zero
commits. If its history cannot be read, `history` is null and `errors` explains
the failure.

## Concurrency and progress

Directory analysis runs concurrently with a bounded number of jobs. `--jobs`
accepts a positive integer; its default is four times the available parallelism,
clamped to 8 through 64 jobs.

When stderr is a terminal, a Ratatui progress display shows current paths,
repositories with the most matching files, and extension totals. Final output
uses stdout, so the progress display can remain visible while results go through
a pipeline. Disable it with `--no-progress`:

```pwsh
ct source hunt 'C:\Users\phillipsdo\source\repos' --jobs 32 --no-progress --output-format json
```

## PowerShell output

The global `--output-format json` selects JSON explicitly. Output also defaults
to JSON when stdout is redirected or piped; terminal output defaults to text.
`--output-format facet-pretty` shows the typed report with Facet's pretty printer.

```pwsh
$report = ct source hunt 'C:\Users\phillipsdo\source\repos' --extension php --extension js --extension ts --output-format json | ConvertFrom-Json
$report.repositories |
    Sort-Object matched_files -Descending |
    Select-Object path, matched_files,
        @{n='php'; e={$_.extension_counts.php}},
        @{n='js'; e={$_.extension_counts.js}},
        @{n='ts'; e={$_.extension_counts.ts}}
```

An extension absent from a repository has no entry in its `extension_counts`
map. PowerShell displays that missing property as an empty value in this example.

To flatten author summaries into rows and sort by commit count:

```pwsh
$report.repositories |
    ForEach-Object {
        $repo = $_
        $_.history.authors |
            Select-Object @{n='repo'; e={$repo.path}}, name, email,
                commit_count, first_commit_at, last_commit_at
    } |
    Sort-Object commit_count -Descending
```

The report contains:

| Field | Meaning |
| --- | --- |
| `root` | Canonical resolved path of the requested scan root. |
| `repositories` | Repositories passing the extension filter, with `path`, `remotes`, `history`, `extension_counts`, `total_files`, and `matched_files`. |
| `repositories[].remotes` | Configured remote records with `name`, `fetch_url`, and `push_url`, read locally. |
| `repositories[].history` | Available local commit history with `authors`, `total_commits`, and `is_shallow`; null when history could not be read. |
| `repositories[].history.total_commits` | Number of distinct reachable commits across `HEAD` and local references. |
| `repositories[].history.is_shallow` | Whether the repository is shallow and may omit earlier commits. |
| `repositories[].history.authors` | Author records with `name`, `email`, `commit_count`, `first_commit_at`, and `last_commit_at`; datetimes are UTC author dates. |
| `repositories[].extension_counts` | JSON map from normalized extension to file count for the scanned part of that repository. |
| `extension_counts` | Aggregate counts across all scanned files, including files outside repositories and repositories omitted by the extension filter. |
| `files_scanned` | Number of files encountered during the scan. |
| `files_outside_repositories` | Number of scanned files without an owning repository. |
| `directories_scanned` | Number of directories scanned. |
| `errors` | Scan failures retained in the report, such as unreadable directories or repository metadata. |

File counts cover only the scanned subtree and respect directory exclusions.
Commit author summaries cover the whole repository's available local history.
Review `errors` when interpreting an incomplete scan.
