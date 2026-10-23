use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_source::SourceHuntReport;
use color_eyre::owo_colors::OwoColorize;
use color_eyre::owo_colors::Stream;
use eyre::Result;
use std::io::Write;

pub(super) fn write_hunt_report(
    report: &SourceHuntReport,
    writer: &mut dyn Write,
    cancellation: &CancellationScope,
) -> Result<()> {
    writeln!(
        writer,
        "{}: {} repositories, {} files, {} directories",
        "Source hunt".if_supports_color(Stream::Stdout, |text| text.bold()),
        report.repositories.len(),
        report.files_scanned,
        report.directories_scanned,
    )?;
    if report.files_outside_repositories > 0 {
        writeln!(
            writer,
            "{} files outside repositories",
            report.files_outside_repositories
        )?;
    }
    for repo in &report.repositories {
        cancellation.bail_if_cancelled()?;
        writeln!(
            writer,
            "\n{}  ({} matched / {} files)",
            terminal_text(&repo.path.to_string_lossy())
                .if_supports_color(Stream::Stdout, |text| text.cyan()),
            repo.matched_files,
            repo.total_files,
        )?;
        for remote in &repo.remotes {
            writeln!(
                writer,
                "  {}: {}",
                terminal_text(&remote.name),
                terminal_text(remote.fetch_url.as_deref().unwrap_or("(no fetch URL)")),
            )?;
            if let Some(push_url) = &remote.push_url
                && Some(push_url) != remote.fetch_url.as_ref()
            {
                writeln!(writer, "    push: {}", terminal_text(push_url))?;
            }
        }
        if let Some(history) = &repo.history {
            writeln!(
                writer,
                "  Commit history: {} commits, {} authors",
                history.total_commits,
                history.authors.len(),
            )?;
            if history.is_shallow {
                writeln!(
                    writer,
                    "    Shallow repository: only available local history is included."
                )?;
            }
            for author in &history.authors {
                cancellation.bail_if_cancelled()?;
                writeln!(
                    writer,
                    "    {} <{}>: {} commits, first {}, last {}",
                    terminal_text(&author.name),
                    terminal_text(&author.email),
                    author.commit_count,
                    author.first_commit_at.to_rfc3339(),
                    author.last_commit_at.to_rfc3339(),
                )?;
            }
        } else {
            writeln!(writer, "  Commit history unavailable")?;
        }
        let mut counts: Vec<_> = repo.extension_counts.iter().collect();
        counts.sort_by(|(left_ext, left_count), (right_ext, right_count)| {
            right_count
                .cmp(left_count)
                .then_with(|| left_ext.cmp(right_ext))
        });
        for (extension, count) in counts {
            writeln!(
                writer,
                "  {:>8}  {}",
                count,
                if extension == "." {
                    "(no extension)".to_owned()
                } else {
                    format!(".{}", terminal_text(extension))
                },
            )?;
        }
    }
    if !report.errors.is_empty() {
        writeln!(
            writer,
            "\n{} scan issues (results may be incomplete):",
            report.errors.len()
        )?;
        for issue in &report.errors {
            cancellation.bail_if_cancelled()?;
            writeln!(
                writer,
                "  {}: {}",
                terminal_text(&issue.path.to_string_lossy()),
                terminal_text(&issue.message),
            )?;
        }
    }
    Ok(())
}

fn terminal_text(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_control() {
                character.escape_default().to_string()
            } else {
                character.to_string()
            }
        })
        .collect()
}
