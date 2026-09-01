use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Result;
use std::borrow::Cow;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct EntraGroupGetRequest<'a> {
    pub group_id: EntraGroupId,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupGetRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_id: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_group<'a>(
    group_id: EntraGroupId,
    auth_context: &'a AzureTenantAuthContext,
) -> EntraGroupGetRequest<'a> {
    EntraGroupGetRequest {
        group_id,
        auth_context: Cow::Borrowed(auth_context),
    }
}

impl EntraGroupGetRequest<'_> {
    fn url(&self) -> String {
        format!("https://graph.microsoft.com/v1.0/groups/{}", self.group_id)
    }
}

#[must_use = "This is a future request, you must .await it"]
#[derive(facet::Facet)]
pub struct GroupByIdRequest<'a> {
    pub group_ids: Vec<EntraGroupId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for GroupByIdRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

pub fn fetch_groups_by_id<'a>(
    group_ids: impl IntoIterator<Item = EntraGroupId>,
    auth_context: &'a AzureTenantAuthContext,
) -> GroupByIdRequest<'a> {
    GroupByIdRequest {
        group_ids: group_ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupGetRequest<'_> {
    type Output = EntraGroup;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.auth_context.tenant_id.to_string().as_str(),
            self.group_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.auth_context.tenant_id,
            group_id = %self.group_id,
            "Fetching group by object id"
        );
        MicrosoftGraphHelper::new(
            self.url(),
            Some(self.cache_key()),
            self.auth_context.as_ref(),
        )
        .fetch_one()
        .await
    }
}

#[async_trait]
impl CacheableCommand for GroupByIdRequest<'_> {
    type Output = Vec<EntraGroup>;

    fn cache_key(&self) -> CacheKey {
        let mut hasher = blake3::Hasher::new();
        for group_id in &self.group_ids {
            hasher.update(group_id.as_ref().as_bytes());
        }
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups_by_id",
            self.auth_context.tenant_id.to_string().as_str(),
            hasher.finalize().to_hex().to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        if self.group_ids.is_empty() {
            return Ok(Vec::new());
        }

        let cache_key = self.cache_key();
        let mut batch = MicrosoftGraphBatchRequest::<EntraGroup>::new(self.auth_context.as_ref());
        batch.cache(cache_key);
        for (index, group_id) in self.group_ids.iter().enumerate() {
            batch.add(MicrosoftGraphBatchRequestEntry::new_get(
                format!("group-{group_id}-{index}"),
                format!("https://graph.microsoft.com/v1.0/groups/{group_id}"),
            ));
        }

        batch
            .send::<EntraGroup>()
            .await?
            .responses
            .into_iter()
            .map(|response| response.into_body())
            .collect()
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupGetRequest<'a>, 'a);
cloud_terrastodon_command::impl_cacheable_into_future!(GroupByIdRequest<'a>, 'a);
cloud_terrastodon_registry::register_thing!(EntraGroupGetRequest<'static>);
cloud_terrastodon_registry::register_thing!(GroupByIdRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupGetRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(GroupByIdRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupGetRequest<'static> => EntraGroup);
cloud_terrastodon_registry::register_into_future!(GroupByIdRequest<'static> => Vec<EntraGroup>);
