use cloud_terrastodon_azure_types::prelude::RoleDefinitionId;
use cloud_terrastodon_azure_types::prelude::RoleManagementPolicyAssignment;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_command::CacheBehaviour;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;
use std::time::Duration;

pub async fn fetch_role_management_policy_assignments(
    scope: impl Scope,
    role_definition_id: RoleDefinitionId,
) -> Result<Vec<RoleManagementPolicyAssignment>> {
    let url = format!(
        "https://management.azure.com/{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments?api-version=2020-10-01&$filter=roleDefinitionId+eq+%27{}%27",
        scope.expanded_form(),
        role_definition_id.expanded_form()
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_behaviour(CacheBehaviour::Some {
        path: PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "roleManagementPolicyAssignments",
            "roleDefinitionId",
            role_definition_id.short_form().as_ref(),
        ]),
        valid_for: Duration::from_hours(1),
    });

    #[derive(Deserialize)]
    struct Response {
        value: Vec<RoleManagementPolicyAssignment>,
    }

    let resp: Response = cmd.run().await?;
    Ok(resp.value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::role_eligibility_schedules::fetch_my_role_eligibility_schedules;
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;
    use humantime::format_duration;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(mut my_roles) = expect_aad_premium_p2_license(
            fetch_my_role_eligibility_schedules().await,
        )
        .await?
        else {
            return Ok(());
        };
        let role = my_roles.swap_remove(0);
        let scope = role.properties.scope;
        let role_definition_id = role.properties.role_definition_id;
        let found_policy_assignments =
            fetch_role_management_policy_assignments(scope, role_definition_id).await?;
        assert!(!found_policy_assignments.is_empty());
        for ass in found_policy_assignments {
            println!(
                "- {} up to {}",
                role.properties
                    .expanded_properties
                    .role_definition
                    .display_name,
                format_duration(ass.get_maximum_activation_duration().unwrap())
            );
        }
        Ok(())
    }
}
