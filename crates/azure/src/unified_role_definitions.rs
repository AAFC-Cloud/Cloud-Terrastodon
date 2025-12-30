use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionCollection;
use cloud_terrastodon_command::CacheKey;
use std::path::PathBuf;
use std::time::Duration;

/// Fetch all Entra role assignments.
///
/// Not to be confused with Azure RBAC role assignments.
pub async fn fetch_all_unified_role_definitions() -> eyre::Result<UnifiedRoleDefinitionCollection> {
    let url = "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions"; // ?$top=500
    let query = MicrosoftGraphHelper::new(
        url,
        Some(CacheKey {
            path: PathBuf::from_iter(["ms", "graph", "GET", "unified_role_definitions"]),
            valid_for: Duration::MAX,
        }),
    );
    query
        .fetch_all()
        .await
        .map(UnifiedRoleDefinitionCollection::new)
}
#[cfg(test)]
mod test {
    use crate::prelude::fetch_all_unified_role_definitions;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let role_definitions = fetch_all_unified_role_definitions().await?;
        println!(
            "Sample role definition: {:#?}",
            role_definitions.values().next()
        );
        println!("Found {} role definitions", role_definitions.len());
        assert!(
            role_definitions
                .values()
                .all(|r| r.resource_scopes == vec!["/".to_string()])
        );
        Ok(())
    }
}
