use crate::resolve_bmc_helix_account::resolve_bmc_helix_account;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_bmc_helix::BmcHelixWorkOrderGetRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixWorkOrderShowArgs {
    /// Work order ID.
    #[facet(figue::positional, figue::label = "ID")]
    pub work_order_id: BmcHelixWorkOrderId,
    /// Tracked server name. Omit to use the configured default server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
    /// Account ID or alias. Omit to use the server's configured default account.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Fetch fresh results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl BmcHelixWorkOrderShowArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let (server, account_id) =
            resolve_bmc_helix_account(self.server.as_ref(), self.account.as_ref())?;
        let work_order = BmcHelixWorkOrderGetRequest {
            server: server.url,
            account_id,
            work_order_id: self.work_order_id,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(work_order))
    }
}
