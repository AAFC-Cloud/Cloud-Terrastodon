use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_azure_types::EntraUserId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Arbitrary, Facet)]
pub struct EntraUserGetRequest {
    pub tenant_id: AzureTenantId,
    pub user_id: EntraUserId,
}

pub fn fetch_entra_user(tenant_id: AzureTenantId, user_id: EntraUserId) -> EntraUserGetRequest {
    EntraUserGetRequest { tenant_id, user_id }
}

impl EntraUserGetRequest {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/users/{}?$select=businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName",
            self.user_id
        )
    }
}

#[async_trait]
impl CacheableCommand for EntraUserGetRequest {
    type Output = EntraUser;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "users",
            self.tenant_id.to_string().as_str(),
            self.user_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.tenant_id,
            user_id = %self.user_id,
            "Fetching user by object id"
        );
        MicrosoftGraphHelper::new(self.tenant_id, self.url(), Some(self.cache_key()))
            .fetch_one()
            .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserGetRequest);
cloud_terrastodon_registry::register_thing!(EntraUserGetRequest);
cloud_terrastodon_registry::register_arbitrary!(EntraUserGetRequest);
cloud_terrastodon_registry::register_into_future!(EntraUserGetRequest => EntraUser);
