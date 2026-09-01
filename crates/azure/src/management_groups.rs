use crate::ResourceGraphHelper;
use cloud_terrastodon_azure_types::ManagementGroup;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use eyre::bail;
use indoc::indoc;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::error;
use tracing::info;

pub async fn fetch_root_management_group(
    auth_context: &AzureTenantAuthContext,
) -> Result<ManagementGroup> {
    info!("Fetching root management group");
    let found = fetch_all_management_groups(auth_context)
        .await?
        .into_iter()
        .find(|mg| mg.name() == mg.tenant_id.to_string());
    match found {
        Some(management_group) => {
            info!("Found root management group");
            Ok(management_group)
        }
        None => {
            let msg = "Failed to find a management group with name matching the tenant ID";
            error!(msg);
            bail!(msg);
        }
    }
}

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct ManagementGroupListRequest<'a> {
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

pub fn fetch_all_management_groups<'a>(
    auth_context: &'a AzureTenantAuthContext,
) -> ManagementGroupListRequest<'a> {
    ManagementGroupListRequest {
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl<'a> arbitrary::Arbitrary<'a> for ManagementGroupListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

#[async_trait]
impl<'a> CacheableCommand for ManagementGroupListRequest<'a> {
    type Output = Vec<ManagementGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "resource_graph",
            "management_groups",
            self.auth_context.tenant_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        info!("Fetching management groups");
        let query = indoc! {r#"
        resourcecontainers
        | where type =~ "Microsoft.Management/managementGroups"
        | project 
            tenant_id=tenantId,
            id,
            display_name=properties.displayName,
            management_group_ancestors_chain=properties.details.managementGroupAncestorsChain
    "#};

        let management_groups =
            ResourceGraphHelper::new(query, Some(self.cache_key()), self.auth_context.as_ref())
                .collect_all::<ManagementGroup>()
                .await?;
        let count = management_groups.len();
        if count == 0 {
            bail!(
                "No management groups found for tenant '{}'",
                self.auth_context.tenant_id
            );
        }
        info!("Found {count} management groups");
        Ok(management_groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(ManagementGroupListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_management_groups(
            &AuthContext::explicit_azure_cli().bind_to_azure_tenant(get_test_tenant_id().await?)?,
        )
        .await?;
        assert!(!result.is_empty());
        Ok(())
    }
}

cloud_terrastodon_registry::register_thing!(ManagementGroupListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(ManagementGroupListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(ManagementGroupListRequest<'static> => Vec<ManagementGroup>);
