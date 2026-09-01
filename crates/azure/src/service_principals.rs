use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraServicePrincipal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct ServicePrincipalListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ServicePrincipalListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_service_principals<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> ServicePrincipalListRequest<'a> {
    ServicePrincipalListRequest {
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
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching service principals");
        let query = MicrosoftGraphHelper::new(
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
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let found: Vec<EntraServicePrincipal> = fetch_all_service_principals(&auth_context).await?;
        assert!(found.len() > 10);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ServicePrincipalListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ServicePrincipalListRequest<'static> => Vec<EntraServicePrincipal>);
