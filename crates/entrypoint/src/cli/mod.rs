pub mod command;
pub mod global_args;

pub mod prelude {
    pub use super::Cli;
    pub use super::GlobalArgs;
    pub use super::command::*;
}

use crate::menu::menu_loop;
use clap::Parser;
pub use command::*;
pub use global_args::GlobalArgs;

/// The primary entrypoint for command-line parsing.
#[derive(Parser, Debug)]
#[command(name = "cloud_terrastodon", about, long_about = None)]
pub struct Cli {
    #[command(flatten)]
    pub global_args: GlobalArgs,

    #[command(subcommand)]
    pub command: Option<CloudTerrastodonCommand>,
}

impl Cli {
    pub async fn invoke(self) -> eyre::Result<()> {
        match self.command {
            Some(cmd) => cmd.invoke().await,
            None => {
                menu_loop().await?;
                Ok(())
            }
        }
    }
}
