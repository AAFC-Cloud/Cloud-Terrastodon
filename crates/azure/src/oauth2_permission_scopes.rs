use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OAuth2PermissionScope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_rest::RestRequest;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[derive(facet::Facet)]
pub struct OAuth2PermissionScopesListRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for OAuth2PermissionScopesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_oauth2_permission_scopes<'a>(
    tenant_id: AzureTenantId,
    service_principal_id: EntraServicePrincipalObjectId,
    auth_context: &'a AuthContext,
) -> OAuth2PermissionScopesListRequest<'a> {
    OAuth2PermissionScopesListRequest {
        tenant_id,
        service_principal_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> cloud_terrastodon_command::CacheableCommand for OAuth2PermissionScopesListRequest<'a> {
    type Output = Vec<OAuth2PermissionScope>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "oauth2_permission_scopes",
            self.tenant_id.to_string().as_ref(),
            self.service_principal_id.to_string().as_ref(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        info!(
            "Fetching OAuth2 permission scopes for {:?}",
            self.service_principal_id
        );
        let url = format!(
            "https://graph.microsoft.com/v1.0/servicePrincipals/{service_principal_id}?$select=oauth2PermissionScopes",
            service_principal_id = self.service_principal_id
        );
        #[derive(facet::Facet)]
        struct Response {
            #[facet(rename = "oauth2PermissionScopes")]
            oauth2_permission_scopes: Vec<OAuth2PermissionScope>,
        }
        let entries = RestRequest::new(http::Method::GET, url.as_str())?
            .auth_context(self.auth_context.as_ref())
            .tenant(self.tenant_id)
            .cache(self.cache_key())
            .receive::<Response>()
            .await?
            .oauth2_permission_scopes;

        info!("Found {} OAuth2 permission scopes", entries.len());
        Ok(entries)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(OAuth2PermissionScopesListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_service_principals;
    use crate::get_test_tenant_id;
    use eyre::OptionExt;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::default();
        let service_principals = fetch_all_service_principals(tenant_id, &auth_context).await?;
        let graph = service_principals
            .iter()
            .find(|sp| sp.display_name == "Microsoft Graph")
            .ok_or_eyre("Failed to find graph sp")?;
        let scopes = fetch_oauth2_permission_scopes(tenant_id, graph.id, &auth_context).await?;
        assert!(scopes.len() > 10);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(OAuth2PermissionScopesListRequest<'static> => Vec<OAuth2PermissionScope>);
