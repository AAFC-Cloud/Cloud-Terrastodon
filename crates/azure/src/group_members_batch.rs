use crate::GroupMembersListRequest;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::AzureTenantId;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Context;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct GroupMembersListBatchRequest {
    pub group_ids: Vec<EntraGroupId>,
    pub tenant_id: AzureTenantId,
}

/// TODO! This does n't auto fetch nextLink stuff :(
#[deprecated = "This function does not handle pagination yet."]
pub fn fetch_group_members_batch(
    tenant_id: AzureTenantId,
    group_ids: impl IntoIterator<Item = EntraGroupId>,
) -> GroupMembersListBatchRequest {
    GroupMembersListBatchRequest {
        group_ids: group_ids.into_iter().collect(),
        tenant_id,
    }
}

#[async_trait]
impl CacheableCommand for GroupMembersListBatchRequest {
    type Output = HashMap<EntraGroupId, Vec<Principal>>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members_batch".to_string(),
            self.tenant_id.to_string(),
            {
                let mut hasher = blake3::Hasher::new();
                for group_id in &self.group_ids {
                    hasher.update(group_id.as_bytes());
                }
                let hash = hasher.finalize();
                hash.to_hex().to_string()
            },
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        let cache_key = self.cache_key();
        let GroupMembersListBatchRequest {
            group_ids,
            tenant_id,
        } = self;
        // Construct the request
        let mut batch_request: MicrosoftGraphBatchRequest<Vec<Principal>> =
            MicrosoftGraphBatchRequest::new(tenant_id);

        // Enable caching since it's a GET request
        batch_request.cache(cache_key);

        // Add the request for each group
        for group_id in &group_ids {
            batch_request.add(GroupMembersListRequest {
                group_id: *group_id,
                tenant_id,
            });
        }

        // Send the request
        let batch_response = batch_request
            .send::<MicrosoftGraphResponse<Principal>>()
            .await?;

        // Extract the members from the response
        let mut rtn = HashMap::new();
        for (group_id, response) in group_ids.into_iter().zip(batch_response.responses)
        // Responses are ordered to match requests
        {
            let members = response
                .into_body()
                .wrap_err_with(|| format!("Failed to fetch members for group {group_id}"))?
                .value;
            rtn.insert(group_id, members);
        }

        // Return the results
        Ok(rtn)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(GroupMembersListBatchRequest);
