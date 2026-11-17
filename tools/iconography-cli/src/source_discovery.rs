use crate::GitHubBranchReference;
use crate::IconCatalogueEntry;
use crate::IconFamily;
use crate::public_asset_download::download;
use cloud_terrastodon_app::CancellationScope;
use eyre::Result;
use eyre::WrapErr;
use eyre::bail;
use eyre::ensure;
use reqwest::Client;
use std::collections::BTreeMap;
use url::Url;

/// Resolve the existing catalogue identities against public upstream sources.
///
/// Azure filenames remain tied to the Portal extraction repository's
/// [README](https://github.com/maskati/azure-icons#azure-icons); its
/// [extractor workflow](https://github.com/maskati/azure-icons/blob/main/.github/workflows/download-azure-icons.yml)
/// records how those files are derived from anonymous Portal assets. The
/// [GitHub reference endpoint](https://docs.github.com/en/rest/git/refs#get-a-reference)
/// selects the commit used for revision-pinned SVG URLs.
///
/// Azure DevOps hashes come from the anonymous
/// [status page](https://status.dev.azure.com/_history). Its embedded asset map
/// is observed website metadata, rather than a documented Azure DevOps REST
/// contract. Missing logical identities fail discovery. An explicit seed whose
/// discovery URL equals its source URL may retain that exact URL.
pub async fn resolve_sources(
    family: IconFamily,
    entries: &[IconCatalogueEntry],
    client: &Client,
    scope: &CancellationScope,
) -> Result<BTreeMap<String, Url>> {
    if entries.is_empty() {
        return Ok(BTreeMap::new());
    }

    match family {
        IconFamily::Azure => resolve_azure_sources(entries, client, scope).await,
        IconFamily::AzureDevOps => resolve_azure_devops_sources(entries, client, scope).await,
    }
}

async fn resolve_azure_sources(
    entries: &[IconCatalogueEntry],
    client: &Client,
    scope: &CancellationScope,
) -> Result<BTreeMap<String, Url>> {
    // GitHub documents this read-only endpoint for public, unauthenticated use:
    // https://docs.github.com/en/rest/git/refs#get-a-reference
    let reference_url =
        Url::parse("https://api.github.com/repos/maskati/azure-icons/git/ref/heads/main")?;
    let reference_body = download(client, &reference_url, scope).await?;
    let reference: GitHubBranchReference =
        facet_json::from_str(std::str::from_utf8(&reference_body)?)
            .wrap_err("Decoding the Azure icon repository's main reference failed")?;
    ensure!(
        reference.reference == "refs/heads/main" && reference.object.kind == "commit",
        "The Azure icon repository's main reference did not select a branch commit"
    );
    ensure!(
        reference.object.sha.len() == 40
            && reference
                .object
                .sha
                .bytes()
                .all(|byte| byte.is_ascii_hexdigit()),
        "The Azure icon repository returned an invalid commit SHA"
    );
    scope.bail_if_cancelled()?;

    let repository_url = Url::parse(&format!(
        "https://raw.githubusercontent.com/maskati/azure-icons/{}/",
        reference.object.sha,
    ))?;
    let readme_body = download(client, &repository_url.join("README.md")?, scope).await?;
    let readme = std::str::from_utf8(&readme_body)
        .wrap_err("The Azure icon repository README was not UTF-8")?;
    scope.bail_if_cancelled()?;

    let mut resolved = BTreeMap::new();
    for entry in entries {
        scope.bail_if_cancelled()?;
        ensure!(
            entry.logical_name.starts_with("svg/")
                && entry.logical_name.ends_with(".svg")
                && entry
                    .logical_name
                    .split('/')
                    .all(|component| !component.is_empty()
                        && component != "."
                        && component != ".."),
            "Azure icon logical name is not a repository SVG path: {}",
            entry.logical_name
        );
        ensure!(
            readme.contains(&format!("]({})", entry.logical_name)),
            "Azure icon {} is absent from the repository's README mapping",
            entry.logical_name
        );
        let mut source_url = repository_url.clone();
        source_url
            .path_segments_mut()
            .map_err(|()| {
                eyre::eyre!("The Azure icon repository URL cannot contain path segments")
            })?
            .pop_if_empty()
            .extend(entry.logical_name.split('/'));
        ensure!(
            resolved
                .insert(entry.logical_name.clone(), source_url)
                .is_none(),
            "Repeated Azure icon logical name: {}",
            entry.logical_name
        );
    }
    Ok(resolved)
}

