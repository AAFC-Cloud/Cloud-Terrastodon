use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicySetDefinitionListRequest;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::ensure;
use facet::Facet;
use nucleo::Config;
use nucleo::Matcher;
use nucleo::pattern::AtomKind;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use tracing::info;

/// Arguments for listing Azure policy set definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Filter policy set definitions to those whose name satisfies the provided Nucleo fuzzy expression
    #[facet(figue::named)]
    pub name: Option<String>,
}

impl AzurePolicySetDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure policy set definitions...");
        let mut policy_set_definitions = AzurePolicySetDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_set_definitions.len(),
            "Fetched Azure policy set definitions",
        );

        if let Some(name) = self.name.as_ref() {
            ensure!(!name.is_empty(), "The name pattern is required");
            let pattern = Pattern::new(
                name,
                CaseMatching::Smart,
                Normalization::Smart,
                AtomKind::Substring,
            );
            let mut matcher = Matcher::new(Config::DEFAULT);
            policy_set_definitions.retain(|policy_set_definition| {
                !pattern
                    .match_list([policy_set_definition.name.as_str()], &mut matcher)
                    .is_empty()
            });
        }

        Ok(CliOutput::facet(policy_set_definitions))
    }
}
