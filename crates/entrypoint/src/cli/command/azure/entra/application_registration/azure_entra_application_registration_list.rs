use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_application_registrations;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) application registrations.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraApplicationRegistrationListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraApplicationRegistrationListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching application registrations");
        let applications = fetch_all_application_registrations(tenant_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &applications)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
