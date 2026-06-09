use clap::Args;
use cloud_terrastodon_software::list_software_counts;
use eyre::Result;

#[derive(Args, Debug, Clone)]
pub struct SoftwareListArgs {}

impl SoftwareListArgs {
    pub async fn invoke(self) -> Result<()> {
        let summaries = tokio::task::spawn_blocking(list_software_counts).await??;
        let pattern_width = summaries
            .iter()
            .map(|summary| summary.pattern.len())
            .max()
            .unwrap_or("pattern".len())
            .max("pattern".len());

        println!("{:<pattern_width$} count", "pattern");
        for summary in summaries {
            println!(
                "{:<pattern_width$} {}",
                summary.pattern, summary.result_count
            );
        }

        Ok(())
    }
}
