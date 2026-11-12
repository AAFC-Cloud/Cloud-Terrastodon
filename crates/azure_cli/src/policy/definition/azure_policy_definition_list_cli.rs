use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicyDefinitionListRequest;
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

/// Arguments for listing Azure policy definitions.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyDefinitionListArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,
    /// Filter policy definitions to those whose name satisfies the provided Nucleo fuzzy expression
    #[facet(figue::named)]
    pub name: Option<String>,
    /// Filter policy definitions to those whose parameter names satisfy all of the provided Nucleo fuzzy expressions
    #[facet(figue::named)]
    pub param_name: Option<Vec<String>>,
}

impl AzurePolicyDefinitionListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!("Fetching Azure policy definitions...");
        let mut policy_definitions = AzurePolicyDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions",
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
            policy_definitions.retain(|policy_definition| {
                !pattern
                    .match_list([policy_definition.name.as_str()], &mut matcher)
                    .is_empty()
            });
        }

        if let Some(param_names) = self.param_name.as_ref() {
            let mut patterns = Vec::with_capacity(param_names.len());
            for pattern in param_names {
                patterns.push(Pattern::new(
                    pattern,
                    CaseMatching::Smart,
                    Normalization::Smart,
                    AtomKind::Substring,
                ));
            }
            ensure!(!patterns.is_empty(), "At least one pattern is required");
            let mut matcher = Matcher::new(Config::DEFAULT);
            policy_definitions.retain(|policy_definition| {
                let Some(parameters) = &policy_definition.parameters else {
                    return false;
                };
                for pattern in &patterns {
                    let matches = pattern.match_list(parameters.keys(), &mut matcher);
                    if matches.is_empty() {
                        return false;
                    }
                }
                true
            });
        }

        Ok(CliOutput::facet(policy_definitions))
    }
}
