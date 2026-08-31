use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct ServicePrincipalRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ServicePrincipalRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_service_principal<'a>(
    tenant_id: AzureTenantId,
    service_principal_id: EntraServicePrincipalObjectId,
    auth_context: &'a AuthContext,
) -> ServicePrincipalRequest<'a> {
    ServicePrincipalRequest {
        tenant_id,
        service_principal_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl ServicePrincipalRequest<'_> {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals/{}",
            self.service_principal_id
        )
    }
}

#[async_trait]
impl CacheableCommand for ServicePrincipalRequest<'_> {
    type Output = EntraServicePrincipal;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.tenant_id.to_string().as_str(),
            self.service_principal_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.tenant_id,
            service_principal_id = %self.service_principal_id,
            "Fetching service principal by object id"
        );
        MicrosoftGraphHelper::new(
            self.tenant_id,
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_one()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServicePrincipalRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(ServicePrincipalRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ServicePrincipalRequest<'static> => EntraServicePrincipal);
