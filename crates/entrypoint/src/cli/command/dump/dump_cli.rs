use super::dump_plan_cli::DumpPlanArgs;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_credentials::AuthSource;
use eyre::Result;

/// Plan cloud inventories from the linked request registry.
#[derive(Debug, Arbitrary, facet::Facet)]
pub struct DumpArgs {
    #[facet(figue::subcommand)]
    pub command: DumpCommand,
}

#[derive(Debug, Arbitrary, facet::Facet)]
#[repr(u8)]
pub enum DumpCommand {
    /// Plan read-only request dependencies offline and optionally export Excalidraw.
    Plan(DumpPlanArgs),
}

impl DumpArgs {
    pub async fn invoke(
        self,
        auth_source: AuthSource,
        cancellation: &CancellationScope,
    ) -> Result<CliOutput> {
        match self.command {
            DumpCommand::Plan(args) => Box::pin(args.invoke(auth_source, cancellation)).await,
        }
    }
}

cloud_terrastodon_registry::register_thing!(DumpArgs);
cloud_terrastodon_registry::register_arbitrary!(DumpArgs);
cloud_terrastodon_registry::register_thing!(DumpCommand);
cloud_terrastodon_registry::register_arbitrary!(DumpCommand);
