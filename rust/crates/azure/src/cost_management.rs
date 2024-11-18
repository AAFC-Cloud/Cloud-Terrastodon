use cloud_terrastodon_core_azure_types::cost_management::QueryDefinition;
use cloud_terrastodon_core_azure_types::cost_management::QueryResult;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;

use crate::prelude::fetch_root_management_group;

pub async fn fetch_cost_query_results(query: &QueryDefinition) -> anyhow::Result<QueryResult> {
    let root = fetch_root_management_group().await?;
    let url = format!(
        "https://management.azure.com/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.CostManagement/query?api-version=2021-10-01",
        root.tenant_id
    );
    let mut cmd = CommandBuilder::new(CommandKind::AzureCLI);
    cmd.args(["rest", "--method", "post", "--url", url.as_ref(), "--body"]);
    cmd.file_arg("body.json", serde_json::to_string_pretty(query)?);
    let resp = cmd.run::<QueryResult>().await?;
    Ok(resp)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn it_works1() -> anyhow::Result<()> {
        let query = QueryDefinition::new_cost_total_this_month();
        println!("{:#?}", query);
        let resp = fetch_cost_query_results(&query).await?;
        println!("{:#?}", resp);
        assert_eq!(resp.properties.next_link, None);
        Ok(())
    }
    #[tokio::test]
    async fn it_works2() -> anyhow::Result<()> {
        let query = QueryDefinition::new_cost_by_day_this_month();
        println!("{:#?}", query);
        let resp = fetch_cost_query_results(&query).await?;
        println!("{:#?}", resp);
        assert_eq!(resp.properties.next_link, None);
        Ok(())
    }
    #[tokio::test]
    async fn it_works3() -> anyhow::Result<()> {
        let query = QueryDefinition::new_cost_by_resource_group_this_month();
        println!("{:#?}", query);
        let resp = fetch_cost_query_results(&query).await?;
        println!("{:#?}", resp);
        assert_eq!(resp.properties.next_link, None);
        Ok(())
    }
}
