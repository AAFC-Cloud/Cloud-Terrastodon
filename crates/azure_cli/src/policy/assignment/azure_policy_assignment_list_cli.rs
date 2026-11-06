use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_policy_assignments;
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

/// Arguments for listing Azure policy assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzurePolicyAssignmentListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
    /// Filter policy assignments to those whose name satisfies the provided Nucleo fuzzy expression
    #[facet(figue::named)]
    pub name: Option<String>,
}

impl AzurePolicyAssignmentListArgs {
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
        info!("Fetching Azure policy assignments...");
        let mut policy_assignments =
            fetch_all_policy_assignments(&tenant_auth_context, executor).await?;
        info!(
            count = policy_assignments.len(),
            "Fetched Azure policy assignments",
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
            policy_assignments.retain(|policy_assignment| {
                !pattern
                    .match_list([policy_assignment.name.as_str()], &mut matcher)
                    .is_empty()
            });
        }

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &policy_assignments)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
