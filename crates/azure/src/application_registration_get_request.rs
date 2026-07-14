use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_azure_types::EntraApplicationRegistrationId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use facet::Facet;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(arbitrary::Arbitrary, Facet)]
pub struct ApplicationRegistrationRequest {
    pub tenant_id: AzureTenantId,
    pub application_registration_id: EntraApplicationRegistrationId,
}

pub fn fetch_application_registration(
    tenant_id: AzureTenantId,
    application_registration_id: EntraApplicationRegistrationId,
) -> ApplicationRegistrationRequest {
    ApplicationRegistrationRequest {
        tenant_id,
        application_registration_id,
    }
}

impl ApplicationRegistrationRequest {
    fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/applications/{}",
            self.application_registration_id
        )
    }
}

#[async_trait]
impl CacheableCommand for ApplicationRegistrationRequest {
    type Output = EntraApplicationRegistration;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.tenant_id.to_string().as_str(),
            self.application_registration_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.tenant_id,
            application_registration_id = %self.application_registration_id,
            "Fetching application registration by object id"
        );
        MicrosoftGraphHelper::new(self.tenant_id, self.url(), Some(self.cache_key()))
            .fetch_one()
            .await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationRequest);
cloud_terrastodon_registry::register_thing!(ApplicationRegistrationRequest);
cloud_terrastodon_registry::register_arbitrary!(ApplicationRegistrationRequest);
cloud_terrastodon_registry::register_into_future!(ApplicationRegistrationRequest => EntraApplicationRegistration);
