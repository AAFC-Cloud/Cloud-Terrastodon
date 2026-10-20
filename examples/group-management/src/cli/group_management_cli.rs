use super::command::GroupManagementPlanArgs;
use super::command::GroupManagementValidateArgs;
use cloud_terrastodon::app::ApplicationCli;
use cloud_terrastodon::app::AuthContext;
use cloud_terrastodon::app::CancellationScope;
use cloud_terrastodon::app::CliOutput;
use cloud_terrastodon::app::GlobalArgs;
use cloud_terrastodon::app::Result;
use cloud_terrastodon::app::facet;
use cloud_terrastodon::app::figue;

/// Validate desired Entra group state or compare it with existing groups.
#[derive(facet::Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
pub struct GroupManagementArgs {
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    #[facet(figue::subcommand)]
    pub command: GroupManagementCommand,
}

impl ApplicationCli for GroupManagementArgs {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

/// Read-only operations; applying a plan is a later milestone.
#[derive(facet::Facet)]
#[cfg_attr(test, derive(arbitrary::Arbitrary))]
#[repr(u8)]
pub enum GroupManagementCommand {
    /// Validate the desired-state file without requesting credentials or directory data.
    Validate(GroupManagementValidateArgs),
    /// Read existing groups and produce a reconciliation plan without changing them.
    Plan(GroupManagementPlanArgs),
}

impl GroupManagementArgs {
    pub async fn invoke(
        self,
        auth: &AuthContext,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
        match self.command {
            GroupManagementCommand::Validate(args) => args.invoke(),
            GroupManagementCommand::Plan(args) => Box::pin(args.invoke(auth, cancellation)).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::GroupManagementArgs;
    use cloud_terrastodon::app::Result;
    use cloud_terrastodon::app::figue;

    #[test]
    fn to_args_consistency() -> Result<()> {
        figue::assert_to_args_consistency::<GroupManagementArgs>(Default::default())?;
        Ok(())
    }

    #[test]
    fn to_args_roundtrip() -> Result<()> {
        figue::assert_to_args_roundtrip::<GroupManagementArgs>(figue::TestToArgsRoundTrip {
            success_count_global: 128,
            max_attempts_global: 10_000,
            // This schema has subcommands, so each leaf needs its own coverage target.
            success_count_per_leaf: 128,
            max_attempts_per_leaf: 10_000,
            ..Default::default()
        })?;
        Ok(())
    }
}