async fn resolve_azure_devops_sources(
    entries: &[IconCatalogueEntry],
    client: &Client,
    scope: &CancellationScope,
) -> Result<BTreeMap<String, Url>> {
    let status_url = Url::parse("https://status.dev.azure.com/_history")?;
    let status_body = download(client, &status_url, scope).await?;
    let assets = parse_asset_map(
        std::str::from_utf8(&status_body)
            .wrap_err("The Azure DevOps public status page was not UTF-8")?,
    )?;
    scope.bail_if_cancelled()?;

    let mut resolved = BTreeMap::new();
    for entry in entries {
        scope.bail_if_cancelled()?;
        let source_url = match assets.get(&entry.logical_name) {
            Some(source_url) => Url::parse(source_url).wrap_err_with(|| {
                format!(
                    "Azure DevOps asset {} has an invalid URL",
                    entry.logical_name
                )
            })?,
            None if entry.discovery_url == entry.source_url => entry.source_url.clone(),
            None => bail!(
                "Azure DevOps asset {} is absent from the public status page's asset map",
                entry.logical_name
            ),
        };
        ensure!(
            resolved
                .insert(entry.logical_name.clone(), source_url)
                .is_none(),
            "Repeated Azure DevOps icon logical name: {}",
            entry.logical_name
        );
    }
    Ok(resolved)
}

/// Decode only the explicitly open logical-name map embedded in the status page.
fn parse_asset_map(document: &str) -> Result<BTreeMap<String, String>> {
    facet_json::from_str(asset_map_json(document)?)
        .wrap_err("Decoding the Azure DevOps public asset map failed")
}

/// Find the object's boundary while respecting quoted braces and JSON escapes.
fn asset_map_json(document: &str) -> Result<&str> {
    const MARKER: &str = "\"_assets\"";

    for (marker_start, _) in document.match_indices(MARKER) {
        let after_marker = marker_start + MARKER.len();
        let remainder = document[after_marker..].trim_start();
        let Some(remainder) = remainder.strip_prefix(':') else {
            continue;
        };
        let remainder = remainder.trim_start();
        ensure!(
            remainder.starts_with('{'),
            "The Azure DevOps public asset map is not a JSON object"
        );
        let start = document.len() - remainder.len();
        let mut depth = 0usize;
        let mut quoted = false;
        let mut escaped = false;
        for (offset, byte) in document.as_bytes()[start..].iter().copied().enumerate() {
            if quoted {
                if escaped {
                    escaped = false;
                } else if byte == b'\\' {
                    escaped = true;
                } else if byte == b'"' {
                    quoted = false;
                }
            } else {
                match byte {
                    b'"' => quoted = true,
                    b'{' => depth += 1,
                    b'}' => {
                        depth -= 1;
                        if depth == 0 {
                            return Ok(&document[start..=start + offset]);
                        }
                    }
                    _ => {}
                }
            }
        }
        bail!("The Azure DevOps public asset map has an unterminated JSON object");
    }

    bail!("The Azure DevOps public status page has no asset map")
}

#[cfg(test)]
mod tests {
    use super::parse_asset_map;

    #[test]
    fn asset_map_boundary_respects_escaped_quotes_and_braces() -> eyre::Result<()> {
        let assets = parse_asset_map(
            r#"<script>{"before":"_assets","shared":{"_assets":{"quoted":"a\\\"} {b","brace":"{asset}.svg"}},"after":42}</script>"#,
        )?;
        assert_eq!(assets.get("quoted").map(String::as_str), Some("a\\\"} {b"));
        assert_eq!(assets.get("brace").map(String::as_str), Some("{asset}.svg"));
        Ok(())
    }

    #[test]
    fn asset_map_requires_a_complete_object() {
        for document in [
            r#"<script>{"other":{}}</script>"#,
            r#"<script>{"_assets":[]}</script>"#,
            r#"<script>{"_assets":{"asset":"unterminated}</script>"#,
            r#"<script>{"_assets":{"asset":"value"</script>"#,
        ] {
            assert!(parse_asset_map(document).is_err(), "{document}");
        }
    }
}
