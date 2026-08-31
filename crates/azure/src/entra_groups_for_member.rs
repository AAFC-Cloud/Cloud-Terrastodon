use crate::fetch_groups_by_id;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::PrincipalId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AuthContext;
use cloud_terrastodon_rest::RestRequest;
use http::Method;
use std::borrow::Cow;
use std::collections::HashSet;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(Debug, Clone, facet::Facet)]
pub struct EntraGroupsForMemberRequest<'a> {
    pub tenant_id: AzureTenantId,
    pub principal_id: PrincipalId,
    pub auth_context: Cow<'a, AuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupsForMemberRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            tenant_id: arbitrary::Arbitrary::arbitrary(u)?,
            principal_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(AuthContext::default()),
        })
    }
}

pub fn fetch_entra_groups_for_member<'a>(
    tenant_id: AzureTenantId,
    principal_id: PrincipalId,
    auth_context: &'a AuthContext,
) -> EntraGroupsForMemberRequest<'a> {
    EntraGroupsForMemberRequest {
        tenant_id,
        principal_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[derive(facet::Facet)]
struct GetMemberGroupsResponse {
    value: Vec<EntraGroupId>,
}

#[async_trait]
impl CacheableCommand for EntraGroupsForMemberRequest<'_> {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "POST".to_string(),
            "entra_groups_for_member".to_string(),
            self.tenant_id.to_string(),
            self.principal_id.to_string(),
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!(
            tenant_id = %self.tenant_id,
            principal_id = %self.principal_id,
            "Fetching Entra groups for principal"
        );
        let response = RestRequest::new(
            Method::POST,
            format!(
                "https://graph.microsoft.com/v1.0/directoryObjects/{}/getMemberGroups",
                self.principal_id
            ),
        )?
        .tenant(self.tenant_id)
        .auth_context(self.auth_context.as_ref())
        .body("{\"securityEnabledOnly\":false}")
        .receive::<GetMemberGroupsResponse>()
        .await?;

        let group_ids: HashSet<_> = response.value.into_iter().collect();
        let groups =
            fetch_groups_by_id(self.tenant_id, group_ids, self.auth_context.as_ref()).await?;
        debug!(
            tenant_id = %self.tenant_id,
            principal_id = %self.principal_id,
            count = groups.len(),
            "Found Entra groups for principal"
        );
        Ok(groups)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupsForMemberRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(EntraGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupsForMemberRequest<'static>);
cloud_terrastodon_registry::register_into_future!(
    EntraGroupsForMemberRequest<'static> => Vec<EntraGroup>,
    effects = [Read]
);
