use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureDevOpsAuthenticationArgs;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPackage;
use cloud_terrastodon_azure_devops::AzureDevOpsAgentPackagesRequest;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::collections::HashMap;

/// Show the newest Azure DevOps agent package by `createdOn`, for each `platform`.
#[derive(Facet, Debug, Clone)]
pub struct AzureDevOpsAgentPackageShowNewestArgs {
    #[facet(flatten)]
    pub authentication: AzureDevOpsAuthenticationArgs,

    /// Azure DevOps organization name or URL.
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}

impl AzureDevOpsAgentPackageShowNewestArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let auth_selection = self.authentication.resolve(&self.org, executor).await?;
        let pkgs = AzureDevOpsAgentPackagesRequest {
            org_url: Cow::Borrowed(&self.org),
            auth_selection,
        }
        .execute(executor)
        .await?;

        // Group by platform and keep the package with the most recent created_on per platform
        let mut newest_by_platform: HashMap<&str, &AzureDevOpsAgentPackage> = HashMap::new();
        for pkg in pkgs.iter() {
            use std::collections::hash_map::Entry;
            match newest_by_platform.entry(&pkg.platform) {
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

        let mut result: Vec<_> = newest_by_platform.into_values().cloned().collect();
        // deterministic order
        result.sort_by(|a, b| a.platform.cmp(&b.platform));

        Ok(CliOutput::facet(result))
    }
}
