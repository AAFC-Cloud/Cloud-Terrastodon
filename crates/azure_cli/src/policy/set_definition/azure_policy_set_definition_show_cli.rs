use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_cli::AzureAuthenticationArgs;
use cloud_terrastodon_azure::AzurePolicySetDefinitionListRequest;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::sync::Arc;
use tracing::info;

/// Arguments for showing a single Azure policy set definition.
#[derive(Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionShowArgs {
    #[facet(flatten)]
    pub authentication: AzureAuthenticationArgs,

    /// Policy set definition id, name, or display name.
    #[facet(figue::positional)]
    pub policy_set_definition: String,
}

impl AzurePolicySetDefinitionShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let authentication = self.authentication.resolve(executor).await?;
        info!(needle = %self.policy_set_definition, tenant_id = %authentication.tenant_id, "Fetching Azure policy set definitions");
        let policy_set_definitions = AzurePolicySetDefinitionListRequest { authentication }
            .execute(executor)
            .await?;
        info!(
            count = policy_set_definitions.len(),
            "Fetched Azure policy set definitions"
        );

        let needle = self.policy_set_definition.trim();
        if let Some(policy_set_definition) = policy_set_definitions
            .iter()
            .find(|policy_set_definition| policy_set_definition.id.expanded_form() == needle)
        {
            return Ok(CliOutput::facet(Arc::clone(policy_set_definition)));
        }

        let mut name_matches = policy_set_definitions
            .iter()
            .filter(|policy_set_definition| {
                policy_set_definition
                    .name
                    .as_str()
                    .eq_ignore_ascii_case(needle)
            })
            .collect::<Vec<_>>();
        if name_matches.is_empty() {
            name_matches = policy_set_definitions
                .iter()
                .filter(|policy_set_definition| {
                    policy_set_definition
                        .display_name
                        .as_deref()
                        .map(|display_name| display_name.eq_ignore_ascii_case(needle))
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>();
        }

        match name_matches.len() {
            0 => {
                bail!("No policy set definition found matching '{}'.", needle);
            }
            1 => Ok(CliOutput::facet(Arc::clone(name_matches.remove(0)))),
            _ => {
                name_matches
                    .sort_by_key(|policy_set_definition| policy_set_definition.id.expanded_form());
                let ids = name_matches
                    .iter()
                    .map(|policy_set_definition| policy_set_definition.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple policy set definitions matched '{}'. Use a full resource id.\n  {}",
                    needle,
                    ids
                );
            }
        }
    }
}
