use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleAssignment;
use cloud_terrastodon_command::{CacheKey, CacheableCommand};
use cloud_terrastodon_command::impl_cacheable_into_future;
use cloud_terrastodon_command::async_trait;
use std::path::PathBuf;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub struct UnifiedRoleAssignmentListRequest;

pub fn fetch_all_unified_role_assignments() -> UnifiedRoleAssignmentListRequest {
    UnifiedRoleAssignmentListRequest
}

#[async_trait]
impl CacheableCommand for UnifiedRoleAssignmentListRequest {
    type Output = Vec<UnifiedRoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "ms",
            "graph",
            "GET",
            "unified_role_assignments",
        ]))
    }

    async fn run(self) -> eyre::Result<Self::Output> {
        debug!("Fetching all unified role assignments");
        let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleAssignments";
        let query = MicrosoftGraphHelper::new(url, Some(self.cache_key()));
        let rtn = query.fetch_all().await?;
        debug!("Fetched {} unified role assignments", rtn.len());
        Ok(rtn)
    }
}

impl_cacheable_into_future!(UnifiedRoleAssignmentListRequest);

#[cfg(test)]
mod test {
    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let assignments = super::fetch_all_unified_role_assignments().await?;
        println!("Assignments: {:#?}", assignments);
        println!("Count: {}", assignments.len());
        assert!(!assignments.is_empty());
        Ok(())
    }
}
