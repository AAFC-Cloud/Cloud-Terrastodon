use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[derive(facet::Facet)]
pub struct EntraGroupMembersListRequest<'a> {
    pub group_id: EntraGroupId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}
impl EntraGroupMembersListRequest<'_> {
    pub fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members",
            self.group_id
        )
    }
}
impl From<EntraGroupMembersListRequest<'_>> for MicrosoftGraphBatchRequestEntry<Vec<Principal>> {
    fn from(request: EntraGroupMembersListRequest<'_>) -> Self {
        MicrosoftGraphBatchRequestEntry::new_get(
            format!("group-members-for-{}", request.group_id),
            request.url(),
        )
    }
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupMembersListRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_group_members<'a>(
    group_id: EntraGroupId,
    auth_context: &'a AzureTenantAuthContext,
) -> EntraGroupMembersListRequest<'a> {
    EntraGroupMembersListRequest {
        group_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupMembersListRequest<'_> {
    type Output = Vec<Principal>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members".to_string(),
            self.auth_context.tenant_id.to_string(),
            self.group_id.as_hyphenated().to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(tenant_id = %self.auth_context.tenant_id, group_id = %self.group_id, "Fetching group members");
        let query = MicrosoftGraphHelper::new(
            format!(
                "https://graph.microsoft.com/v1.0/groups/{}/members",
                self.group_id
            ),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        );
        let members = query.fetch_all::<Principal>().await?;
        debug!(
            "Found {} members for group {}",
            members.len(),
            self.group_id
        );
        Ok(members)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupMembersListRequest<'a>, 'a);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fetch_all_groups;
    use crate::get_test_tenant_id;
    use cloud_terrastodon_credentials::AuthContext;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_members() -> eyre::Result<()> {
        let tenant_id = get_test_tenant_id().await?;
        let auth_context = AuthContext::explicit_azure_cli();
        let auth_context = auth_context.bind_to_azure_tenant(tenant_id)?;
        let groups = fetch_all_groups(&auth_context).await?;
        assert!(!groups.is_empty());
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            let members = fetch_group_members(group.id, &auth_context).await?;
            if !members.is_empty() {
                return Ok(());
            }
        }
        bail!("Failed to ensure group member fetching worked after {tries} tries")
    }
}

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupMembersListRequest<'static> => Vec<Principal>);
