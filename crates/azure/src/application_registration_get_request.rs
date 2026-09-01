use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraApplicationClientId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Facet)]
pub struct ApplicationRegistrationGetRequest<'a> {
    pub application_id: EntraApplicationClientId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for ApplicationRegistrationGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            application_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_application_registration<'a>(
    application_id: EntraApplicationClientId,
    auth_context: &'a AzureTenantAuthContext,
) -> ApplicationRegistrationGetRequest<'a> {
    ApplicationRegistrationGetRequest {
        application_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl ApplicationRegistrationGetRequest<'_> {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/applications/{}",
            self.application_id
        )
    }
}

#[async_trait]
impl CacheableCommand for ApplicationRegistrationGetRequest<'_> {
    type Output = EntraApplicationRegistration;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.auth_context.tenant_id.to_string().as_str(),
            self.application_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.auth_context.tenant_id,
            application_registration_id = %self.application_id,
            "Fetching application registration by object id"
        );
        MicrosoftGraphHelper::new(
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_one()
        .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationGetRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(ApplicationRegistrationGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationGetRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ApplicationRegistrationGetRequest<'static> => EntraApplicationRegistration);
