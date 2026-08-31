use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::MicrosoftGraphOrganization;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use eyre::ensure;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct AzureTenantDetailsRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for AzureTenantDetailsRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_azure_tenant_details<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> AzureTenantDetailsRequest<'a> {
    AzureTenantDetailsRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for AzureTenantDetailsRequest<'_> {
    type Output = MicrosoftGraphOrganization;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "organization",
            self.tenant_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = "https://graph.microsoft.com/v1.0/organization";
        let resp = MicrosoftGraphHelper::new(
            self.tenant_id,
            url,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_all::<MicrosoftGraphOrganization>()
        .await?;
        ensure!(
            resp.len() == 1,
            "Expected exactly one organization in response, got {}",
            resp.len()
        );
        Ok(resp.into_iter().next().unwrap())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(AzureTenantDetailsRequest<'a>, 'a);

#[cfg(test)]
mod test {
    use crate::fetch_azure_tenant_details;
    use crate::list_tracked_tenants;
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::ensure;
    use std::collections::HashSet;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let tenants = list_tracked_tenants().await?;
        ensure!(!tenants.is_empty(), "Expected at least one tracked tenant");
        let mut seen = HashSet::new();
        let auth_context = AuthContext::default();
        for tenant_id in tenants {
            let details = fetch_azure_tenant_details(tenant_id, &auth_context).await?;
            let unique = seen.insert(details.entity.id.clone());
            ensure!(unique, "Duplicate tenant ID found: {}", details.entity.id);
        }
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(AzureTenantDetailsRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(AzureTenantDetailsRequest<'static>);
cloud_terrastodon_registry::register_into_future!(AzureTenantDetailsRequest<'static> => MicrosoftGraphOrganization);
