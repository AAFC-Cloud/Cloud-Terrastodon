use super::account_default_clear_cli::AccountDefaultClearArgs;
use super::account_default_set_cli::AccountDefaultSetArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
/// Configure an explicit account default within its tenant or service namespace.
#[derive(Debug, Facet)]
pub struct AccountDefaultArgs {
    #[facet(figue::subcommand)]
    pub command: AccountDefaultCommand,
}
#[derive(Debug, Facet)]
#[repr(u8)]
pub enum AccountDefaultCommand {
    Set(AccountDefaultSetArgs),
    Clear(AccountDefaultClearArgs),
}
impl AccountDefaultArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            AccountDefaultCommand::Set(args) => Box::pin(args.invoke(cancellation)).await,
            AccountDefaultCommand::Clear(args) => Box::pin(args.invoke(cancellation)).await,
        }
    }
}
