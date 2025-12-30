use std::pin::Pin;

use cloud_terrastodon_azure_devops_types::prelude::AzureDevOpsProjectName;
use cloud_terrastodon_command::CacheInvalidatable;
use cloud_terrastodon_command::async_trait;
use eyre::OptionExt;
use eyre::bail;

use crate::prelude::get_azure_devops_cli_config;

#[must_use = "This is a future request, you must .await it"]
pub struct DefaultAzureDevOpsProjectNameRequest;

pub fn get_default_project_name() -> DefaultAzureDevOpsProjectNameRequest {
    DefaultAzureDevOpsProjectNameRequest
}

#[async_trait]
impl CacheInvalidatable for DefaultAzureDevOpsProjectNameRequest {
    async fn invalidate(&self) -> eyre::Result<()> {
        get_azure_devops_cli_config().invalidate().await?;
        Ok(())
    }
}

impl IntoFuture for DefaultAzureDevOpsProjectNameRequest {
    type Output = eyre::Result<AzureDevOpsProjectName>;
    type IntoFuture = Pin<Box<dyn std::future::Future<Output = Self::Output> + Send>>;

    fn into_future(self) -> Self::IntoFuture {
        Box::pin(async move {
            let config = get_azure_devops_cli_config().await?;
            let project = config
                .lines()
                .find(|line| line.contains("project"))
                .ok_or_eyre("Expected project to be configured using `az devops configure --defaults project=MyProject`")?;
            let Some((_, project)) = project.split_once('=') else {
                bail!("Expected project to have a slash before the name, found {project:?}");
            };
            
            let project = project.trim();
            Ok(AzureDevOpsProjectName::new(project.to_string()))
        })
    }
}


#[cfg(test)]
mod test {
    use crate::prelude::get_default_project_name;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let x = get_default_project_name().await?;
        println!("The default project is {x:?}");
        Ok(())
    }
}
