use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::EntraApplicationClientId;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
/// Register a named Entra credential source under an explicit tenant.
#[derive(Debug, Facet)]
pub struct AccountAddWorkloadIdentityArgs {
    #[facet(figue::named, proxy = String)]
    pub name: CloudTerrastodonAccountAlias,
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,
    #[facet(figue::named, proxy = String)]
    pub client_id: EntraApplicationClientId,
    #[facet(figue::named)]
    pub assertion_file: PathBuf,
}
impl AccountAddWorkloadIdentityArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let profile = CloudTerrastodonAccountStore::configured()
            .add(
                self.name,
                CloudTerrastodonAccountCredential::WorkloadIdentity {
                    tenant_id: self.tenant.resolve(cancellation).await?,
                    client_id: self.client_id,
                    assertion_file: std::path::absolute(self.assertion_file)?.try_into()?,
                },
                cancellation,
            )
            .await?;
        Ok(CliOutput::facet(profile))
    }
}
