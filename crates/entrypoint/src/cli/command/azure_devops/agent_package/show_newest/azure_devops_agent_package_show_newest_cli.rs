use clap::Args;
use cloud_terrastodon_azure_devops::prelude::{fetch_azure_devops_agent_packages, get_default_organization_url, AzureDevOpsAgentPackage};
use eyre::Result;
use serde_json::to_writer_pretty;
use std::collections::HashMap;
use std::io::stdout;

/// Show the newest Azure DevOps agent package by `createdOn`, for each `platform`.
#[derive(Args, Debug, Clone)]
pub struct AzureDevOpsAgentPackageShowNewestArgs {}

impl AzureDevOpsAgentPackageShowNewestArgs {
    pub async fn invoke(self) -> Result<()> {
        let org_url = get_default_organization_url().await?;
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
