use crate::resolve_bmc_helix_account::resolve_bmc_helix_account;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_bmc_helix::BmcHelixTicketListRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use facet::Facet;
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Facet)]
pub struct BmcHelixTicketListArgs {
    /// Tracked server name. Omit to use the configured default server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
    /// Account ID or alias. Omit to use the server's configured default account.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Maximum number of tickets to return.
    #[facet(figue::named, default = BmcHelixTicketListRequest::DEFAULT_LIMIT)]
    pub limit: NonZeroUsize,
    /// Fetch fresh results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl BmcHelixTicketListArgs {
    pub async fn invoke(self, executor: &CloudTerrastodonExecutor) -> Result<CliOutput> {
        let (server, account_id) =
            resolve_bmc_helix_account(self.server.as_ref(), self.account.as_ref())?;
        let tickets = BmcHelixTicketListRequest {
            innovation_suite_url: server.innovation_suite_origin().clone(),
            server: server.url,
            account_id,
            limit: self.limit,
        }
        .execute(executor)
        .with_invalidation(self.no_cache)
        .await?;
        Ok(CliOutput::facet(tickets))
    }
}
