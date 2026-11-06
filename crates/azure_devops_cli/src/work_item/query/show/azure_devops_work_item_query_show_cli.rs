use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemQueryPath;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemQueryShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub project: AzureDevOpsProjectArgument<'static>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Saved query UUID or folder/query path.
    #[facet(figue::positional)]
    pub query: AzureDevOpsWorkItemQueryPath,
}

impl AzureDevOpsWorkItemQueryShowArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self.tenant.bind_auth_context(auth, cancellation).await?;
        let executor = &auth_context.bind_executor(executor)?;
        let query = AzureDevOpsWorkItemQueryGetRequest {
            org_url: Cow::Owned(self.org),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            query: self.query,
        }
        .execute(executor)
        .await?;
        facet_json::to_writer_std_pretty(std::io::stdout(), &query)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
