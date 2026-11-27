use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_azure::get_default_tenant_id;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::io::Error;
use std::io::ErrorKind;

/// Show the default tenant saved by `ct az tenant default`.
#[derive(Facet, Debug, Clone)]
pub struct AzureTenantShowDefaultArgs {}

impl AzureTenantShowDefaultArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let tenant = match get_default_tenant_id(cancellation).await {
            Ok(tenant) => Some(tenant),
            Err(error)
                if error
                    .downcast_ref::<Error>()
                    .is_some_and(|source| source.kind() == ErrorKind::NotFound) =>
            {
                None
            }
            Err(error) => return Err(error),
        };
        Ok(CliOutput::facet(tenant))
    }
}
