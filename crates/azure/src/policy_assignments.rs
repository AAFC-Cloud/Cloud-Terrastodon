use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::PolicyAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct PolicyAssignmentListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_policy_assignments<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> PolicyAssignmentListRequest<'a> {
    PolicyAssignmentListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for PolicyAssignmentListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for PolicyAssignmentListRequest<'a> {
    type Output = Vec<PolicyAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "policy_assignments",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let mut qb = ResourceGraphHelper::new(
            r#"
policyresources
| where type =~ "microsoft.authorization/policyassignments"
| project 
 id,
 name,
 location,
 identity,
 properties
    "#,
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        qb.collect_all().await
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(PolicyAssignmentListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_policy_assignments(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(PolicyAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(PolicyAssignmentListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(PolicyAssignmentListRequest<'static> => Vec<PolicyAssignment>);
