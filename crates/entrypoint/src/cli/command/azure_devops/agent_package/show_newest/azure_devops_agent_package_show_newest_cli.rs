use cloud_terrastodon_azure_devops::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::fetch_azure_devops_agent_packages;
use cloud_terrastodon_command::to_writer_pretty;
use eyre::Result;
use std::collections::HashMap;
use std::io::stdout;

/// Show the newest Azure DevOps agent package by `createdOn`, for each `platform`.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageShowNewestArgs {
    /// Azure DevOps organization name or URL. Defaults to the configured organization.
    #[facet(figue::named)]
    pub org: Option<AzureDevOpsOrganizationUrl>,
}

impl AzureDevOpsAgentPackageShowNewestArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url =
            crate::cli::azure_devops::resolve_azure_devops_organization_url(self.org).await?;
        let pkgs = fetch_azure_devops_agent_packages(&org_url).await?;

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

        to_writer_pretty(stdout(), &result)?;

        Ok(())
    }
}
