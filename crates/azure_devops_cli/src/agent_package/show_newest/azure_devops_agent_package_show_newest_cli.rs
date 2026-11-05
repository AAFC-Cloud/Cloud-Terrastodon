use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureDevOpsTenantArgumentExt;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_packages;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use std::collections::HashMap;
use std::io::stdout;

/// Show the newest Azure DevOps agent package by `createdOn`, for each `platform`.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageShowNewestArgs {
    /// Tenant id or tracked alias for bearer authentication; PAT authentication may omit it.
    #[facet(figue::named)]
    pub tenant: Option<AzureTenantArgument<'static>>,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsAgentPackageShowNewestArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<()> {
        let azure_devops_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let pkgs =
            fetch_azure_devops_agent_packages(&self.org, &azure_devops_auth_context, cancellation)
                .await?;

        // Group by platform and keep the package with the most recent created_on per platform
        let mut newest_by_platform: HashMap<String, AzureDevOpsAgentPackage> = HashMap::new();
        for pkg in pkgs.into_iter() {
            use std::collections::hash_map::Entry;
            match newest_by_platform.entry(pkg.platform.clone()) {
                Entry::Vacant(e) => {
                    e.insert(pkg);
                }
                Entry::Occupied(mut o) => {
                    if pkg.created_on > o.get().created_on {
                        o.insert(pkg);
                    }
                }
            }
        }

        let mut result: Vec<_> = newest_by_platform.into_values().collect();
        // deterministic order
        result.sort_by(|a, b| a.platform.cmp(&b.platform));

        facet_json::to_writer_std_pretty(stdout(), &result)
            .map_err(|error| eyre::eyre!("{error:?}"))?;

        Ok(())
    }
}
