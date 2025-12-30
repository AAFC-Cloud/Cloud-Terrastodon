use crate::prelude::get_azure_devops_cli_config;
use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind::AzureCLI;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::bstr::ByteSlice;
use eyre::OptionExt;
use eyre::bail;
use std::pin::Pin;
use tracing::info;

#[must_use = "This is a future request, you must .await it"]
pub struct DefaultAzureDevOpsOrganizationUrlRequest;

pub fn get_default_organization_url() -> DefaultAzureDevOpsOrganizationUrlRequest {
    DefaultAzureDevOpsOrganizationUrlRequest
}

#[async_trait]
impl CacheInvalidatable for DefaultAzureDevOpsOrganizationUrlRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        get_azure_devops_cli_config().invalidate().await?;
        Ok(())
    }
}

impl IntoFuture for DefaultAzureDevOpsOrganizationUrlRequest {
    type Output = eyre::Result<AzureDevOpsOrganizationUrl>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let config = get_azure_devops_cli_config().await?;
            let org = config
                .lines()
                .find(|line| line.contains("organization"))
                .ok_or_eyre("Expected organization to be configured using `az devops configure --defaults organization=https://dev.azure.com/myorg/`")?;
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

pub async fn set_default_organization_url(org: AzureDevOpsOrganizationUrl) -> eyre::Result<()> {
    info!("Setting default organization to {}", org);
    let mut cmd = CommandBuilder::new(AzureCLI);
    cmd.args([
        "devops",
        "configure",
        "--defaults",
        format!("organization={}", org).as_str(),
    ]);
    let resp = cmd.run_raw().await?;
    if !resp.success() {
        bail!(
            "Failed to set default organization: {}",
            resp.stderr.to_str()?
        );
    }

    get_default_organization_url().invalidate().await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::prelude::get_default_organization_url;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_organization_url().await?;
        println!("The default org is {x}");
        Ok(())
    }
}
