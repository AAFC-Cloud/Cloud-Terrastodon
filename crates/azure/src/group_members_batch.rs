use crate::prelude::GroupMembersListRequest;
use crate::prelude::MicrosoftGraphBatchRequest;
use crate::prelude::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use eyre::Context;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct GroupMembersListBatchRequest {
    pub group_ids: Vec<GroupId>,
}

pub fn fetch_group_members_batch(
    group_ids: impl IntoIterator<Item = GroupId>,
) -> GroupMembersListBatchRequest {
    GroupMembersListBatchRequest {
        group_ids: group_ids.into_iter().collect(),
    }
}

#[async_trait]
impl CacheableCommand for GroupMembersListBatchRequest {
    type Output = HashMap<GroupId, Vec<Principal>>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members_batch".to_string(),
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
        // Construct the request
        let mut batch_request: MicrosoftGraphBatchRequest<Vec<Principal>> =
            MicrosoftGraphBatchRequest::new();

        // Enable caching since it's a GET request
        batch_request.cache(self.cache_key());

        // Add the request for each group
        for group_id in &self.group_ids {
            batch_request.add(GroupMembersListRequest {
                group_id: *group_id,
            });
        }

        // Send the request
        let batch_response = batch_request
            .send::<MicrosoftGraphResponse<Principal>>()
            .await?;

        // Extract the members from the response
        let mut rtn = HashMap::new();
        for (group_id, response) in self
            .group_ids
            .into_iter()
            .zip(batch_response.responses.into_iter()) // Responses are ordered to match requests
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
