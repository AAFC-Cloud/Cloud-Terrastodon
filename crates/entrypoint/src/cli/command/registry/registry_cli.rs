use super::entry::RegistryEntryArgs;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Inspect the operations linked into this executable's registry.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct RegistryArgs {
    #[facet(figue::subcommand)]
    pub command: RegistryCommand,
}

/// Registry metadata inspection commands.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[repr(u8)]
pub enum RegistryCommand {
    /// Inspect registered operations and their declared effects.
    Entry(RegistryEntryArgs),
}

impl RegistryArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        self.command.invoke(cancellation).await
    }
}

impl RegistryCommand {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self {
            Self::Entry(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}

cloud_terrastodon_registry::register_thing!(RegistryArgs);
cloud_terrastodon_registry::register_arbitrary!(RegistryArgs);
cloud_terrastodon_registry::register_thing!(RegistryCommand);
cloud_terrastodon_registry::register_arbitrary!(RegistryCommand);
