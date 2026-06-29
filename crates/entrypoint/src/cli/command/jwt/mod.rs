pub mod decode;

use crate::cli::command::jwt::decode::JwtDecodeArgs;
use clap::Args;
use clap::Subcommand;
use eyre::Result;

/// JWT-related commands.
#[derive(Args, Debug, Clone)]
pub struct JwtArgs {
    #[command(subcommand)]
    pub command: JwtCommand,
}

#[derive(Subcommand, Debug, Clone)]
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
