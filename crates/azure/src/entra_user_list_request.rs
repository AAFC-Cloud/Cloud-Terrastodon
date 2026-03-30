use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
pub struct EntraUserListRequest {
    pub tenant_id: AzureTenantId,
}

pub fn fetch_all_entra_users(tenant_id: AzureTenantId) -> EntraUserListRequest {
    EntraUserListRequest { tenant_id }
}

#[async_trait]
impl CacheableCommand for EntraUserListRequest {
    type Output = Vec<EntraUser>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "users",
            self.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, "Fetching users");
        let users: Vec<EntraUser> = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/users?$select=businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName",
            Some(self.cache_key()),
        )
        .fetch_all()
        .await?;
        debug!("Found {} users", users.len());
        Ok(users)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_entra_users(get_test_tenant_id().await?).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
