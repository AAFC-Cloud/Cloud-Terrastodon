use super::CognitiveServicesAccountArgument;
use clap::Args;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountDeployment;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountResource;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use cloud_terrastodon_azure::fetch_cognitive_services_account_deployments;
use eyre::Result;
use eyre::bail;
use serde::Serialize;
use std::io::Write;

#[derive(Debug, Clone, Serialize)]
struct CognitiveServicesAccountDeploymentListEntry {
    account: AzureCognitiveServicesAccountResource,
    deployment: AzureCognitiveServicesAccountDeployment,
}

/// Arguments for listing deployments for Azure Cognitive Services accounts.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Optional Cognitive Services account resource id, resource name, or wildcard pattern.
    pub account: Option<CognitiveServicesAccountArgument<'static>>,
}

impl AzureCognitiveServicesDeploymentListArgs {
    pub async fn invoke(self) -> Result<()> {
        let Self { tenant, account } = self;
        let tenant_id = tenant.resolve().await?;
        let accounts = fetch_all_cognitive_services_accounts(tenant_id).await?;

        let deployments = if let Some(account_argument) = account {
            let mut matches = accounts
                .into_iter()
                .filter(|item| account_argument.matches(item))
                .collect::<Vec<_>>();

            match matches.len() {
                0 => bail!("No Cognitive Services account found matching '{}'.", account_argument),
                1 => {
                    let account = matches.remove(0);
                    let deployments =
                        fetch_cognitive_services_account_deployments(tenant_id, account.id.clone())
                            .await?;
                    deployments
                        .into_iter()
                        .map(|deployment| CognitiveServicesAccountDeploymentListEntry {
                            account: account.clone(),
                            deployment,
                        })
                        .collect::<Vec<_>>()
                }
                _ => {
                    matches.sort_by_key(|item| item.id.expanded_form());
                    let ids = matches
                        .iter()
                        .map(|item| item.id.expanded_form())
                        .collect::<Vec<_>>()
                        .join("\n  ");
                    bail!(
                        "Multiple Cognitive Services accounts matched '{}'. Use a full resource id.\n  {}",
                        account_argument,
                        ids
                    )
                }
            }
        } else {
            let mut deployments = Vec::new();
            for account in accounts {
                let account_deployments =
                    fetch_cognitive_services_account_deployments(tenant_id, account.id.clone())
                        .await?;
                deployments.extend(account_deployments.into_iter().map(|deployment| {
                    CognitiveServicesAccountDeploymentListEntry {
                        account: account.clone(),
                        deployment,
                    }
                }));
            }
            deployments
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &deployments)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
