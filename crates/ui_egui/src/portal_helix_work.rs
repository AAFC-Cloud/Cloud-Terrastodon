use crate::portal_helix_server_view::PortalHelixServerView;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix::BmcHelixTicketListRequest;
use cloud_terrastodon_bmc_helix::BmcHelixWorkOrderGetRequest;
use cloud_terrastodon_bmc_helix::BmcHelixWorkOrderListRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicket;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrder;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eyre::Result;
use std::sync::Arc;

/// Helix inventories and details fetched using their corresponding SDK requests.
#[derive(Debug)]
pub(crate) enum PortalHelixWork {
    List(Arc<Vec<BmcHelixWorkOrder>>),
    Detail(Arc<BmcHelixWorkOrder>),
    Tickets(Arc<Vec<BmcHelixTicket>>),
}

impl PortalHelixWork {
    pub async fn load(
        server: BmcHelixServer,
        account_id: CloudTerrastodonAccountId,
        view: PortalHelixServerView,
        refresh: bool,
        executor: &CloudTerrastodonExecutor,
    ) -> Result<Self> {
        match view {
            PortalHelixServerView::WorkOrders => executor
                .execute(BmcHelixWorkOrderListRequest {
                    server: server.url,
                    account_id,
                    filter: None,
                    limit: BmcHelixWorkOrderListRequest::DEFAULT_LIMIT,
                })
                .with_invalidation(refresh)
                .await
                .map(Self::List),
            PortalHelixServerView::WorkOrder(work_order_id) => executor
                .execute(BmcHelixWorkOrderGetRequest {
                    server: server.url,
                    account_id,
                    work_order_id,
                })
                .with_invalidation(refresh)
                .await
                .map(Self::Detail),
            PortalHelixServerView::Tickets | PortalHelixServerView::Ticket(_) => executor
                .execute(BmcHelixTicketListRequest {
                    innovation_suite_url: server.innovation_suite_origin().clone(),
                    server: server.url,
                    account_id,
                    limit: BmcHelixTicketListRequest::DEFAULT_LIMIT,
                })
                .with_invalidation(refresh)
                .await
                .map(Self::Tickets),
            PortalHelixServerView::Overview => {
                eyre::bail!("Server overview has no ticket or work order request");
            }
        }
    }
}
