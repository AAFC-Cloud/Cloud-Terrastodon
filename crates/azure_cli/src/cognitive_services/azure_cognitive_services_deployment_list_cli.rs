use crate::cognitive_services::CognitiveServicesAccountArgument;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountDeployment;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountDeploymentListRequest;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountListRequest;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountResource;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::sync::Arc;

#[derive(Debug, Clone, Facet)]
struct CognitiveServicesAccountDeploymentListEntry {
    account: Arc<AzureCognitiveServicesAccountResource>,
    deployment: Arc<AzureCognitiveServicesAccountDeployment>,
}

/// Arguments for listing deployments for Azure Cognitive Services accounts.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Optional Cognitive Services account resource id, resource name, or wildcard pattern.
    #[facet(figue::positional, default)]
    pub cognitive_services_account: Option<CognitiveServicesAccountArgument<'static>>,
}

impl AzureCognitiveServicesDeploymentListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let Self {
            authentication,
            cognitive_services_account,
        } = self;
        let authentication = authentication.resolve(executor).await?;
        let accounts = AzureCognitiveServicesAccountListRequest { authentication }
            .execute(executor)
            .await?;

        let deployments = if let Some(account_argument) = cognitive_services_account {
            let mut matches = accounts
                .into_iter()
                .filter(|item| account_argument.matches(item))
                .collect::<Vec<_>>();

            match matches.len() {
                0 => {
                    bail!(
                        "No Cognitive Services account found matching '{}'.",
                        account_argument
                    );
                }
                1 => {
                    let account = matches.remove(0);
                    let deployments = AzureCognitiveServicesAccountDeploymentListRequest {
                        authentication,
                        account_id: (account.id.clone()),
                    }
                    .execute(executor)
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
                    );
                }
            }
        } else {
            let mut deployments = Vec::new();
            for account in accounts {
                let account_deployments = AzureCognitiveServicesAccountDeploymentListRequest {
                    authentication,
                    account_id: (account.id.clone()),
                }
                .execute(executor)
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

        Ok(CliOutput::facet(deployments))
    }
}
