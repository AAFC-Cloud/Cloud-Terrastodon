use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicyDefinitionListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for showing a single Azure policy definition.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicyDefinitionShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Policy definition id, name, or display name.
    #[facet(figue::positional)]
    pub policy_definition: String,
}

impl AzurePolicyDefinitionShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.policy_definition, tenant_id = %authentication.tenant_id, "Fetching Azure policy definitions");
        let policy_definitions = AzurePolicyDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_definitions.len(),
            "Fetched Azure policy definitions"
        );

        let needle = self.policy_definition.trim();
        if let Some(policy_definition) = policy_definitions
            .iter()
            .find(|policy_definition| policy_definition.id.expanded_form() == needle)
        {
            return Ok(CliOutput::facet(Arc::clone(policy_definition)));
        }

        let mut name_matches = policy_definitions
            .iter()
            .filter(|policy_definition| {
                policy_definition.name.as_str().eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();
        if name_matches.is_empty() {
            name_matches = policy_definitions
                .iter()
                .filter(|policy_definition| {
                    policy_definition
                        .display_name
                        .as_deref()
                        .map(|display_name| display_name.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => {
                bail!("No policy definition found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(Arc::clone(name_matches.remove(0)))),
            _ => {
                name_matches.sort_by_key(|policy_definition| policy_definition.id.expanded_form());
                let ids = name_matches
                    .iter()
                    .map(|policy_definition| policy_definition.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple policy definitions matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                );
            }
        }
    }
}
