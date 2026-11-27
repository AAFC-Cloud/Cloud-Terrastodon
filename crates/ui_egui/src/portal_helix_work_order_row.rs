use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrder;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use facet::Facet;

/// Visible work-order columns. Each server can return additional form fields.
#[derive(Facet)]
pub(crate) struct PortalHelixWorkOrderRow {
    pub id: BmcHelixWorkOrderId,
    pub summary: Option<String>,
    pub status: Option<String>,
    pub target: PortalTabTarget,
}

impl PortalHelixWorkOrderRow {
    pub fn new(server: &BmcHelixServerName, work_order: &BmcHelixWorkOrder) -> Self {
        let field = |name: &str| {
            work_order
                .values
                .get(name)
                .and_then(|value| facet_json::from_str::<String>(value.as_ref()).ok())
        };
        Self {
            id: work_order.id.clone(),
            summary: field("Description").or_else(|| field("Summary")),
            status: field("Status"),
            target: PortalHelixServerView::WorkOrder(work_order.id.clone()).target(server),
        }
    }
}
