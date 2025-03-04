use crate::management_groups::fetch_root_management_group;
use cloud_terrastodon_core_azure_types::prelude::PimEntraRoleSettings;
use cloud_terrastodon_core_azure_types::prelude::uuid::Uuid;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use eyre::Result;
use eyre::bail;
use serde::Deserialize;
use std::path::PathBuf;

pub async fn fetch_entra_pim_role_settings(
    role_definition_id: &Uuid,
) -> Result<PimEntraRoleSettings> {
    let tenant_id = fetch_root_management_group().await?.tenant_id;
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleSettings?{}",
        format_args!(
            "$select={}&$filter={}",
            "id,roleDefinitionId,userMemberSettings",
            format_args!("(roleDefinition/id eq '{}')", role_definition_id,),
        )
    );

    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_dir(PathBuf::from_iter([
        "az",
        "rest",
        "GET",
        "pim_roleSettings",
        role_definition_id.to_string().replace(" ","_").as_ref(),
    ]));

    #[derive(Deserialize)]
    struct Response {
        value: Vec<PimEntraRoleSettings>,
    }

    let mut result: Result<Response, _> = cmd.run().await;
    if result.is_err() {
        // single retry - sometimes this returns a gateway error
        result = cmd.run().await;
    }
    let mut resp = result?;

    if resp.value.len() != 1 {
        bail!("Expected a single result, got {}", resp.value.len());
    }
    Ok(resp.value.pop().unwrap())
}

#[cfg(test)]
mod tests {
    use crate::pim_entra_role_assignments::fetch_my_entra_pim_role_assignments;

    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let role_assignments = fetch_my_entra_pim_role_assignments().await?;
        println!("Found {} role assignments", role_assignments.len());
        for role_assignment in role_assignments {
            let role_setting =
                fetch_entra_pim_role_settings(role_assignment.role_definition_id()).await?;
            println!("- {:?}", role_setting);
            assert!(role_setting.get_maximum_grant_period()?.as_secs() % (60 * 30) == 0);
        }
        Ok(())
    }
}
