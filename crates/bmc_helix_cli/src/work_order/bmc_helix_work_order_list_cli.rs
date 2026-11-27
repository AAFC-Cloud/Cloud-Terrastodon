use crate::resolve_bmc_helix_account::resolve_bmc_helix_account;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_bmc_helix::BmcHelixWorkOrderListRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixWorkOrderListArgs {
    /// Tracked server name. Omit to use the configured default server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
    /// Account ID or alias. Omit to use the server's configured default account.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// BMC qualification, for example: 'Status' = "Assigned".
    #[facet(figue::named)]
    pub filter: Option<String>,
    /// Maximum number of work orders to return.
    #[facet(figue::named, default = BmcHelixWorkOrderListRequest::DEFAULT_LIMIT)]
    pub limit: NonZeroUsize,
    /// Fetch fresh results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl BmcHelixWorkOrderListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let (server, account_id) =
            resolve_bmc_helix_account(self.server.as_ref(), self.account.as_ref())?;
        let work_orders = BmcHelixWorkOrderListRequest {
            server: server.url,
            account_id,
            filter: self.filter,
            limit: self.limit,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(work_orders))
    }
}
