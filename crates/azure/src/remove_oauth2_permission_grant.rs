use crate::bust_oauth2_permission_grants_cache;
use cloud_terrastodon_azure_types::OAuth2PermissionGrantId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::borrow::Cow;
use std::path::PathBuf;
use std::time::Duration;

#[derive(facet::Facet)]
pub struct OAuth2PermissionGrantRemoveRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
    pub id: OAuth2PermissionGrantId,
}

impl<'a> arbitrary::Arbitrary<'a> for OAuth2PermissionGrantRemoveRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
            id: arbitrary::Arbitrary::arbitrary(u)?,
        })
    }
}

pub fn remove_oauth2_permission_grant<'a>(
    auth_context: &'a AzureTenantAuthContext,
    id: OAuth2PermissionGrantId,
) -> OAuth2PermissionGrantRemoveRequest<'a> {
    OAuth2PermissionGrantRemoveRequest {
        auth_context: Cow::Borrowed(auth_context),
        id,
    }
}

#[async_trait]
impl CacheableCommand for OAuth2PermissionGrantRemoveRequest<'_> {
    type Output = ();

    fn cache_key(&self) -> CacheKey {
        CacheKey {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "DELETE",
                "oauth2PermissionGrants",
                self.auth_context.tenant_id.to_string().as_str(),
                self.id.to_string().as_str(),
            ]),
            valid_for: Duration::ZERO,
        }
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let cache_key = self.cache_key();
        let url = format!(
            "https://graph.microsoft.com/v1.0/oauth2PermissionGrants/{}",
            self.id
        );
        RestRequest::new(Method::DELETE, &url)?
            .tenant(self.auth_context.tenant_id)
            .auth_context(&self.auth_context.auth_context)
            .cache(cache_key)
            .receive_raw()
            .await?;
        bust_oauth2_permission_grants_cache(self.auth_context.tenant_id).await?;
        Ok(())
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionGrantRemoveRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(OAuth2PermissionGrantRemoveRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionGrantRemoveRequest<'static>);
cloud_terrastodon_registry::register_into_future!(OAuth2PermissionGrantRemoveRequest<'static> => ());
