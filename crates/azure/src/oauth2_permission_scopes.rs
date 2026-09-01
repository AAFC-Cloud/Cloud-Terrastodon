use cloud_terrastodon_azure_types::EntraServicePrincipalObjectId;
use cloud_terrastodon_azure_types::OAuth2PermissionScope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use cloud_terrastodon_rest::RestRequest;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::info;

#[derive(facet::Facet)]
pub struct OAuth2PermissionScopesListRequest<'a> {
    pub service_principal_id: EntraServicePrincipalObjectId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for OAuth2PermissionScopesListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            service_principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_oauth2_permission_scopes<'a>(
    service_principal_id: EntraServicePrincipalObjectId,
    auth_context: &'a AzureTenantAuthContext,
) -> OAuth2PermissionScopesListRequest<'a> {
    OAuth2PermissionScopesListRequest {
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
            self.auth_context.tenant_id.to_string().as_ref(),
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
            .auth_context(&self.auth_context.auth_context)
            .tenant(self.auth_context.tenant_id)
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
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::OptionExt;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let tenant_auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
        let service_principals = fetch_all_service_principals(&tenant_auth_context).await?;
        let graph = service_principals
            .iter()
            .find(|sp| sp.display_name == "Microsoft Graph")
            .ok_or_eyre("Failed to find graph sp")?;
        let scopes = fetch_oauth2_permission_scopes(graph.id, &tenant_auth_context).await?;
        assert!(scopes.len() > 10);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(OAuth2PermissionScopesListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(OAuth2PermissionScopesListRequest<'static> => Vec<OAuth2PermissionScope>);
