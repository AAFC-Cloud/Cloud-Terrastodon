use clap::Args;
use cloud_terrastodon_azure::prelude::AzureTenantArgument;
use cloud_terrastodon_azure::prelude::list_tracked_tenant_aliases;
use cloud_terrastodon_azure::prelude::list_tracked_tenant_aliases_for;
use cloud_terrastodon_azure::prelude::tracked_tenant_aliases_file;
use cloud_terrastodon_azure::prelude::resolve_tracked_tenant_argument;
use cloud_terrastodon_azure::prelude::TrackedTenantAlias;
use eyre::Result;
use std::io::Write;

/// Arguments for listing aliases for tracked Azure tenants.
#[derive(Args, Debug, Clone)]
pub struct AzureTenantAliasListArgs {
    /// Optional tracked tenant id or Cloud Terrastodon alias to filter by.
    #[arg(long)]
    pub tenant: Option<AzureTenantArgument<'static>>,
}

impl AzureTenantAliasListArgs {
    pub async fn invoke(self) -> Result<()> {
        let mut aliases = if let Some(tenant) = self.tenant {
            let tenant_id = resolve_tracked_tenant_argument(tenant).await?;
            list_tracked_tenant_aliases_for(tenant_id).await?
        } else {
            list_tracked_tenant_aliases()
                .await?
                .into_iter()
                .flat_map(|(tenant, aliases)| {
                    let tenant_id = tenant.tenant_id;
                    let path = tracked_tenant_aliases_file(tenant_id);
                    aliases.into_iter().map(move |alias| TrackedTenantAlias {
                        tenant_id,
                        alias,
                        path: path.clone(),
                    })
                })
                .collect::<Vec<_>>()
        };

        aliases.sort_by(|left, right| {
            left.alias
                .cmp(&right.alias)
                .then(left.tenant_id.cmp(&right.tenant_id))
        });

        let stdout = std::io::stdout();
        let mut handle = stdout.lock();
        serde_json::to_writer_pretty(&mut handle, &aliases)?;
        handle.write_all(b"\n")?;
        Ok(())
    }
}