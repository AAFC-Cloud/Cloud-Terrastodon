use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use eyre::Result;
use facet::Facet;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

const USER_LIST_CACHE_DURATION: Duration = Duration::MAX;

#[must_use = "This is a future request, you must .await it"]
#[derive(Facet)]
pub struct EntraUserListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> Arbitrary<'a> for EntraUserListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: AzureTenantId::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_all_entra_users<'a>(
    tenant_id: AzureTenantId,
    auth_context: &'a AuthContext,
) -> EntraUserListRequest<'a> {
    EntraUserListRequest {
        tenant_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for EntraUserListRequest<'a> {
    type Output = Vec<EntraUser>;

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "users",
                self.tenant_id.to_string().as_str(),
            ]),
            valid_for: USER_LIST_CACHE_DURATION,
        }
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.tenant_id, "Fetching users");
        let helper = MicrosoftGraphHelper::new(
            self.tenant_id,
            "https://graph.microsoft.com/v1.0/users?$select=businessPhones,displayName,givenName,id,jobTitle,mail,otherMails,mobilePhone,officeLocation,preferredLanguage,surname,userPrincipalName",
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let users: Vec<EntraUser> = helper.fetch_all().await?;
        debug!("Found {} users", users.len());
        Ok(users)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraUserListRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(EntraUserListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraUserListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(Vec<EntraUser>);
cloud_terrastodon_registry::register_into_future!(EntraUserListRequest<'static> => Vec<EntraUser>);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::default();
        let result = fetch_all_entra_users(get_test_tenant_id().await?, &auth_context).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
