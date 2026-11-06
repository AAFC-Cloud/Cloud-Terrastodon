use super::CognitiveServicesAccountArgument;
use cloud_terrastodon_auth::AuthContext;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantArgumentExt;
use cloud_terrastodon_azure::Scope;
use cloud_terrastodon_azure::fetch_all_cognitive_services_accounts;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::io::Write;

/// Arguments for showing a single Azure Cognitive Services account.
#[derive(Facet, Debug, Clone)]
pub struct AzureCognitiveServicesShowArgs {
    /// Tracked tenant id or alias to query. Defaults to the active Azure CLI tenant.
    #[facet(figue::named, default)]
    pub tenant: AzureTenantArgument<'static>,

    /// Cognitive Services account resource id, resource name, or wildcard pattern.
    #[facet(figue::positional)]
    pub account: CognitiveServicesAccountArgument<'static>,
}

impl AzureCognitiveServicesShowArgs {
    pub async fn invoke(
        self,
        auth_context: &AuthContext,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<()> {
        let cancellation = executor.cancellation();
        let Self { tenant, account } = self;
        let tenant_auth_context = tenant.bind_auth_context(auth_context, cancellation).await?;
        let executor = &tenant_auth_context.auth_context.bind_executor(executor)?;
        let accounts =
            fetch_all_cognitive_services_accounts(&tenant_auth_context, executor).await?;
        let mut matches = accounts
            .into_iter()
            .filter(|item| account.matches(item))
            .collect::<Vec<_>>();

        match matches.len() {
            0 => {
                bail!(
                    "No Cognitive Services account found matching '{}'.",
                    account
                );
            }
            1 => {
                let stdout = std::io::stdout();
                let mut handle = stdout.lock();
                facet_json::to_writer_std_pretty(&mut handle, &matches.remove(0))
                    .map_err(|error| eyre::eyre!("{error:?}"))?;
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
                );
            }
        }
    }
}
