use crate::resolve_bmc_helix_account::resolve_bmc_helix_account;
use cloud_terrastodon_app::CliOutput;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_bmc_helix::BmcHelixTicketListRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketRecordId;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_execution::ExecutableRequest;
use eyre::Result;
use eyre::bail;
use facet::Facet;
use std::num::NonZeroUsize;

/// Show an entry from the Helix Shared Ticket Console.
#[derive(Debug, Clone, Facet)]
pub struct BmcHelixTicketShowArgs {
    /// Record ID from the Shared Ticket Console.
    #[facet(figue::positional, figue::label = "RECORD_ID", proxy = String)]
    pub record_id: BmcHelixTicketRecordId,
    /// Tracked server name. Omit to use the configured default server.
    #[facet(figue::named)]
    pub server: Option<BmcHelixServerName>,
    /// Account ID or alias. Omit to use the server's configured default account.
    #[facet(figue::named)]
    pub account: Option<CloudTerrastodonAccountArgument>,
    /// Maximum number of Shared Ticket Console entries to search.
    #[facet(figue::named, default = BmcHelixTicketListRequest::DEFAULT_LIMIT)]
    pub limit: NonZeroUsize,
    /// Fetch fresh results.
    #[facet(figue::named, default)]
    pub no_cache: bool,
}

impl BmcHelixTicketShowArgs {
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
        let Some(ticket) = tickets.iter().find(|ticket| ticket.id == self.record_id) else {
            bail!(
                "Helix ticket {} did not appear in the Shared Ticket Console results",
                self.record_id
            );
        };
        Ok(CliOutput::facet(ticket.clone()))
    }
}
