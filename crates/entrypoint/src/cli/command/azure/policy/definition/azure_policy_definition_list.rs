use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_policy_definitions;
use eyre::Result;
use eyre::ensure;
use nucleo::Config;
use nucleo::Matcher;
use nucleo::pattern::AtomKind;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure policy definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicyDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
    /// Filter policy definitions to those whose name satisfies the provided Nucleo fuzzy expression
    #[arg(long)]
    pub name: Option<String>,
    /// Filter policy definitions to those whose parameter names satisfy all of the provided Nucleo fuzzy expressions
    #[arg(long)]
    pub param_name: Option<Vec<String>>,
}

impl AzurePolicyDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure policy definitions...");
        let mut policy_definitions = fetch_all_policy_definitions(tenant_id).await?;
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
            policy_definitions.retain(
                |policy_definition: &cloud_terrastodon_azure::PolicyDefinition| {
                    for pattern in &patterns {
                        let matches =
                            pattern.match_list(policy_definition.parameters.keys(), &mut matcher);
                        if matches.is_empty() {
                            return false;
                        }
                    }
                    true
                },
            );
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &policy_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
