use cloud_terrastodon_azure_types::prelude::AzureTenantId;
use cloud_terrastodon_command::CommandBuilder;
use cloud_terrastodon_command::CommandKind;
use http::Method;

pub fn build_microsoft_graph_rest_command(
    method: Method,
    url: &str,
    tenant_id: Option<&AzureTenantId>,
) -> CommandBuilder {
    let mut cmd = CommandBuilder::new(CommandKind::CloudTerrastodon);
    cmd.args(["rest", "--method", method.as_str(), "--url", url]);
    if let Some(tenant_id) = tenant_id {
        let tenant_id = tenant_id.to_string();
        cmd.args(["--tenant", tenant_id.as_str()]);
    }
    cmd
}

pub fn build_microsoft_graph_rest_get_command(
    url: &str,
    tenant_id: Option<&AzureTenantId>,
) -> CommandBuilder {
    build_microsoft_graph_rest_command(Method::GET, url, tenant_id)
}
