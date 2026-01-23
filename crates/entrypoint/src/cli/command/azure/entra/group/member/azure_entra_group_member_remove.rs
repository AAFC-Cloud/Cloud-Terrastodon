use clap::Args;
use cloud_terrastodon_azure::prelude::GroupId;
use cloud_terrastodon_azure::prelude::PrincipalId;
use cloud_terrastodon_azure::prelude::remove_group_member;
use eyre::Result;
use tracing::info;

/// Remove a member from an Entra (Azure AD) group.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupMemberRemoveArgs {
    /// Entra group id (GUID).
    #[arg(long = "group-id")]
    pub group_id: GroupId,

    /// Principal id to remove from the group (user/service-principal/group GUID).
    #[arg(long = "member-id")]
    pub member_id: PrincipalId,
}

impl AzureEntraGroupMemberRemoveArgs {
    pub async fn invoke(self) -> Result<()> {
        remove_group_member(self.group_id, self.member_id).await?;
        info!(group_id=%self.group_id, member_id=%self.member_id, "Removed member from group");
        Ok(())
    }
}
