use super::CognitiveServicesAccountArgument;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountDeploymentListRequest;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Arguments for showing a single deployment for an Azure Cognitive Services account.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesDeploymentShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Cognitive Services account resource id, resource name, or wildcard pattern.
    #[facet(figue::positional)]
    pub cognitive_services_account: CognitiveServicesAccountArgument<'static>,

    /// Deployment resource id or deployment name.
    #[facet(figue::positional)]
    pub deployment: String,
}

impl AzureCognitiveServicesDeploymentShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let Self {
            authentication,
            cognitive_services_account,
            deployment,
        } = self;
        let authentication = authentication.resolve(executor).await?;
        let accounts = AzureCognitiveServicesAccountListRequest { authentication }
            .execute(executor)
            .await?;
        let mut account_matches = accounts
            .into_iter()
            .filter(|item| cognitive_services_account.matches(item))
            .collect::<Vec<_>>();

        let account = match account_matches.len() {
            0 => {
                bail!(
                    "No Cognitive Services account found matching '{}'.",
                    cognitive_services_account
                );
            }
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
                    cognitive_services_account,
                    ids
                );
            }
        };

        let deployment_needle = deployment.trim();
        let mut matches = AzureCognitiveServicesAccountDeploymentListRequest {
            authentication,
            account_id: (account.id.clone()),
        }
        .execute(executor)
        .await?
        .into_iter()
        .filter(|deployment| {
            deployment.id.expanded_form() == deployment_needle
                || deployment.name.eq_ignore_ascii_case(deployment_needle)
        })
        .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!("No deployment found matching '{}'.", deployment_needle);
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
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
                );
            }
        }
    }
}
