use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinition;
use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionId;
use cloud_terrastodon_command::CacheBehaviour;
use std::path::PathBuf;
use std::time::Duration;

/// Fetch an individual Entra role assignment
pub async fn fetch_unified_role_definition(
    role_definition_id: &UnifiedRoleDefinitionId,
) -> eyre::Result<UnifiedRoleDefinition> {
    let role_definition_id = role_definition_id.as_ref();
    let url = format!(
        "https://graph.microsoft.com/beta/roleManagement/directory/roleDefinitions/{role_definition_id}?$expand=inheritsPermissionsFrom"
    );
    let query = MicrosoftGraphHelper::new(
        url,
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "ms",
                "graph",
                "GET",
                "unified_role_definition",
                role_definition_id.to_string().as_ref(),
            ]),
            valid_for: Duration::from_hours(24),
        },
    );

    let found = query.fetch_one().await?;
    Ok(found)
}

/// Unravels the [`UnifiedRoleDefinition::inherits_permissions_from`] chain
/// into the top-level [`UnifiedRoleDefinition::role_permissions`]
pub async fn fetch_unified_role_definition_deep(
    role_definition_id: &UnifiedRoleDefinitionId,
) -> eyre::Result<UnifiedRoleDefinition> {
    let mut this = fetch_unified_role_definition(role_definition_id).await?;
    let mut next = std::mem::take(&mut this.inherits_permissions_from);
    while let Some(parent_id) = next.pop() {
        let parent = fetch_unified_role_definition(&parent_id.id).await?;
        this.inherits_permissions_from.push(parent_id);
        this.role_permissions.extend(parent.role_permissions);
        next.extend(parent.inherits_permissions_from);
    }

    Ok(this)
}

#[cfg(test)]
mod test {
    use crate::prelude::fetch_unified_role_definition;
    use crate::prelude::fetch_unified_role_definition_deep;
    use cloud_terrastodon_azure_types::prelude::UnifiedRoleDefinitionId;

    #[tokio::test]
    pub async fn it_works_single() -> eyre::Result<()> {
        let application_developer_role_id: UnifiedRoleDefinitionId =
            "cf1c38e5-3621-4004-a7cb-879624dced7c".parse()?;
        let directory_readers_role_id: UnifiedRoleDefinitionId =
            "88d8e3e3-8f55-4a1e-953a-9b9898b8876b".parse()?;
        let found = fetch_unified_role_definition(&application_developer_role_id).await?;
        println!("{:#?}", found);
        assert!(
            matches!(found.inherits_permissions_from.as_slice(), [x] if x.id == directory_readers_role_id)
        );
        Ok(())
    }

    #[tokio::test]
    pub async fn it_works_single_deep() -> eyre::Result<()> {
        let application_developer_role_id: UnifiedRoleDefinitionId =
            "cf1c38e5-3621-4004-a7cb-879624dced7c".parse()?;
        let found = fetch_unified_role_definition_deep(&application_developer_role_id).await?;
        println!("{:#?}", found);
        Ok(())
    }
}
