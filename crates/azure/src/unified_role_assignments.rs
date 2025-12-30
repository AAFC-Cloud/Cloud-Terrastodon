use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleAssignment;
use cloud_terrastodon_command::CacheKey;
use std::path::PathBuf;
use std::time::Duration;
use tracing::debug;

/// Fetches Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub async fn fetch_all_unified_role_assignments() -> eyre::Result<Vec<UnifiedRoleAssignment>> {
    debug!("Fetching all unified role assignments");
    let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleAssignments";
    let query = MicrosoftGraphHelper::new(
        url,
        Some(CacheKey {
            path: PathBuf::from_iter(["ms", "graph", "GET", "unified_role_assignments"]),
            valid_for: Duration::MAX,
        }),
    );
    let rtn = query.fetch_all().await?;
    debug!("Fetched {} unified role assignments", rtn.len());
    Ok(rtn)
}

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
