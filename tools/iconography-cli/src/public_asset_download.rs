use cloud_terrastodon_app::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use reqwest::Client;
use url::Url;

/// Download a public GET response without consulting service credentials.
///
/// The eight MiB body limit is a library safety bound for icon assets and their
/// discovery documents, rather than a guarantee made by an upstream service.
/// The caller supplies its configured client timeout and owns interruption of
/// the executing future; this serial read uses its supplied scope for progress
/// checkpoints.
pub async fn download(client: &Client, url: &Url, scope: &CancellationScope) -> Result<Vec<u8>> {
    const MAX_BODY_BYTES: usize = 8 * 1024 * 1024;

    let mut response = client
        .get(url.clone())
        .send()
        .await
        .wrap_err_with(|| format!("Downloading public iconography source {url} failed"))?
        .error_for_status()
        .wrap_err_with(|| format!("Public iconography source {url} returned an error"))?;
    scope.bail_if_cancelled()?;
    ensure!(
        response
            .content_length()
            .is_none_or(|length| length <= MAX_BODY_BYTES as u64),
        "Public iconography source {url} exceeds the {MAX_BODY_BYTES}-byte body limit"
    );

    let mut bytes = Vec::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .wrap_err_with(|| format!("Reading public iconography source {url} failed"))?
    {
        scope.bail_if_cancelled()?;
        ensure!(
            chunk.len() <= MAX_BODY_BYTES - bytes.len(),
            "Public iconography source {url} exceeds the {MAX_BODY_BYTES}-byte body limit"
        );
        bytes.extend_from_slice(&chunk);
    }
    scope.bail_if_cancelled()?;
    Ok(bytes)
}
