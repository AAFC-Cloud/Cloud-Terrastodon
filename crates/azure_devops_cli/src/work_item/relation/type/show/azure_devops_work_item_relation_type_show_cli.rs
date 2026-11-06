use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemRelationTypeName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemRelationTypeShowArgs {
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Resource name or reference name.
    #[facet(figue::positional)]
    pub name: AzureDevOpsWorkItemRelationTypeName,
}

impl AzureDevOpsWorkItemRelationTypeShowArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let auth_context = self.tenant.bind_auth_context(auth, cancellation).await?;
        let executor = &auth_context.bind_executor(executor)?;
        let value = AzureDevOpsWorkItemRelationTypeGetRequest {
            org_url: Cow::Owned(self.org),
            auth_context: Cow::Owned(auth_context),
            name: self.name,
        }
        .execute(executor)
        .await?;
        facet_json::to_writer_std_pretty(std::io::stdout(), &value)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        Ok(())
    }
}
