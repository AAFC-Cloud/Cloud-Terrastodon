use crate::GiteaInstanceUrl;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::GiteaSearchResults;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use eyre::Context;
use eyre::Result;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

pub const GITEA_UNCACHED_DELAY: Duration = Duration::from_millis(500);
pub const GITEA_PAGE_SIZE: usize = 50;

pub async fn gitea_api_get<T: DeserializeOwned + Send + 'static>(
    tenant: &GiteaInstanceUrl,
    endpoint: &str,
    cache_key: Option<CacheKey>,
) -> Result<T> {
    let mut cmd = CommandBuilder::new(CommandKind::Gitea);
    cmd.args(["api", tenant.api_url(endpoint).as_str()]);
    cmd.use_cache(cache_key);
    cmd.run_polite(GITEA_UNCACHED_DELAY).await
}

pub async fn gitea_api_get_best_effort<T: DeserializeOwned + Send + 'static>(
    tenant: &GiteaInstanceUrl,
    endpoint: &str,
    cache_key: Option<CacheKey>,
) -> Result<Option<T>> {
    let mut cmd = CommandBuilder::new(CommandKind::Gitea);
    cmd.args(["api", tenant.api_url(endpoint).as_str()]);
    cmd.use_cache(cache_key);
    match cmd.run_raw_polite(GITEA_UNCACHED_DELAY).await {
        Ok(output) => Ok(Some(parse_command_output::<T>(&output)?)),
        Err(error) => {
            tracing::debug!(%error, endpoint, "Skipping Gitea endpoint after best-effort failure");
            Ok(None)
        }
    }
}

pub async fn gitea_api_get_paged<T>(
    tenant: &GiteaInstanceUrl,
    cache_root: CacheKey,
    endpoint_builder: impl Fn(usize, usize) -> String,
) -> Result<Vec<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    let mut items = Vec::new();
    let mut page = 1usize;
    loop {
        let page_cache_key = CacheKey::new(cache_root.path.join("pages").join(page.to_string()));
        let page_items: Vec<T> = gitea_api_get(
            tenant,
            &endpoint_builder(page, GITEA_PAGE_SIZE),
            Some(page_cache_key),
        )
        .await?;
        let count = page_items.len();
        items.extend(page_items);
        if count < GITEA_PAGE_SIZE {
            break;
        }
        page += 1;
    }
    Ok(items)
}

pub async fn gitea_api_get_search_paged<T>(
    tenant: &GiteaInstanceUrl,
    cache_root: CacheKey,
    endpoint_builder: impl Fn(usize, usize) -> String,
) -> Result<Vec<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    let mut items = Vec::new();
    let mut page = 1usize;
    loop {
        let page_cache_key = CacheKey::new(cache_root.path.join("pages").join(page.to_string()));
        let results: GiteaSearchResults<T> = gitea_api_get(
            tenant,
            &endpoint_builder(page, GITEA_PAGE_SIZE),
            Some(page_cache_key),
        )
        .await?;
        let count = results.data.len();
        items.extend(results.data);
        if count < GITEA_PAGE_SIZE {
            break;
        }
        page += 1;
    }
    Ok(items)
}

pub fn parse_command_output<T: DeserializeOwned>(output: &CommandOutput) -> Result<T> {
    serde_json::from_slice(&output.stdout).wrap_err("Failed to deserialize command output as JSON")
}

pub fn dedupe_repositories(mut repositories: Vec<GiteaRepo>) -> Vec<GiteaRepo> {
    let mut by_id = BTreeMap::<GiteaRepoId, GiteaRepo>::new();
    for repo in repositories.drain(..) {
        by_id.entry(repo.id).or_insert(repo);
    }
    by_id.into_values().collect()
}

pub fn tenant_cache_key_prefix(tenant: &GiteaInstanceUrl) -> PathBuf {
    PathBuf::from_iter(["tea", tenant.storage_key().as_str()])
}
