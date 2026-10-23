use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraDirectoryObject;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

/// List a group's direct owners as shallow Microsoft Graph directory objects.
///
/// Follows every `@odata.nextLink` beneath this tenant/group's invalidatable
/// cache key. Optional names retain limited-information responses. Graph v1.0
/// [list owners](https://learn.microsoft.com/en-us/graph/api/group-list-owners?view=graph-rest-1.0)
/// can omit service principals, so pagination alone does not prove completeness.
#[derive(facet::Facet)]
pub struct EntraGroupOwnersListRequest<'a> {
    #[facet(skip_serializing)]
    pub cancellation: CancellationScope,

    pub group_id: EntraGroupId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupOwnersListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            cancellation: arbitrary::Arbitrary::arbitrary(u)?,
            group_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_group_owners<'a>(
    group_id: EntraGroupId,
    auth_context: &'a AzureTenantAuthContext,
    cancellation: &CancellationScope,
) -> EntraGroupOwnersListRequest<'a> {
    EntraGroupOwnersListRequest {
        cancellation: cancellation.clone(),
        group_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupOwnersListRequest<'_> {
    type Output = Vec<EntraDirectoryObject>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "v1.0",
            "GET",
            "tenants",
            self.auth_context.tenant_id.to_string().as_str(),
            "groups",
            self.group_id.to_string().as_str(),
            "owners",
            "directory_objects",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(tenant_id = %self.auth_context.tenant_id, group_id = %self.group_id, "Fetching group owners");
        let query = MicrosoftGraphHelper::new(
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/owners",
                self.group_id
            ),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
            &self.cancellation,
        );
        let owners = query
            .fetch_all::<EntraDirectoryObject>(&self.cancellation)
            .await?;
        debug!("Found {} owners for group {}", owners.len(), self.group_id);
        Ok(owners)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupOwnersListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_groups;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_command::CancellationToken;
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_owners() -> eyre::Result<()> {
        CancellationToken::new()
            .cancellable_scope(|cancellation| async move {
                let tenant_id = get_test_tenant_id(&cancellation).await?;
                let auth_context = AuthContext::explicit_azure_cli();
                let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
                let groups = fetch_all_groups(&auth_context, &cancellation).await?;
                assert!(!groups.is_empty());
                // there's a chance that some groups just don't have members lol
                // lets hope that we aren't unlucky many times in a row
                let tries = 10.min(groups.len());
                for group in groups.iter().take(tries) {
                    let owners = fetch_group_owners(group.id, &auth_context, &cancellation).await?;
                    if !owners.is_empty() {
                        return Ok(());
                    }
                }
                bail!("Failed to ensure group owner fetching worked after {tries} tries");
            })
            .await
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupOwnersListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupOwnersListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupOwnersListRequest<'static> => Vec<EntraDirectoryObject>, effects = [Read]);
