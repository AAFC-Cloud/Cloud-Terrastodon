use super::output::write_hunt_report;
use super::source_hunt_progress_terminal::SourceHuntProgressTerminal;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_source::SourceFileExtension;
use cloud_terrastodon_source::SourceHuntProgress;
use cloud_terrastodon_source::SourceHuntRequest;
use eyre::Result;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::MissedTickBehavior;

/// Inventory working-tree extensions, configured Git remotes, and commit authors.
#[derive(Debug, Clone, facet::Facet)]
pub struct SourceHuntArgs {
    /// Directory tree to search. Starting inside a repository scans only this subtree.
    #[facet(figue::positional)]
    pub path: PathBuf,
    /// Select repositories containing any of these extensions; repeat for multiple.
    /// Accepts php or .PHP. Selected repositories retain all extension counts.
    #[facet(figue::named, default)]
    pub extension: Vec<SourceFileExtension>,
    /// Skip directories with this exact name, in addition to .git; repeat as needed.
    #[facet(figue::named, default)]
    pub exclude_dir: Vec<String>,
    /// Maximum concurrent directory scans. Defaults to four per CPU, bounded to 8..64.
    #[facet(figue::named, default = default_jobs())]
    pub jobs: usize,
    /// Disable the live stderr progress display.
    #[facet(figue::named, default = false)]
    pub no_progress: bool,
}

fn default_jobs() -> usize {
    std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(2)
        .saturating_mul(4)
        .clamp(8, 64)
}

impl SourceHuntArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        eyre::ensure!(self.jobs > 0, "--jobs must be greater than zero");
        let mut terminal = if !self.no_progress && SourceHuntProgressTerminal::is_supported() {
            Some(SourceHuntProgressTerminal::new().await?)
        } else {
            None
        };
        let (progress, updates) = watch::channel(SourceHuntProgress::default());
        let mut hunt = Box::pin(
            SourceHuntRequest {
                root: self.path,
                extensions: self.extension,
                exclude_dirs: self.exclude_dir,
                jobs: self.jobs,
                cancellation: cancellation.clone(),
                progress: terminal.as_ref().map(|_| progress),
            }
            .into_future(),
        );
        let mut ticks = tokio::time::interval(Duration::from_millis(100));
        ticks.set_missed_tick_behavior(MissedTickBehavior::Skip);
        let result = loop {
            tokio::select! {
                result = &mut hunt => break result,
                _ = ticks.tick(), if terminal.is_some() => {
                    let drawing = match terminal.as_mut() {
                        Some(terminal) => terminal.draw(&updates.borrow()),
                        None => Err(eyre::eyre!("progress tick requires an active terminal")),
                    };
                    if let Err(error) = drawing {
                        // A failed progress display must not abandon directory workers.
                        if let Some(terminal) = terminal.take()
                            && let Err(restore_error) = terminal.finish().await
                        {
                            tracing::warn!(%restore_error, "Restoring source hunt progress failed");
                        }
                        tracing::warn!(%error, "Source hunt progress disabled; inventory continues");
                    }
                }
            }
        };
        if let Some(terminal) = terminal {
            terminal.finish().await?;
        }
        Ok(CliOutput::facet_with_text(result?, write_hunt_report))
    }
}

cloud_terrastodon_registry::register_thing!(SourceHuntArgs);
