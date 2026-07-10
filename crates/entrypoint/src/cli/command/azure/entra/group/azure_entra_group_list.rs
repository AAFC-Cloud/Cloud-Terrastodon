use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::PrincipalId;
use cloud_terrastodon_azure::fetch_all_groups;
use cloud_terrastodon_azure::fetch_entra_groups_for_member;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// List Entra (Azure AD) groups.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Only list groups containing this principal, including nested group membership.
    #[facet(figue::named)]
    pub for_member: Option<PrincipalId>,
}

impl AzureEntraGroupListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        let groups = match self.for_member {
            Some(principal_id) => {
                info!(%tenant_id, %principal_id, "Fetching Entra groups for principal");
                fetch_entra_groups_for_member(tenant_id, principal_id).await?
            }
            None => {
                info!(%tenant_id, "Fetching Entra groups");
                fetch_all_groups(tenant_id).await?
            }
        };

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &groups)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
