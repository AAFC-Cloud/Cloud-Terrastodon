use crate::prelude::fetch_root_management_group;
use cloud_terrastodon_azure_types::cost_management::QueryDefinition;
use cloud_terrastodon_azure_types::cost_management::QueryResult;
use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;

pub async fn fetch_cost_query_results(
    tenant_id: AzureTenantId,
    query: &QueryDefinition,
) -> eyre::Result<QueryResult> {
    let root = fetch_root_management_group(tenant_id).await?;
    let url = format!(
        "https://management.azure.com/providers/Microsoft.Management/managementGroups/{}/providers/Microsoft.CostManagement/query?api-version=2021-10-01",
        root.tenant_id
    );
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", "post", "--url", url.as_ref(), "--body"]);
    cmd.azure_file_arg("body.json", serde_json::to_string_pretty(query)?);
    let resp = cmd.run::<QueryResult>().await?;
    Ok(resp)
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::prelude::get_test_tenant_id;

    #[tokio::test]
    async fn it_works1() -> eyre::Result<()> {
        let query = QueryDefinition::new_cost_total_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works2() -> eyre::Result<()> {
        let query = QueryDefinition::new_cost_by_day_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
    #[tokio::test]
    async fn it_works3() -> eyre::Result<()> {
        let query = QueryDefinition::new_cost_by_resource_group_this_month();
        let resp = fetch_cost_query_results(get_test_tenant_id().await?, &query).await?;
        assert_eq!(resp.properties.next_link, None);
        assert!(!resp.properties.columns.is_empty());
        Ok(())
    }
}
