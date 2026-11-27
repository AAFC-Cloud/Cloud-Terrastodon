use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::EntraUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsEntraPermissionSubject;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use cloud_terrastodon_azure_devops::scan_azure_devops_entra_permissions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use eyre::ensure;
use facet::Facet;

/// Find Azure DevOps grants for an Entra user or group across tracked organizations.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsEntraPermissionsArgs {
    /// Entra user object ID.
    #[facet(figue::named)]
    pub user: Option<EntraUserId>,
    /// Entra group object ID.
    #[facet(figue::named)]
    pub group: Option<EntraGroupId>,
    /// Restrict the scan to this tracked organization. Repeat to select multiple
    /// organizations. Omit to scan all organizations tracked for the selected tenant.
    #[facet(figue::named, default)]
    pub org: Vec<AzureDevOpsOrganizationUrl>,
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Refresh cached identities and permission reports.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl AzureDevOpsEntraPermissionsArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let subject = match (self.user, self.group) {
            (Some(user), None) => AzureDevOpsEntraPermissionSubject::User(user),
            (None, Some(group)) => AzureDevOpsEntraPermissionSubject::Group(group),
            _ => {
                bail!("Specify exactly one of --user or --group");
            }
        };
        let tenant_id = self.authentication.resolve_tenant_id(executor).await?;
        let mut organizations = TrackedAzureDevOpsOrganizationStore::configured()
            .list_for_tenant(tenant_id)
            .await?;
        for url in &self.org {
            ensure!(
                organizations
                    .iter()
                    .any(|organization| organization.url == *url),
                "Organization {url} is not tracked for tenant {tenant_id}; check its association with ct az devops org show"
            );
        }
        if !self.org.is_empty() {
            organizations.retain(|organization| self.org.contains(&organization.url));
        }
        let reports = if organizations.is_empty() {
            Vec::new()
        } else {
            scan_azure_devops_entra_permissions(
                organizations,
                self.authentication.resolve(executor).await?,
                subject,
                self.no_cache,
                executor,
            )
            .await?
            .into_iter()
            .map(|organization| organization.map_failure(|error| format!("{error:#}")))
            .collect()
        };
        Ok(CliOutput::facet(reports))
    }
}
