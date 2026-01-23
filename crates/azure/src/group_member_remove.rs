use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::PrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct GroupMemberRemoveRequest {
    pub group_id: GroupId,
    pub member_id: PrincipalId,
}

pub fn remove_group_member(
    group_id: impl Into<GroupId>,
    member_id: impl Into<PrincipalId>,
) -> GroupMemberRemoveRequest {
    GroupMemberRemoveRequest {
        group_id: group_id.into(),
        member_id: member_id.into(),
    }
}

#[async_trait]
impl CacheableCommand for GroupMemberRemoveRequest {
    type Output = ();

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "az",
                "ad",
                "group",
                "member",
                "remove",
                self.group_id.to_string().as_str(),
                self.member_id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args([
            "ad",
            "group",
            "member",
            "remove",
            "--group",
            self.group_id.to_string().as_str(),
            "--member-id",
            self.member_id.to_string().as_str(),
        ]);
        cmd.cache(self.cache_key());
        cmd.run_raw().await?;
        Ok(())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupMemberRemoveRequest);
