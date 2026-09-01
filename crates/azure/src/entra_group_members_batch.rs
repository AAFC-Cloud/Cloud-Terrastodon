use crate::EntraGroupMembersListRequest;
use crate::MicrosoftGraphBatchRequest;
use crate::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::EntraGroupId;
use cloud_terrastodon_azure_types::Principal;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CacheableCommand;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_credentials::AzureTenantAuthContext;
use eyre::Context;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(facet::Facet)]
pub struct EntraGroupMembersListBatchRequest<'a> {
    pub group_ids: Vec<EntraGroupId>,
    pub auth_context: Cow<'a, AzureTenantAuthContext>,
}

impl<'a> arbitrary::Arbitrary<'a> for EntraGroupMembersListBatchRequest<'static> {
    fn arbitrary(u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self {
            group_ids: arbitrary::Arbitrary::arbitrary(u)?,
            auth_context: Cow::Owned(arbitrary::Arbitrary::arbitrary(u)?),
        })
    }
}

/// TODO! This does n't auto fetch nextLink stuff :(
#[deprecated = "This function does not handle pagination yet."]
pub fn fetch_group_members_batch<'a>(
    group_ids: impl IntoIterator<Item = EntraGroupId>,
    auth_context: &'a AzureTenantAuthContext,
) -> EntraGroupMembersListBatchRequest<'a> {
    EntraGroupMembersListBatchRequest {
        group_ids: group_ids.into_iter().collect(),
        auth_context: Cow::Borrowed(auth_context),
    }
}

#[async_trait]
impl CacheableCommand for EntraGroupMembersListBatchRequest<'_> {
    type Output = HashMap<EntraGroupId, Vec<Principal>>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms".to_string(),
            "graph".to_string(),
            "GET".to_string(),
            "group_members_batch".to_string(),
            self.auth_context.tenant_id.to_string(),
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
        let EntraGroupMembersListBatchRequest {
            group_ids,
            auth_context,
        } = self;
        // Construct the request
        let mut batch_request: MicrosoftGraphBatchRequest<'_, Vec<Principal>> =
            MicrosoftGraphBatchRequest::new(auth_context.as_ref());

        // Enable caching since it's a GET request
        batch_request.cache(cache_key);

        // Add the request for each group
        for group_id in &group_ids {
            batch_request.add(EntraGroupMembersListRequest {
                group_id: *group_id,
                auth_context: Cow::Borrowed(auth_context.as_ref()),
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

cloud_terrastodon_command::impl_cacheable_into_future!(EntraGroupMembersListBatchRequest<'a>, 'a);

cloud_terrastodon_registry::register_thing!(EntraGroupMembersListBatchRequest<'static>);
cloud_terrastodon_registry::register_arbitrary!(EntraGroupMembersListBatchRequest<'static>);
cloud_terrastodon_registry::register_into_future!(EntraGroupMembersListBatchRequest<'static> => HashMap<EntraGroupId, Vec<Principal>>);
