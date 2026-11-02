use cloud_terrastodon_app::App;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_iconography_cli::cli::IconographyArgs;

fn main() -> Result<()> {
    App::new("iconography", env!("CARGO_PKG_VERSION")).run(
        |cli: IconographyArgs, context| async move {
            let output_format = cli.global_args.output_format;
            let output = cli.invoke(&context.cancellation).await?;
            output.emit(output_format, &context.cancellation)
        },
    )
}
