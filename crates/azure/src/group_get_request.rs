use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphBatchRequestEntry;
use crate::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroup;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Result;
use std::path::PathBuf;
use tracing::debug;

#[must_use = "This is a future request, you must .await it"]
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct GroupRequest {
    pub tenant_id: AzureTenantId,
    pub group_id: EntraGroupId,
}

pub fn fetch_group(tenant_id: AzureTenantId, group_id: EntraGroupId) -> GroupRequest {
    GroupRequest {
        tenant_id,
        group_id,
    }
}

impl GroupRequest {
    fn url(&self) -> String {
        format!("https://graph.microsoft.com/v1.0/groups/{}", self.group_id)
    }
}

#[must_use = "This is a future request, you must .await it"]
#[derive(arbitrary::Arbitrary, facet::Facet)]
pub struct GroupByIdRequest {
    pub tenant_id: AzureTenantId,
    pub group_ids: Vec<EntraGroupId>,
}

pub fn fetch_groups_by_id(
    tenant_id: AzureTenantId,
    group_ids: impl IntoIterator<Item = EntraGroupId>,
) -> GroupByIdRequest {
    GroupByIdRequest {
        tenant_id,
        group_ids: group_ids.into_iter().collect(),
    }
}

#[async_trait]
impl CacheableCommand for GroupRequest {
    type Output = EntraGroup;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "groups",
            self.tenant_id.to_string().as_str(),
            self.group_id.to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        debug!(
            tenant_id = %self.tenant_id,
            group_id = %self.group_id,
            "Fetching group by object id"
        );
        MicrosoftGraphHelper::new(self.tenant_id, self.url(), Some(self.cache_key()))
            .fetch_one()
            .await
    }
}

#[async_trait]
impl CacheableCommand for GroupByIdRequest {
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
            self.tenant_id.to_string().as_str(),
            hasher.finalize().to_hex().to_string().as_str(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        if self.group_ids.is_empty() {
            return Ok(Vec::new());
        }

        let cache_key = self.cache_key();
        let mut batch = MicrosoftGraphBatchRequest::<EntraGroup>::new(self.tenant_id);
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

cloud_terrastodon_command::impl_cacheable_into_future!(GroupRequest);
cloud_terrastodon_command::impl_cacheable_into_future!(GroupByIdRequest);
cloud_terrastodon_registry::register_thing!(GroupRequest);
cloud_terrastodon_registry::register_thing!(GroupByIdRequest);
cloud_terrastodon_registry::register_arbitrary!(GroupRequest);
cloud_terrastodon_registry::register_arbitrary!(GroupByIdRequest);
cloud_terrastodon_registry::register_into_future!(GroupRequest => EntraGroup);
cloud_terrastodon_registry::register_into_future!(GroupByIdRequest => Vec<EntraGroup>);
