use cloud_terrastodon_software::list_software_counts;
use eyre::Result;
use teamy_cancellation::CancellationToken;

#[derive(facet::Facet, Debug, Clone)]
pub struct SoftwareListArgs {}

impl SoftwareListArgs {
    pub async fn invoke(self, cancellation_token: &CancellationToken) -> Result<()> {
        let summaries = tokio::task::spawn_blocking({
            let cancellation_token = cancellation_token.clone();
            move || list_software_counts(&cancellation_token)
        })
        .await??;
        let query_width = summaries
            .iter()
            .map(|summary| summary.query.len())
            .max()
            .unwrap_or("query".len())
            .max("query".len());

        println!("{:<query_width$} count", "query");
        for summary in summaries {
            println!("{:<query_width$} {}", summary.query, summary.result_count);
        }

        Ok(())
    }
}
