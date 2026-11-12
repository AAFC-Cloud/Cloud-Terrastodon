use super::CognitiveServicesAccountArgument;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzureCognitiveServicesAccountListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;

/// Arguments for showing a single Azure Cognitive Services account.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Cognitive Services account resource id, resource name, or wildcard pattern.
    #[facet(figue::positional)]
    pub cognitive_services_account: CognitiveServicesAccountArgument<'static>,
}

impl AzureCognitiveServicesShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let Self {
            authentication,
            cognitive_services_account,
        } = self;
        let authentication = authentication.resolve(executor).await?;
        let accounts = AzureCognitiveServicesAccountListRequest { authentication }
            .execute(executor)
            .await?;
        let mut matches = accounts
            .into_iter()
            .filter(|item| cognitive_services_account.matches(item))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!(
                    "No Cognitive Services account found matching '{}'.",
                    cognitive_services_account
                );
            }
            1 => Ok(CliOutput::facet(matches.remove(0))),
            _ => {
                matches.sort_by_key(|account| account.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|account| account.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple Cognitive Services accounts matched '{}'. Use a full resource id.\n  {}",
                    cognitive_services_account,
                    ids
                );
            }
        }
    }
}
