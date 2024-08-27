use crate::prelude::fetch_current_user;
use anyhow::Result;
use cloud_terrasotodon_core_azure_types::prelude::PimEntraRoleAssignment;
use cloud_terrasotodon_core_command::prelude::CommandBuilder;
use cloud_terrasotodon_core_command::prelude::CommandKind;
use itertools::Itertools;
use serde::Deserialize;
use std::path::PathBuf;

pub async fn fetch_my_entra_pim_role_assignments() -> Result<Vec<PimEntraRoleAssignment>> {
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
    cmd.use_cache_dir(PathBuf::from(
        "az rest --method GET --url pim_roleAssignments",
    ));

    #[derive(Deserialize)]
    struct Response {
        value: Vec<PimEntraRoleAssignment>,
    }

    let mut result: Result<Response, _> = cmd.run().await;
    if result.is_err() {
        // single retry - sometimes this returns a gateway error
        result = cmd.run().await;
    }
    Ok(result?.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_my_entra_pim_role_assignments().await?;
        for ass in result {
            println!("- {:?}", ass)
        }
        Ok(())
    }
}
