use clap::Args;
use cloud_terrastodon_azure::prelude::add_group_member;
use cloud_terrastodon_azure::prelude::GroupId;
use cloud_terrastodon_azure::prelude::PrincipalId;
use eyre::Result;
use tracing::info;

/// Add a member to an Entra (Azure AD) group.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraGroupMemberAddArgs {
    /// Entra group id (GUID).
    #[arg(long = "group-id")]
    pub group_id: GroupId,

    /// Principal id to add to the group (user/service-principal/group GUID).
    #[arg(long = "member-id")]
    pub member_id: PrincipalId,
}

impl AzureEntraGroupMemberAddArgs {
    pub async fn invoke(self) -> Result<()> {
        add_group_member(self.group_id, self.member_id).await?;
        info!(group_id=%self.group_id, member_id=%self.member_id, "Added member to group");
        Ok(())
    }
}
