use super::RegistryEntryListArgs;
use arbitrary::Arbitrary;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Inspect the registry's operation entries.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
pub struct RegistryEntryArgs {
    #[facet(figue::subcommand)]
    pub command: RegistryEntryCommand,
}

/// Commands for auditing registered operation metadata.
#[derive(Debug, Clone, Arbitrary, facet::Facet)]
#[repr(u8)]
pub enum RegistryEntryCommand {
    /// List registered operations, effects, signatures, and source locations.
    List(RegistryEntryListArgs),
}

impl RegistryEntryArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        self.command.invoke(cancellation).await
    }
}

impl RegistryEntryCommand {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self {
            Self::List(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}

cloud_terrastodon_registry::register_thing!(RegistryEntryArgs);
cloud_terrastodon_registry::register_arbitrary!(RegistryEntryArgs);
cloud_terrastodon_registry::register_thing!(RegistryEntryCommand);
cloud_terrastodon_registry::register_arbitrary!(RegistryEntryCommand);
