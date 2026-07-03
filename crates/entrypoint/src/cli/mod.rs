pub mod command;
pub mod global_args;
pub(crate) mod scalar_args;

use crate::menu::menu_loop;
pub use command::*;
pub use global_args::GlobalArgs;
use teamy_cancellation::CancellationToken;

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

cloud_terrastodon_registry::register_thing!(Cli);
impl Cli {
    pub async fn invoke(self, cancellation_token: &CancellationToken) -> eyre::Result<()> {
        match self.command {
            Some(cmd) => cmd.invoke(cancellation_token).await,
            None => {
                menu_loop().await?;
                Ok(())
            }
        }
    }
}

