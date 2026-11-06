use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_role_assignments;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Azure role assignments.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureRoleAssignmentListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureRoleAssignmentListArgs {
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
        info!("Fetching Azure role assignments");
        let role_assignments = fetch_all_role_assignments(&tenant_auth_context, executor).await?;
        info!(
            count = role_assignments.len(),
            "Fetched Azure role assignments"
        );

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &role_assignments)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
