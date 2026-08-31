use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
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
pub struct ServicePrincipalListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ServicePrincipalListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_service_principals<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> ServicePrincipalListRequest<'a> {
    ServicePrincipalListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for ServicePrincipalListRequest<'a> {
    type Output = Vec<EntraServicePrincipal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "service_principals",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching service principals");
        let query = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/servicePrincipals",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let entries: Vec<EntraServicePrincipal> = query.fetch_all().await?;
        debug!("Found {} service principals", entries.len());
        Ok(entries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ServicePrincipalListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use crate::fetch_all_service_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_azure_types::EntraServicePrincipal;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let auth_context = AuthContext::default();
        let found: Vec<EntraServicePrincipal> =
            fetch_all_service_principals(get_test_tenant_id().await?, &auth_context).await?;
        assert!(found.len() > 10);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ServicePrincipalListRequest<'static> => Vec<EntraServicePrincipal>);
