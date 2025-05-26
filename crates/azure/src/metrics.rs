use chrono::Datelike;
use chrono::Local;
use chrono::SecondsFormat;
use chrono::TimeZone;
use cloud_terrastodon_azure_types::prelude::AsScope;
use cloud_terrastodon_azure_types::prelude::Metrics;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_pathing::AppDir;
use eyre::OptionExt;
use tempfile::Builder;
use tokio::io::AsyncWriteExt;
use tracing::info;

use crate::prelude::BatchRequest;
use crate::prelude::BatchRequestEntry;

pub async fn fetch_metrics(
    resource_ids: impl IntoIterator<Item = impl AsScope>,
) -> eyre::Result<Metrics> {
    let mut batch_request = BatchRequest::new();
    for id in resource_ids {
        let id = id.as_scope().expanded_form();
        let end = chrono::Utc::now();
        let begin = chrono::Utc
            .with_ymd_and_hms(end.year() - 7, 1, 1, 0, 0, 0)
            .earliest()
            .ok_or_eyre("Failed to construct starting data")?;
        let url = format!(
            // "{}/providers/Microsoft.Insights/metrics?timespan={}/{}&interval=FULL&metricnames=UsedCapacity&aggregation=maximum&metricNamespace=Microsoft.Storage/storageAccounts&api-version=2019-07-01",
            "{}/providers/Microsoft.Insights/metrics?timespan={}/{}&interval=PT1H&metricnames=UsedCapacity&aggregation=maximum&metricNamespace=Microsoft.Storage/storageAccounts&api-version=2019-07-01",
            id,
            begin.to_rfc3339_opts(SecondsFormat::Secs, true),
            end.to_rfc3339_opts(SecondsFormat::Secs, true)
        );
        batch_request.requests.push(BatchRequestEntry::new_get(url));
    }
    info!(
        "Fetching metrics for {} resources",
        batch_request.requests.len()
    );
    let resp = batch_request.invoke::<serde_json::Value>().await?;
    // dbg!(resp);
    let dir = AppDir::Temp.as_path_buf();
    let (file, file_path) = Builder::new()
        .prefix(Local::now().format("%Y%m%d_%H%M%S_").to_string().as_str())
        .suffix(".json")
        .tempfile_in(dir)?
        .keep()?;
    let mut file = tokio::fs::File::from_std(file);
    file.write_all(serde_json::to_string_pretty(&resp)?.as_bytes())
        .await?;
    println!("Dumped info for previewing to {}", file_path.display());

    Ok(Metrics {})
}

#[cfg(test)]
mod tests {
    use crate::prelude::fetch_all_storage_accounts;

    use super::*;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let resources = fetch_all_storage_accounts().await?;
        let resources = resources.iter().take(3);
        let metrics = fetch_metrics(resources).await?;
        dbg!(metrics);
        Ok(())
    }
}
