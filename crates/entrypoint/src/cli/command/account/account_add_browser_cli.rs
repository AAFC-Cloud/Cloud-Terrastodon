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
/// Register a named Entra credential source under an explicit tenant.
#[derive(Debug, Facet)]
pub struct AccountAddBrowserArgs {
    #[facet(figue::named, proxy = String)]
    pub name: CloudTerrastodonAccountAlias,
    #[facet(figue::named)]
    pub tenant: AzureTenantArgument<'static>,
    #[facet(figue::named, proxy = String)]
    pub client_id: EntraApplicationClientId,
}
impl AccountAddBrowserArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        let profile = CloudTerrastodonAccountStore::configured()
            .add(
                self.name,
                CloudTerrastodonAccountCredential::Browser {
                    tenant_id: self.tenant.resolve(cancellation).await?,
                    client_id: self.client_id,
                },
                cancellation,
            )
            .await?;
        Ok(CliOutput::facet(profile))
    }
}
