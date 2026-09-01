use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
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
pub struct ApplicationRegistrationListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationRegistrationListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_application_registrations<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> ApplicationRegistrationListRequest<'a> {
    ApplicationRegistrationListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for ApplicationRegistrationListRequest<'_> {
    type Output = Vec<EntraApplicationRegistration>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.auth_context.tenant_id, "Fetching application registrations");
        let applications: Vec<EntraApplicationRegistration> = MicrosoftGraphHelper::new(
            "https://graph.microsoft.com/v1.0/applications",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_all()
        .await?;
        debug!(
            tenant_id = %self.auth_context.tenant_id,
            count = applications.len(),
            "Found application registrations"
        );
        Ok(applications)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn list_application_registrations() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::explicit_azure_cli().bind_to_azure_tenant(tenant_id)?;
        let result = fetch_all_application_registrations(&auth_context).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ApplicationRegistrationListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ApplicationRegistrationListRequest<'static> => Vec<EntraApplicationRegistration>);
