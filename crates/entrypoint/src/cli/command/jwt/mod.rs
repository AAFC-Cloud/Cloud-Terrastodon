pub mod decode_cli;

use crate::cli::command::jwt::decode_cli::JwtDecodeArgs;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;

/// JWT-related commands.
#[derive(facet::Facet, Debug, Clone)]
pub struct JwtArgs {
    #[facet(figue::subcommand)]
    pub command: JwtCommand,
}

#[derive(facet::Facet, Debug, Clone)]
#[repr(u8)]
pub enum JwtCommand {
    /// Decode a JWT without validating its signature.
    Decode(JwtDecodeArgs),
}

impl JwtArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        self.command.invoke(cancellation).await
    }
}

impl JwtCommand {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<()> {
        match self {
            JwtCommand::Decode(args) => args.invoke(cancellation).await,
        }
    }
}
