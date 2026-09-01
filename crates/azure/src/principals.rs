use crate::fetch_all_entra_users;
use crate::fetch_all_security_groups;
use crate::fetch_all_service_principals;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_azure_types::PrincipalCollection;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use itertools::Itertools;
use std::borrow::Cow;
use std::path::PathBuf;
use tokio::try_join;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct PrincipalListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for PrincipalListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_all_principals<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> PrincipalListRequest<'a> {
    PrincipalListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl<'a> CacheableCommand for PrincipalListRequest<'a> {
    type Output = PrincipalCollection;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "principals",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!("Fetching principals (users, security groups, and service principals)");
        let auth_context = self.auth_context;
        let (users, security_groups, service_principals) = try_join!(
            fetch_all_entra_users(auth_context.as_ref()),
            fetch_all_security_groups(auth_context.as_ref()),
            fetch_all_service_principals(auth_context.as_ref())
        )?;
        let principals: Vec<Principal> = users
            .into_iter()
            .map_into()
            .chain(security_groups.into_iter().map_into())
            .chain(service_principals.into_iter().map_into())
            .collect();
        debug!("Found {} principals", principals.len());
        Ok(PrincipalCollection::new(principals))
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PrincipalListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use crate::fetch_all_principals;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> eyre::Result<()> {
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(get_test_tenant_id().await?)?;
        let found = fetch_all_principals(&auth_context).await?;
        assert!(found.len() > 10);
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrincipalListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PrincipalCollection);
cloud_terrastodon_registry::register_into_future!(PrincipalListRequest<'static> => PrincipalCollection);
