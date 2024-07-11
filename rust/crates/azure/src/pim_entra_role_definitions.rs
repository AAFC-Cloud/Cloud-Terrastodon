use crate::management_groups::fetch_root_management_group;
use anyhow::Result;
use azure_types::prelude::PimEntraRoleDefinition;
use command::prelude::CommandBuilder;
use command::prelude::CommandKind;
use serde::Deserialize;
use std::path::PathBuf;

pub async fn fetch_all_entra_pim_role_definitions() -> Result<Vec<PimEntraRoleDefinition>> {
    let tenant_id = fetch_root_management_group().await?.tenant_id;
    let url = format!("https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleDefinitions?$select=id,displayName,type,isbuiltIn&$orderby=displayName");
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_dir(PathBuf::from(
        "az rest --method GET --url pim_roleDefinitions",
    ));

    #[derive(Deserialize)]
    struct Response {
        value: Vec<PimEntraRoleDefinition>,
    }

    let resp: Response = cmd.run().await?;
    Ok(resp.value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works() -> Result<()> {
        let result = fetch_all_entra_pim_role_definitions().await?;
        for role_definition in result {
            println!("- {:?}", role_definition)
        }
        Ok(())
    }
}
