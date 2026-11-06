use crate::GiteaInstanceUrl;
use crate::GiteaLoginsListRequest;
use crate::list_gitea_logins;
use cloud_terrastodon_cache::CacheInvalidationKeys;
use cloud_terrastodon_cache::CacheKey;
use cloud_terrastodon_cache::HasCacheKey;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use cloud_terrastodon_execution::ExecutionFuture;
use cloud_terrastodon_execution::RequestExecution;
use eyre::OptionExt;
use eyre::bail;

#[must_use = "This is a future request, you must .execute it"]
#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct DefaultGiteaInstanceUrlRequest;

pub fn get_default_gitea_instance_url(
    executor: &CloudTerrastodonExecutor,
) -> RequestExecution<'_, DefaultGiteaInstanceUrlRequest> {
    executor.execute(DefaultGiteaInstanceUrlRequest)
}

impl CacheInvalidationKeys for DefaultGiteaInstanceUrlRequest {
    fn cache_keys_to_invalidate(&self) -> eyre::Result<Vec<CacheKey>> {
        Ok(vec![GiteaLoginsListRequest.cache_key()?])
    }
}

impl ExecutableRequest for DefaultGiteaInstanceUrlRequest {
    type Error = eyre::Report;
    type Output = GiteaInstanceUrl;

    fn run<'a>(self, executor: CloudTerrastodonExecutor) -> ExecutionFuture<'a, Self::Output>
    where
        Self: 'a,
    {
        Box::pin(async move {
            let logins = list_gitea_logins(&executor).await?;
            if logins.is_empty() {
                bail!("No Gitea logins were found. Configure one with the `tea` CLI first.");
            }

            if let Some(login) = logins.iter().find(|login| login.is_default()) {
                return Ok(login.url.clone());
            }

            if logins.len() == 1 {
                return Ok(logins
                    .into_iter()
                    .next()
                    .ok_or_eyre("Expected exactly one login")?
                    .url);
            }

            bail!(
                "No default Gitea login was found. Set a default with `tea login default <name>` or specify `--tenant`."
            );
        })
    }
}

cloud_terrastodon_registry::register_thing!(DefaultGiteaInstanceUrlRequest);
cloud_terrastodon_registry::register_arbitrary!(DefaultGiteaInstanceUrlRequest);
cloud_terrastodon_registry_execution::register_executable_request!(
    DefaultGiteaInstanceUrlRequest => GiteaInstanceUrl,
    effects = [Read]
);
