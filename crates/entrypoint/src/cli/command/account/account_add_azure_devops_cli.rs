use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::CancellationScope;
use eyre::Result;
use facet::Facet;
/// Register a PAT account in an explicit organization namespace, without a token.
#[derive(Debug, Facet)]
pub struct AccountAddAzureDevOpsArgs {
    #[facet(figue::named, proxy = String)]
    pub name: CloudTerrastodonAccountAlias,
    #[facet(figue::named)]
    pub org: AzureDevOpsOrganizationUrl,
}
impl AccountAddAzureDevOpsArgs {
    pub async fn invoke(self, cancellation: &CancellationScope) -> Result<CliOutput> {
        Ok(CliOutput::facet(
            CloudTerrastodonAccountStore::configured()
                .add(
                    self.name,
                    CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                        organization: self.org,
                    },
                    cancellation,
                )
                .await?,
        ))
    }
}
