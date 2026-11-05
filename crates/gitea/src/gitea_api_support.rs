use crate::GiteaInstanceUrl;
use crate::GiteaLogin;
use crate::GiteaRepo;
use crate::GiteaRepoId;
use crate::GiteaSearchResults;
use crate::list_gitea_logins;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::CacheScope;
use cloud_terrastodon_command::CancellationReportExt;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::CommandOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Context;
use eyre::Result;
use facet::Facet;
use std::collections::BTreeMap;
use std::time::Duration;

pub const GITEA_UNCACHED_DELAY: Duration = Duration::from_millis(500);
pub const GITEA_PAGE_SIZE: usize = 50;

/// Read through tea's explicitly selected external login profile.
/// External tool credentials are separate from the executor's Azure provider.
/// Results are not reused because tea does not expose a credential generation.
pub async fn gitea_api_get<T: Facet<'static> + Send + 'static>(
    tenant: &GiteaInstanceUrl,
    endpoint: &str,
    cache_key: Option<CacheKey>,
    executor: &CloudTerrastodonExecutor,
) -> Result<T> {
    let url = tenant.api_url(endpoint);
    eyre::ensure!(
        url.starts_with(&format!("{tenant}/")),
        "Gitea endpoint must belong to the selected instance"
    );
    let login = GiteaLogin::for_instance(list_gitea_logins(executor).await?, tenant)?;
    let mut cmd = CommandBuilder::new(CommandKind::Gitea);
    cmd.cancellation_safe();
    // tea owns the external credential lifecycle. An explicit profile prevents
    // ambient default selection; no executor credential lookup is requested.
    // Its credential generation is unavailable, so service results execute on
    // every call rather than being reused under a merely non-secret name.
    cmd.args(["api", "--login", &login.name, url.as_str()]);
    cmd.use_cache(cache_key.map(|key| CacheKey {
        scope: key.scope,
        valid_for: Duration::ZERO,
    }));
    cmd.run_polite_with_artifacts(GITEA_UNCACHED_DELAY, executor)
        .await
}

pub async fn gitea_api_get_best_effort<T: Facet<'static> + Send + 'static>(
    tenant: &GiteaInstanceUrl,
    endpoint: &str,
    cache_key: Option<CacheKey>,
    executor: &CloudTerrastodonExecutor,
) -> Result<Option<T>> {
    // Keep the raw inspection contract separate from the typed getter's artifacts.
    let cache_key = cache_key
        .map(|key| -> Result<CacheKey> {
            Ok(CacheKey {
                scope: key.scope.join("raw-best-effort")?,
                valid_for: Duration::ZERO,
            })
        })
        .transpose()?;
    let url = tenant.api_url(endpoint);
    eyre::ensure!(
        url.starts_with(&format!("{tenant}/")),
        "Gitea endpoint must belong to the selected instance"
    );
    let login = GiteaLogin::for_instance(list_gitea_logins(executor).await?, tenant)?;
    let mut cmd = CommandBuilder::new(CommandKind::Gitea);
    cmd.cancellation_safe();
    // tea owns the external credential lifecycle. An explicit profile prevents
    // ambient default selection; no executor credential lookup is requested.
    // Its credential generation is unavailable, so service results execute on
    // every call rather than being reused under a merely non-secret name.
    cmd.args(["api", "--login", &login.name, url.as_str()]);
    cmd.use_cache(cache_key.map(|key| CacheKey {
        scope: key.scope,
        valid_for: Duration::ZERO,
    }));
    match cmd
        .run_raw_polite_with_artifacts(GITEA_UNCACHED_DELAY, executor)
        .await
    {
        Ok(output) => Ok(Some(parse_command_output::<T>(&output)?)),
        Err(error) if error.is_cancellation() => Err(error),
        Err(error) => {
            executor.cancellation().bail_if_cancelled()?;
            tracing::debug!(%error, endpoint, "Skipping Gitea endpoint after best-effort failure");
            Ok(None)
        }
    }
}

pub async fn gitea_api_get_paged<T>(
    tenant: &GiteaInstanceUrl,
    cache_root: CacheKey,
    endpoint_builder: impl Fn(usize, usize) -> String,
    executor: &CloudTerrastodonExecutor,
) -> Result<Vec<T>>
where
    T: Facet<'static> + Send + 'static,
{
    let mut items = Vec::new();
    let mut page = 1usize;
    loop {
        executor.cancellation().bail_if_cancelled()?;
        let page_cache_key = CacheKey {
            scope: cache_root.scope.join("pages")?.join(page.to_string())?,
            valid_for: Duration::ZERO,
        };
        let page_items: Vec<T> = gitea_api_get(
            tenant,
            &endpoint_builder(page, GITEA_PAGE_SIZE),
            Some(page_cache_key),
            executor,
        )
        .await?;
        executor.cancellation().bail_if_cancelled()?;
        let count = page_items.len();
        items.extend(page_items);
        if count < GITEA_PAGE_SIZE {
            break;
        }
        page += 1;
    }
    executor.cancellation().bail_if_cancelled()?;
    Ok(items)
}

pub async fn gitea_api_get_search_paged<T>(
    tenant: &GiteaInstanceUrl,
    cache_root: CacheKey,
    endpoint_builder: impl Fn(usize, usize) -> String,
    executor: &CloudTerrastodonExecutor,
) -> Result<Vec<T>>
where
    T: Facet<'static> + Send + 'static,
{
    let mut items = Vec::new();
    let mut page = 1usize;
    loop {
        executor.cancellation().bail_if_cancelled()?;
        let page_cache_key = CacheKey {
            scope: cache_root.scope.join("pages")?.join(page.to_string())?,
            valid_for: Duration::ZERO,
        };
        let results: GiteaSearchResults<T> = gitea_api_get(
            tenant,
            &endpoint_builder(page, GITEA_PAGE_SIZE),
            Some(page_cache_key),
            executor,
        )
        .await?;
        executor.cancellation().bail_if_cancelled()?;
        let count = results.data.len();
        items.extend(results.data);
        if count < GITEA_PAGE_SIZE {
            break;
        }
        page += 1;
    }
    executor.cancellation().bail_if_cancelled()?;
    Ok(items)
}

pub fn parse_command_output<T: Facet<'static>>(output: &CommandOutput) -> Result<T> {
    facet_json::from_slice(&output.stdout)
        .map_err(|error| eyre::eyre!("{error:?}"))
        .wrap_err("Failed to deserialize command output as JSON")
}

pub fn dedupe_repositories(mut repositories: Vec<GiteaRepo>) -> Vec<GiteaRepo> {
    let mut by_id = BTreeMap::<GiteaRepoId, GiteaRepo>::new();
    for repo in repositories.drain(..) {
        by_id.entry(repo.id).or_insert(repo);
    }
    by_id.into_values().collect()
}

pub fn tenant_cache_key_prefix(tenant: &GiteaInstanceUrl) -> Result<CacheScope> {
    CacheScope::try_from_components(["tea", tenant.storage_key().as_str()])
}
