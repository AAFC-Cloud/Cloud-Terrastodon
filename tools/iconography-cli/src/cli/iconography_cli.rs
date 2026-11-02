use super::IconographyCheckArgs;
use super::IconographyGenerateArgs;
use super::IconographyUpdateArgs;
use cloud_terrastodon_app::ApplicationCli;
use cloud_terrastodon_app::CancellationScope;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_app::GlobalArgs;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_app::figue;
use facet::Facet;

/// Maintain the checked-in icon catalogues and generated asset crates.
#[derive(Facet)]
pub struct IconographyArgs {
    #[facet(flatten)]
    pub global_args: GlobalArgs,
    #[facet(figue::subcommand)]
    pub command: IconographyCommand,
}

/// Public-source updating and offline catalogue maintenance operations.
#[derive(Facet)]
#[repr(u8)]
pub enum IconographyCommand {
    /// Download selected public assets, record provenance, and regenerate.
    Update(IconographyUpdateArgs),
    /// Generate Rust APIs and Cargo category features from local inputs.
    Generate(IconographyGenerateArgs),
    /// Validate local inputs and fail if generated outputs have drifted.
    Check(IconographyCheckArgs),
}

impl ApplicationCli for IconographyArgs {
    fn global_args(&self) -> &GlobalArgs {
        &self.global_args
    }
}

impl IconographyArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        match self.command {
            IconographyCommand::Update(args) => Box::pin(args.invoke(cancellation)).await,
            IconographyCommand::Generate(args) => args.invoke(cancellation),
            IconographyCommand::Check(args) => args.invoke(cancellation),
        }
    }
}
