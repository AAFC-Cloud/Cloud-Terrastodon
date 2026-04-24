use super::CognitiveServicesAccountArgument;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use cloud_terrastodon_azure::fetch_cognitive_services_account_deployments;
use eyre::Result;
use eyre::bail;
use std::io::Write;

/// Arguments for showing a single deployment for an Azure Cognitive Services account.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Cognitive Services account resource id, resource name, or wildcard pattern.
    pub account: CognitiveServicesAccountArgument<'static>,

    /// Deployment resource id or deployment name.
    pub deployment: String,
}

impl AzureCognitiveServicesDeploymentShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let Self {
            tenant,
            account,
            deployment,
        } = self;
        let tenant_id = tenant.resolve().await?;
        let accounts = fetch_all_cognitive_services_accounts(tenant_id).await?;
        let mut account_matches = accounts
            .into_iter()
            .filter(|item| account.matches(item))
            .collect::<Vec<_>>();

        let account = match account_matches.len() {
            0 => bail!("No Cognitive Services account found matching '{}'.", account),
            1 => account_matches.remove(0),
            _ => {
                account_matches.sort_by_key(|item| item.id.expanded_form());
                let ids = account_matches
                    .iter()
                    .map(|item| item.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple Cognitive Services accounts matched '{}'. Use a full resource id.\n  {}",
                    account,
                    ids
                )
            }
        };

        let deployment_needle = deployment.trim();
        let mut matches = fetch_cognitive_services_account_deployments(tenant_id, account.id)
            .await?
            .into_iter()
            .filter(|deployment| {
                deployment.id.expanded_form() == deployment_needle
                    || deployment.name.eq_ignore_ascii_case(deployment_needle)
            })
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No deployment found matching '{}'.", deployment_needle),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|deployment| deployment.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|deployment| deployment.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple deployments matched '{}'. Use a full resource id.\n  {}",
                    deployment_needle,
                    ids
                )
            }
        }
    }
}
