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
                "ms".to_string(),
                "graph".to_string(),
                "GET".to_string(),
                "governance_role_assignments".to_string(),
                principal_id.to_string(),
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
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    pub async fn it_works() -> eyre::Result<()> {
        let me = fetch_current_user().await?.id;
        let Some(governance_role_assignments) = expect_aad_premium_p2_license(
            fetch_governance_role_assignments_for_principal(&me).await,
        )
        .await?
        else {
            return Ok(());
        };

        for role in governance_role_assignments {
            println!("Role assignment: {role:#?}");
        }

        Ok(())
    }
}
