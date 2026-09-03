use crate::MicrosoftGraphHelper;
use arbitrary::Arbitrary;
use cloud_terrastodon_azure_types::EntraUser;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
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
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    #[facet(opaque, skip, default)]
    pub progress_hook: Option<crate::MicrosoftGraphProgressHook>,
}

impl<'a> Arbitrary<'a> for EntraUserListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(AzureTenantAuthContext::arbitrary(u)?),
            progress_hook: None,
        })
    }
}

pub fn fetch_all_entra_users<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> EntraUserListRequest<'a> {
    EntraUserListRequest {
        auth_context: Cow::Borrowed(auth_context),
        progress_hook: None,
    }
}

impl<'a> EntraUserListRequest<'a> {
    pub fn progress_hook<F>(mut self, progress_hook: F) -> Self
    where
        F: Fn(crate::MicrosoftGraphProgress) + Send + Sync + 'static,
    {
        self.progress_hook = Some(Box::new(progress_hook));
        self
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
                self.auth_context.tenant_id.to_string().as_str(),
            ]),
            valid_for: USER_LIST_CACHE_DURATION,
        }
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(tenant_id = %self.auth_context.tenant_id, "Fetching users");
        let mut helper = MicrosoftGraphHelper::new(
            format!(
                "https://graph.microsoft.com/v1.0/users?$select={}&$count=true",
                EntraUser::SELECT
            ),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .with_consistency_level_eventual();
        helper.progress_hook = self.progress_hook;
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
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let result = fetch_all_entra_users(&auth_context).await?;
        assert!(!result.is_empty());
        Ok(())
    }
}
