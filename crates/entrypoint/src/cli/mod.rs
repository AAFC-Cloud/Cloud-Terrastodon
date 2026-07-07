use arbitrary::Arbitrary;
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

impl<'a> Arbitrary<'a> for Cli {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            global_args: GlobalArgs::arbitrary(u)?,
            builtins: figue::FigueBuiltins::default(),
            command: Option::<CloudTerrastodonCommand>::arbitrary(u)?,
        })
    }
}
cloud_terrastodon_registry::register_thing!(Cli);
cloud_terrastodon_registry::register_arbitrary!(Cli);
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
