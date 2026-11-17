use cloud_terrastodon_app::App;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_iconography_cli::cli::IconographyArgs;

fn main() -> Result<()> {
    App::new("iconography", env!("CARGO_PKG_VERSION"))
        .run(|cli: IconographyArgs, context| async move { cli.invoke(&context.cancellation).await })
}
