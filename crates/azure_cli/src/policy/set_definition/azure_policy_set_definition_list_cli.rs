use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_policy_set_definitions;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
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

/// Arguments for listing Azure policy set definitions.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicySetDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
    /// Filter policy set definitions to those whose name satisfies the provided Nucleo fuzzy expression
    #[facet(figue::named)]
    pub name: Option<String>,
}

impl AzurePolicySetDefinitionListArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let tenant_auth_context = self
            .tenant
            .bind_auth_context(auth_context, cancellation)
            .await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        info!("Fetching Azure policy set definitions...");
        let mut policy_set_definitions =
            fetch_all_policy_set_definitions(&tenant_auth_context, executor).await?;
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
        facet_json::to_writer_std_pretty(&mut handle, &policy_set_definitions)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
