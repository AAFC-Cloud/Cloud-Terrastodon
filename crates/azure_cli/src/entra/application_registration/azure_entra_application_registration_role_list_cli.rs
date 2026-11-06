use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::fetch_application_roles;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use facet::Facet;
use std::io::Write;
use tracing::info;

/// List app roles and resource-specific application permissions for an app id.
#[derive(Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationRoleListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Application (client) id of the resource service principal.
    #[facet(figue::positional)]
    pub application_id: EntraApplicationClientId,
}

impl AzureEntraApplicationRegistrationRoleListArgs {
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
        let tenant_id = tenant_auth_context.tenant_id;
        info!(
            %tenant_id,
            application_id = %self.application_id,
            "Fetching application roles"
        );
        let permissions = fetch_application_roles(
            tenant_id,
            self.application_id,
            tenant_auth_context.auth_context.azure_bearer_source()?,
            executor,
        )
        .await?;

        let stdout: std::io::Stdout = std::io::stdout();
        let mut handle = stdout.lock();
        facet_json::to_writer_std_pretty(&mut handle, &permissions)
            .map_err(|error| eyre::eyre!("{error:?}"))?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
