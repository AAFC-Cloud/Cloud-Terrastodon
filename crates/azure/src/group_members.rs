use crate::prelude::MicrosoftGraphBatchRequest;
use crate::prelude::MicrosoftGraphBatchRequestEntry;
use crate::prelude::MicrosoftGraphBatchResponseEntryBody;
use crate::prelude::MicrosoftGraphHelper;
use crate::prelude::MicrosoftGraphResponse;
use cloud_terrastodon_azure_types::prelude::GroupId;
use cloud_terrastodon_azure_types::prelude::Principal;
use cloud_terrastodon_command::CacheBehaviour;
use eyre::Context;
use eyre::Result;
use eyre::bail;
use http::Method;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

pub struct GetGroupMembersOperation {
    group_id: GroupId,
}
impl GetGroupMembersOperation {
    pub fn new(group_id: GroupId) -> Self {
        Self { group_id }
    }
    pub fn key(&self) -> PathBuf {
        PathBuf::from_iter(["group_members", self.group_id.to_string().as_ref()])
    }
    pub fn try_from_key(key: &Path) -> eyre::Result<Self> {
        let parts = key.iter().collect::<Vec<_>>();
        if parts.len() != 2 {
            eyre::bail!("Invalid part count for GetGroupMembersOperation");
        }
        if parts[0] != "group_members" {
            eyre::bail!("Invalid prefix for GetGroupMembersOperation");
        }
        let group_id = parts[1]
            .to_string_lossy()
            .parse()
            .wrap_err("Failed to parse group ID")?;
        Ok(Self::new(group_id))
    }
    pub fn url(&self) -> String {
        format!(
            "https://graph.microsoft.com/v1.0/groups/{}/members",
            self.group_id
        )
    }
    pub fn method(&self) -> Method {
        Method::GET
    }
}
impl From<GetGroupMembersOperation> for MicrosoftGraphBatchRequestEntry<Vec<Principal>> {
    fn from(op: GetGroupMembersOperation) -> Self {
        MicrosoftGraphBatchRequestEntry::new_get(op.key().to_string_lossy().to_string(), op.url())
    }
}

pub async fn fetch_group_members(group_id: GroupId) -> Result<Vec<Principal>> {
    debug!("Fetching members for group {}", group_id);
    let members = MicrosoftGraphHelper::new(
        format!("https://graph.microsoft.com/v1.0/groups/{group_id}/members"),
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "group_members",
                group_id.as_hyphenated().to_string().as_ref(),
            ]),
            valid_for: Duration::from_hours(8),
        },
    )
    .fetch_all::<Principal>()
    .await?;
    debug!("Found {} members for group {}", members.len(), group_id);
    Ok(members)
}

pub async fn fetch_group_members_batch(
    group_ids: impl IntoIterator<Item = GroupId>,
) -> Result<HashMap<GroupId, Vec<Principal>>> {
    let mut batch_request: MicrosoftGraphBatchRequest<Vec<Principal>> =
        MicrosoftGraphBatchRequest::new();
    batch_request.add_all(group_ids.into_iter().map(GetGroupMembersOperation::new));

    let batch_response = batch_request
        .send::<MicrosoftGraphResponse<Principal>>()
        .await?;
    let mut rtn = HashMap::default();
    for entry in batch_response.responses {
        let op = GetGroupMembersOperation::try_from_key(&PathBuf::from(&entry.id))
            .wrap_err_with(|| format!("Failed to parse operation from key {}", entry.id))?;
        match entry.body {
            MicrosoftGraphBatchResponseEntryBody::Success(members) => {
                rtn.insert(op.group_id, members.value);
            }
            MicrosoftGraphBatchResponseEntryBody::Error(
                microsoft_graph_batch_response_entry_error,
            ) => {
                bail!(
                    "Failed to fetch members for group {}: {:#?}",
                    op.group_id,
                    microsoft_graph_batch_response_entry_error
                );
            }
        }
    }
    Ok(rtn)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::groups::fetch_all_groups;
    use eyre::bail;

    #[tokio::test]
    async fn list_group_members() -> Result<()> {
        let groups = fetch_all_groups().await?;
        // there's a chance that some groups just don't have members lol
        // lets hope that we aren't unlucky many times in a row
        let tries = 10.min(groups.len());
        for group in groups.iter().take(tries) {
            println!("Checking group {} for members", group.id);
            let members = fetch_group_members(group.id).await?;
            if !members.is_empty() {
                println!("Found {} members for group {}", members.len(), group.id);
                return Ok(());
            }
        }
        bail!("Failed to ensure group member fetching worked after {tries} tries")
    }
}
