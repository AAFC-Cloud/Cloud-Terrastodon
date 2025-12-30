use crate::prelude::fetch_current_user;
use cloud_terrastodon_azure_types::prelude::GovernanceRoleAssignment;
use cloud_terrastodon_command::CacheKey;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use cloud_terrastodon_command::async_trait;
use cloud_terrastodon_command::impl_cacheable_into_future;
use eyre::Result;
use itertools::Itertools;
use serde::Deserialize;
use std::path::PathBuf;

pub struct MyEntraPimRoleAssignmentsListRequest;

pub fn fetch_my_entra_pim_role_assignments() -> MyEntraPimRoleAssignmentsListRequest {
    MyEntraPimRoleAssignmentsListRequest
}

#[async_trait]
impl cloud_terrastodon_command::CacheableCommand for MyEntraPimRoleAssignmentsListRequest {
    type Output = Vec<GovernanceRoleAssignment>;

    fn cache_key(&self) -> CacheKey {
        CacheKey::new(PathBuf::from_iter([
            "az",
            "rest",
            "GET",
            "pim_roleAssignments",
        ]))
    }

    async fn run(self) -> Result<Self::Output> {
        let my_object_id = fetch_current_user().await?.id;
        let url = format!(
            "{}{}{}{}",
            "https://graph.microsoft.com/beta/",
            "privilegedAccess/aadroles/roleAssignments",
            format_args!(
                "?$filter=(subject/id eq '{}') and (assignmentState in ('Eligible', 'Active'))",
                my_object_id
            ),
            format_args!(
                "&$select={}",
                [
                    "assignmentState",
                    "endDateTime",
                    "id",
                    "linkedEligibleRoleAssignmentId",
                    "memberType",
                    "roleDefinitionId",
                    "startDateTime",
                    "status",
                    "subjectId",
                ]
                .into_iter()
                .join(",")
            )
        );
        let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
        cmd.args(["rest", "--method", "GET", "--url", &url]);

        #[derive(Deserialize)]
        struct Response {
            value: Vec<GovernanceRoleAssignment>,
        }

        let mut result: Result<Response, _> = cmd.run().await;
        if result.is_err() {
            // single retry - sometimes this returns a gateway error
            result = cmd.run().await;
        }
        Ok(result?.value)
    }
}

impl_cacheable_into_future!(MyEntraPimRoleAssignmentsListRequest);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::test_helpers::expect_aad_premium_p2_license;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let Some(result) =
            expect_aad_premium_p2_license(fetch_my_entra_pim_role_assignments().await).await?
        else {
            return Ok(());
        };
        for ass in result {
            println!("- {:?}", ass)
        }
        Ok(())
    }
}
