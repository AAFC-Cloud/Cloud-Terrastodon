use chrono::DateTime;
use chrono::Utc;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsProjectArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemExpand;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemGetRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsWorkItemId;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;

#[derive(Debug, Clone, facet::Facet)]
pub struct AzureDevOpsWorkItemShowArgs {
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
    #[facet(figue::named)]
    pub project: Option<AzureDevOpsProjectArgument<'static>>,
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,
    /// Work item ID.
    #[facet(figue::positional)]
    pub id: AzureDevOpsWorkItemId,
    /// Response expansion (all includes fields and relations).
    #[facet(figue::named, default)]
    pub expand: AzureDevOpsWorkItemExpand,
    /// Comma-separated field reference names.
    #[facet(figue::named)]
    pub fields: Option<String>,
    /// Read a historical snapshot at this UTC timestamp.
    #[facet(figue::named)]
    pub as_of: Option<DateTime<Utc>>,
}

impl AzureDevOpsWorkItemShowArgs {
    pub async fn invoke(self, auth: &AuthContext) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let auth_context = self.tenant.bind_auth_context(auth).await?;
        let item = AzureDevOpsWorkItemGetRequest {
            org_url: Cow::Owned(org_url),
            project: self.project,
            auth_context: Cow::Owned(auth_context),
            id: self.id,
            expand: self.expand,
            fields: self
                .fields
                .map(|fields| {
                    fields
                        .split(',')
                        .map(|field| field.trim().to_owned())
                        .collect()
                })
                .unwrap_or_default(),
            as_of: self.as_of,
        }
        .await?;
        cloud_terrastodon_command::to_writer_pretty(std::io::stdout(), &item)?;
        Ok(())
    }
}
