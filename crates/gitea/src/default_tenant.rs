use crate::GiteaInstanceUrl;
use crate::list_gitea_logins;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::async_trait;
use eyre::OptionExt;
use eyre::bail;
use std::future::IntoFuture;
use std::pin::Pin;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, arbitrary::Arbitrary, facet::Facet)]
pub struct DefaultGiteaInstanceUrlRequest;

pub fn get_default_gitea_instance_url() -> DefaultGiteaInstanceUrlRequest {
    DefaultGiteaInstanceUrlRequest
}

#[async_trait]
impl CacheInvalidatable for DefaultGiteaInstanceUrlRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        list_gitea_logins().invalidate().await
    }
}

impl IntoFuture for DefaultGiteaInstanceUrlRequest {
    type Output = eyre::Result<GiteaInstanceUrl>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let logins = list_gitea_logins().await?;
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
            )
        })
    }
}

cloud_terrastodon_registry::register_thing!(DefaultGiteaInstanceUrlRequest);
cloud_terrastodon_registry::register_arbitrary!(DefaultGiteaInstanceUrlRequest);
cloud_terrastodon_registry::register_into_future!(
    DefaultGiteaInstanceUrlRequest => GiteaInstanceUrl,
    effects = [Read]
);
