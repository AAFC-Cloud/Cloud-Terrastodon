use clap::Args;
use cloud_terrastodon_software::list_software_counts_with_cancel;
use eyre::Result;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

#[derive(Args, Debug, Clone)]
pub struct SoftwareListArgs {}

impl SoftwareListArgs {
    pub async fn invoke(self) -> Result<()> {
        let cancel = Arc::new(AtomicBool::new(false));
        let cancel_signal = {
            let cancel = Arc::clone(&cancel);
            tokio::spawn(async move {
                let _ = tokio::signal::ctrl_c().await;
                cancel.store(true, Ordering::Relaxed);
            })
        };
        let summaries = tokio::task::spawn_blocking({
            let cancel = Arc::clone(&cancel);
            move || list_software_counts_with_cancel(Some(cancel.as_ref()))
        })
        .await??;
        cancel_signal.abort();
        let query_width = summaries
            .iter()
            .map(|summary| summary.query.len())
            .max()
            .unwrap_or("query".len())
            .max("query".len());

        println!("{:<query_width$} count", "query");
        for summary in summaries {
            println!(
                "{:<query_width$} {}",
                summary.query, summary.result_count
            );
        }

        Ok(())
    }
}
