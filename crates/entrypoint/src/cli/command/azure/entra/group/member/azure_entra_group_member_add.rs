use cloud_terrastodon_azure::EntraGroupId;
use cloud_terrastodon_azure::PrincipalId;
use cloud_terrastodon_azure::add_group_member;
use eyre::Result;
use tracing::info;

/// Add a member to an Entra (Azure AD) group.
#[derive(facet::Facet, Debug, Clone)]
pub struct AzureEntraGroupMemberAddArgs {
    /// Entra group id (GUID).
    #[facet(figue::named, opaque, proxy = String)]
    pub group_id: EntraGroupId,

    /// Principal id to add to the group (user/service-principal/group GUID).
    #[facet(figue::named, opaque, proxy = String)]
    pub member_id: PrincipalId,
}

impl AzureEntraGroupMemberAddArgs {
    pub async fn invoke(self) -> Result<()> {
        add_group_member(self.group_id, self.member_id).await?;
        info!(group_id=%self.group_id, member_id=%self.member_id, "Added member to group");
        Ok(())
    }
}
