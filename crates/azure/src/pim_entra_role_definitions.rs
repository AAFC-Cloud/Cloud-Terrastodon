use crate::management_groups::fetch_root_management_group;
use cloud_terrastodon_azure_types::prelude::PimEntraRoleDefinition;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use eyre::Result;
use serde::Deserialize;
use std::path::PathBuf;

pub async fn fetch_all_entra_pim_role_definitions() -> Result<Vec<PimEntraRoleDefinition>> {
    let tenant_id = fetch_root_management_group().await?.tenant_id;
    let url = format!(
        "https://graph.microsoft.com/beta/privilegedAccess/aadroles/resources/{tenant_id}/roleDefinitions?$select=id,displayName,type,isbuiltIn&$orderby=displayName"
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "GET", "--url", &url]);
    cmd.use_cache_dir(PathBuf::from_iter([
        "az",
        "rest",
        "GET",
        "pim_roleDefinitions",
    ]));

    #[derive(Deserialize)]
    struct Response {
        value: Vec<PimEntraRoleDefinition>,
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
        let result = fetch_all_entra_pim_role_definitions().await?;
        for role_definition in result {
            println!("- {:?}", role_definition)
        }
        Ok(())
    }
}
