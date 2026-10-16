use crate::get_azure_devops_cli_config;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_devops_types::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind::AzureCLI;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::bstr::ByteSlice;
use eyre::OptionExt;
use eyre::bail;
use std::pin::Pin;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct DefaultAzureDevOpsOrganizationUrlRequest {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,
}

pub fn get_default_organization_url(
    cancellation: &CancellationScope,
) -> DefaultAzureDevOpsOrganizationUrlRequest {
    DefaultAzureDevOpsOrganizationUrlRequest {
        cancellation: cancellation.clone(),
    }
}

#[async_trait]
impl CacheInvalidatable for DefaultAzureDevOpsOrganizationUrlRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        get_azure_devops_cli_config(&self.cancellation)
            .invalidate()
            .await?;
        Ok(())
    }
}

impl IntoFuture for DefaultAzureDevOpsOrganizationUrlRequest {
    type Output = eyre::Result<AzureDevOpsOrganizationUrl>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let config = get_azure_devops_cli_config(&self.cancellation).await?;
            let org = config
                .lines()
                .find(|line| line.contains("organization"))
                .ok_or_eyre("Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/` followed by `cloud_terrastodon clean` to empty the invalid cache.")?;
            let Some((_, org)) = org.rsplit_once('=') else {
                bail!("Missing equal sign delimiting value, found {org:?}");
            };
            let url = org
                .trim()
                .to_string()
                .parse::<AzureDevOpsOrganizationUrl>()?;
            Ok(url)
        })
    }
}

pub async fn set_default_organization_url(
    org: AzureDevOpsOrganizationUrl,
    cancellation: &CancellationScope,
) -> eyre::Result<()> {
    info!("Setting default organization to {}", org);
    let mut cmd = CommandBuilder::new(AzureCLI);
    cmd.args([
        "devops",
        "configure",
        "--defaults",
        format!("organization={}", org).as_str(),
    ]);
    cancellation
        .deferred(
            "setting default organization and invalidating cached configuration",
            |cancellation| async move {
                let resp = cmd.run_raw(&cancellation).await?;
                if !resp.success() {
                    bail!(
                        "Failed to set default organization: {}",
                        resp.stderr.to_str()?
                    );
                }

                get_default_organization_url(&cancellation)
                    .invalidate()
                    .await?;
                Ok(())
            },
        )
        .await
}

cloud_terrastodon_registry::register_thing!(DefaultAzureDevOpsOrganizationUrlRequest);
cloud_terrastodon_registry::register_arbitrary!(DefaultAzureDevOpsOrganizationUrlRequest);
cloud_terrastodon_registry::register_into_future!(
    DefaultAzureDevOpsOrganizationUrlRequest => AzureDevOpsOrganizationUrl,
    effects = [Read]
);

#[cfg(test)]
mod test {
    use crate::get_default_organization_url;
    use cloud_terrastodon_command::CancellationToken;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        CancellationToken::new()
            .with_async_scope(|cancellation| async move {
                let x = get_default_organization_url(&cancellation).await?;
                assert!(!x.to_string().is_empty());
                Ok(())
            })
            .await
    }
}
