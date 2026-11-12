use super::TestListArgs;
use super::TestRunArgs;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use facet::Facet;

/// Inspect or execute explicitly registered runtime verification cases.
#[derive(Debug, Facet)]
pub struct TestArgs {
    #[facet(figue::subcommand)]
    pub command: TestCommand,
}

#[derive(Debug, Facet)]
#[repr(u8)]
#[expect(
    clippy::large_enum_variant,
    reason = "Figue requires a struct subcommand payload; invocation futures are boxed at dispatch"
)]
pub enum TestCommand {
    /// List verification metadata without loading accounts or contacting services.
    List(TestListArgs),
    /// Execute cases whose registered names match a pattern.
    Run(TestRunArgs),
}

impl TestArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> eyre::Result<CliOutput> {
        match self.command {
            TestCommand::List(args) => args.invoke(),
            TestCommand::Run(args) => Box::pin(args.invoke(executor)).await,
        }
    }
}
