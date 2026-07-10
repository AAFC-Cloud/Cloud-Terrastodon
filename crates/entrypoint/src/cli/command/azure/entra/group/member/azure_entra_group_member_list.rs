use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::fetch_group_members;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List the members of an Entra (Azure AD) group.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberListArgs {
    /// Entra group id (GUID).
    #[facet(figue::named, proxy = String)]
    pub group_id: EntraGroupId,

    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraGroupMemberListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, group_id = %self.group_id, "Fetching Entra group members");
        let members = fetch_group_members(tenant_id, self.group_id).await?;

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &members)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
