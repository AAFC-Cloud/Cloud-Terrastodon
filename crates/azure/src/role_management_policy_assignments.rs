use cloud_terrastodon_azure_types::prelude::RoleDefinitionId;
use cloud_terrastodon_azure_types::prelude::RoleManagementPolicyAssignment;
use cloud_terrastodon_azure_types::prelude::Scope;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_hcl_types::prelude::Sanitizable;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

pub struct RoleManagementPolicyAssignmentsForRoleRequest<T: Scope + Send> {
    pub scope: T,
    pub role_definition_id: RoleDefinitionId,
}

pub fn fetch_role_management_policy_assignments<T: Scope + Send>(
    scope: T,
    role_definition_id: RoleDefinitionId,
) -> RoleManagementPolicyAssignmentsForRoleRequest<T> {
    RoleManagementPolicyAssignmentsForRoleRequest {
        scope,
        role_definition_id,
    }
}

#[async_trait]
impl<T: Scope + Send> cloud_terrastodon_command::CacheableCommand
    for RoleManagementPolicyAssignmentsForRoleRequest<T>
{
    type Output = Vec<RoleManagementPolicyAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            self.scope.expanded_form().sanitize().as_ref(),
            "roleManagementPolicyAssignments",
            "roleDefinitionId",
            self.role_definition_id.short_form().as_ref(),
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let url = format!(
            "https://management.azure.com/{}/providers/Microsoft.Authorization/roleManagementPolicyAssignments?api-version=2020-10-01&$filter=roleDefinitionId+eq+%27{}%27",
            self.scope.expanded_form(),
            self.role_definition_id.expanded_form()
        );

        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.cache(self.cache_key());
        cmd.args(["rest", "--method", "GET", "--url", &url]);

        #[derive(Deserialize)]
        struct Response {
            value: Vec<RoleManagementPolicyAssignment>,
        }

        let mut result: Result<Response, _> = cmd.run().await;
        if result.is_err() {
            // single retry - sometimes this returns a gateway error
            result = cmd.run().await;
        }
        Ok(result?.value)
    }
}

cloud_terrastodon_command::impl_cacheable_into_future!(RoleManagementPolicyAssignmentsForRoleRequest<T>, T: Scope + Send + 'static);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;
    use crate::role_eligibility_schedules::fetch_my_role_eligibility_schedules;
    use humantime::format_duration;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(mut my_roles) =
            expect_aad_premium_p2_license(fetch_my_role_eligibility_schedules().await).await?
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
