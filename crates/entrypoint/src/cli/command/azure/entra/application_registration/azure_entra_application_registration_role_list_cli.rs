use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_azure::fetch_application_roles;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List app roles and resource-specific application permissions for an app id.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationRoleListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Application (client) id of the resource service principal.
    #[facet(figue::positional)]
    pub application_id: EntraApplicationClientId,
}

impl AzureEntraApplicationRegistrationRoleListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(
            %tenant_id,
            application_id = %self.application_id,
            "Fetching application roles"
        );
        let permissions = fetch_application_roles(tenant_id, self.application_id).await?;

        let stdout: std::io::Stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &permissions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
