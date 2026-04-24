use super::CognitiveServicesAccountArgument;
use clap::Args;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use eyre::Result;
use eyre::bail;
use std::io::Write;

/// Arguments for showing a single Azure Cognitive Services account.
#[derive(Args, Debug, Clone)]
pub struct AzureCognitiveServicesShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[arg(long, default_value_t)]
    pub tenant: AzureTenantArgument<'static>,

    /// Cognitive Services account resource id, resource name, or wildcard pattern.
    pub account: CognitiveServicesAccountArgument<'static>,
}

impl AzureCognitiveServicesShowArgs {
    pub async fn invoke(self) -> Result<()> {
        let Self { tenant, account } = self;
        let tenant_id = tenant.resolve().await?;
        let accounts = fetch_all_cognitive_services_accounts(tenant_id).await?;
        let mut matches = accounts
            .into_iter()
            .filter(|item| account.matches(item))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => bail!("No Cognitive Services account found matching '{}'.", account),
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                serde_json::to_writer_pretty(&mut handle, &matches.remove(0))?;
                handle.write_all(b"\n")?;
                Ok(())
            }
            _ => {
                matches.sort_by_key(|account| account.id.expanded_form());
                let ids = matches
                    .iter()
                    .map(|account| account.id.expanded_form())
                    .collect::<Vec<_>>()
                    .join("\n  ");
                bail!(
                    "Multiple Cognitive Services accounts matched '{}'. Use a full resource id.\n  {}",
                    account,
                    ids
                )
            }
        }
    }
}
