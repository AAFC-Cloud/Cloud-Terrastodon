use crate::prelude::MicrosoftGraphHelper;
use cloud_terrastodon_azure_types::prelude::GovernanceRoleAssignment;
use cloud_terrastodon_azure_types::prelude::PrincipalId;
use cloud_terrastodon_command::CacheBehaviour;
use std::path::PathBuf;
use std::time::Duration;

/// See also: https://github.com/Azure/azure-cli/issues/28854
pub async fn fetch_governance_role_assignments_for_principal(
    principal_id: impl Into<PrincipalId>,
) -> eyre::Result<Vec<GovernanceRoleAssignment>> {
    let principal_id: PrincipalId = principal_id.into();
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/roleAssignments?$expand=linkedEligibleRoleAssignment,subject,roleDefinition($expand=resource)&$filter=(subject/id eq '{}')",
        principal_id
    );
    MicrosoftGraphHelper::new(
        url,
        CacheBehaviour::Some {
            path: PathBuf::from_iter([
                "governance_role_assignments",
                principal_id.to_string().as_str(),
            ]),
            valid_for: Duration::from_hours(8),
        },
    )
    .fetch_all()
    .await
}

#[cfg(test)]
mod test {
    use crate::auth::fetch_current_user;
    use crate::prelude::fetch_governance_role_assignments_for_principal;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let me = fetch_current_user().await?.id;
        let governance_role_assignments =
            fetch_governance_role_assignments_for_principal(&me).await?;
        for role in governance_role_assignments {
            println!("Role assignment: {role:#?}");
        }
        Ok(())
    }
}
