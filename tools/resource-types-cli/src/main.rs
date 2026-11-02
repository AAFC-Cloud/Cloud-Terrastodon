use cloud_terrastodon_app::App;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_resource_types_cli::ResourceTypesArgs;

fn main() -> Result<()> {
    App::new("resource-types", env!("CARGO_PKG_VERSION"))
        .without_authentication()
        .run(|cli: ResourceTypesArgs, context| async move {
            let output = cli.invoke(&context.cancellation).await?;
            output.emit(cli.global_args.output_format, &context.cancellation)
        })
}
