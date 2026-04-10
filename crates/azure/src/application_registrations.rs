use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraApplicationRegistration;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct ApplicationRegistrationListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_application_registrations(
    tenant_id: AzureTenantId,
) -> ApplicationRegistrationListRequest {
    ApplicationRegistrationListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for ApplicationRegistrationListRequest {
    type Output = Vec<EntraApplicationRegistration>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "applications",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, "Fetching application registrations");
        let applications: Vec<EntraApplicationRegistration> = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/applications",
            Some(self.cache_key()),
        )
        .fetch_all()
        .await?;
        debug!(
            tenant_id = %self.tenant_id,
            count = applications.len(),
            "Found application registrations"
        );
        Ok(applications)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ApplicationRegistrationListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn list_application_registrations() -> Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let result = fetch_all_application_registrations(tenant_id).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
