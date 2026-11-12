use cloud_terrastodon_app::App;
use cloud_terrastodon_app::Result;
use cloud_terrastodon_auth::CredentialAuthenticationProvider;
use cloud_terrastodon_resource_types_cli::ResourceTypesArgs;
use std::sync::Arc;

fn main() -> Result<()> {
    App::new("resource-types", env!("CARGO_PKG_VERSION"))
        .authentication_provider(Arc::new(CredentialAuthenticationProvider::configured()))
        .run(|cli: ResourceTypesArgs, context| async move {
            let output = cli.invoke(&context.executor()).await?;
            output.emit(cli.global_args.output_format, &context.cancellation)
        })
}
