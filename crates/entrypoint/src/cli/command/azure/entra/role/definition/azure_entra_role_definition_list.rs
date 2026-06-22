use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::fetch_all_entra_role_definitions;
use eyre::Result;
use std::io::Write;
use tracing::info;

/// Arguments for listing Entra role definitions.
#[derive(Args, Debug, Clone)]
pub struct AzureEntraRoleDefinitionListArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,
}

impl AzureEntraRoleDefinitionListArgs {
    pub async fn invoke(self) -> Result<()> {
        let tenant_id = self.tenant.resolve().await?;
        info!(%tenant_id, "Fetching Entra role definitions");
        let mut role_definitions = fetch_all_entra_role_definitions(tenant_id)
            .await?
            .into_iter()
            .collect::<Vec<_>>();
        role_definitions.sort_unstable_by(|left, right| {
            left.display_name.cmp(&right.display_name).then_with(|| {
                left.template_id
                    .to_string()
                    .cmp(&right.template_id.to_string())
            })
        });

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        cloud_terrastodon_command::to_writer_pretty(&mut handle, &role_definitions)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}
