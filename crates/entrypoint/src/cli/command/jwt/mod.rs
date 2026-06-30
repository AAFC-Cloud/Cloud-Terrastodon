pub mod decode;

use crate::cli::command::jwt::decode::JwtDecodeArgs;
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
    pub async fn invoke(self) -> Result<()> {
        self.command.invoke().await
    }
}

impl JwtCommand {
    pub async fn invoke(self) -> Result<()> {
        match self {
            JwtCommand::Decode(args) => args.invoke().await,
        }
    }
}
