use crate::AzureBatchRequest;
use crate::AzureBatchRequestEntry;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_types::ArbitraryJson;
use cloud_terrastodon_azure_types::AsScope;
use cloud_terrastodon_azure_types::Metrics;
use cloud_terrastodon_azure_types::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_pathing::AppDir;
use cloud_terrastodon_pathing::Existy;
use eyre::WrapErr;
use jiff::Timestamp;
use jiff::ZonedDateTime;
use jiff::civil::Date;
use jiff::fmt::temporal::DateTimePrinter;
use jiff::tz::TimeZone;
use tempfile::Builder;
use tokio::io::AsyncWriteExt;
use tracing::info;

pub async fn fetch_metrics(
    authentication: &AzureBearerAuthenticationSelection,
    resource_ids: impl IntoIterator<Item = impl AsScope>,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<Metrics> {
    let mut batch_request = AzureBatchRequest::new(*authentication).cancellation_safe();
    for id in resource_ids {
        let id = id.as_scope().expanded_form();
        let end = Timestamp::now();
        let beginning_date = Date::new(end.to_zoned_datetime(TimeZone::UTC).year() - 7, 1, 1)?;
        let begin = TimeZone::UTC
            .to_timestamp(beginning_date.at(0, 0, 0, 0))
            .wrap_err("constructing the start of the metrics range")?;
        let printer = DateTimePrinter::new().precision(Some(0));
        let url = format!(
            // "{}/providers/Microsoft.Insights/metrics?timespan={}/{}&interval=FULL&metricnames=UsedCapacity&aggregation=maximum&metricNamespace=Microsoft.Storage/storageAccounts&api-version=2019-07-01",
            "{}/providers/Microsoft.Insights/metrics?timespan={}/{}&interval=PT1H&metricnames=UsedCapacity&aggregation=maximum&metricNamespace=Microsoft.Storage/storageAccounts&api-version=2019-07-01",
            id,
            printer.timestamp_to_string(&begin),
            printer.timestamp_to_string(&end)
        );
        batch_request
            .requests
            .push(AzureBatchRequestEntry::new_get(url));
    }
    info!(
        "Fetching metrics for {} resources",
        batch_request.requests.len()
    );
    let resp = batch_request.invoke::<ArbitraryJson>(executor).await?;
    // dbg!(resp);
    let dir = AppDir::Temp.as_path_buf();
    dir.ensure_dir_exists().await?;
    let (file, file_path) = Builder::new()
        .prefix(&ZonedDateTime::now().strftime("%Y%m%d_%H%M%S_").to_string())
        .suffix(".json")
        .tempfile_in(dir)?
        .keep()?;
    let mut file = tokio::fs::File::from_std(file);
    let json = facet_json::to_string_pretty(&resp).wrap_err("Serializing metrics for preview")?;
    file.write_all(json.as_bytes()).await?;
    println!("Dumped info for previewing to {}", file_path.display());

    Ok(Metrics {})
}
