use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use eyre::ensure;
use eyre::Result;
use nucleo::Config;
use nucleo::Matcher;
use nucleo::pattern::AtomKind;
use nucleo::pattern::CaseMatching;
use nucleo::pattern::Normalization;
use nucleo::pattern::Pattern;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure policy set definitions.
#[derive(Args, Debug, Clone)]
pub struct AzurePolicySetDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
    /// Filter policy set definitions to those whose name satisfies the provided Nucleo fuzzy expression
    #[arg(long)]
    pub name: Option<String>,
}

impl AzurePolicySetDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!("Fetching Azure policy set definitions...");
        let mut policy_set_definitions = fetch_all_policy_set_definitions(tenant_id).await?;
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

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &policy_set_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
