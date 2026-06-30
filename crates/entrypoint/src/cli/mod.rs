pub mod command;
pub mod global_args;
pub(crate) mod scalar_args;

use crate::menu::menu_loop;
pub use command::*;
pub use global_args::GlobalArgs;

/// The primary entrypoint for command-line parsing.
#[derive(facet::Facet, Debug)]
#[facet(rename = "cloud_terrastodon")]
pub struct Cli {
    #[facet(flatten)]
    pub global_args: GlobalArgs,

    #[facet(flatten)]
    pub builtins: figue::FigueBuiltins,

    #[facet(figue::subcommand, default)]
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
