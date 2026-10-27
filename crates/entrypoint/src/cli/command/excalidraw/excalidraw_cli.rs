use super::excalidraw_interactive_cli::ExcalidrawInteractiveArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// Work with editable Excalidraw documents on disk.
#[derive(Debug, facet::Facet)]
pub struct ExcalidrawArgs {
    #[facet(figue::subcommand)]
    pub command: ExcalidrawCommand,
}

#[derive(Debug, facet::Facet)]
#[repr(u8)]
pub enum ExcalidrawCommand {
    /// Watch a drawing, display canvas answers, and publish live test updates and logs.
    Interactive(ExcalidrawInteractiveArgs),
}

impl ExcalidrawArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            ExcalidrawCommand::Interactive(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}

cloud_terrastodon_registry::register_thing!(ExcalidrawArgs);
cloud_terrastodon_registry::register_thing!(ExcalidrawCommand);
