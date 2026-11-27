use crate::icons::HELIX_ICON;
use crate::icons::HELIX_TICKETS_ICON;
use crate::icons::HELIX_WORK_ORDER_ICON;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketRecordId;
use cloud_terrastodon_bmc_helix_types::BmcHelixWorkOrderId;
use eframe::egui::ImageSource;
use facet::Facet;

/// A resource section or entry within a saved Helix server.
#[derive(Clone, Debug, Eq, Facet, Hash, PartialEq)]
#[repr(C)]
pub enum PortalHelixServerView {
    Overview,
    WorkOrders,
    WorkOrder(BmcHelixWorkOrderId),
    Tickets,
    Ticket(BmcHelixTicketRecordId),
}

impl PortalHelixServerView {
    pub(crate) fn target(self, server: &BmcHelixServerName) -> PortalTabTarget {
        PortalTabTarget::HelixServer {
            server: server.clone(),
            view: self,
        }
    }

    pub(crate) fn icon(&self) -> ImageSource<'static> {
        match self {
            Self::Overview => HELIX_ICON,
            Self::WorkOrders | Self::WorkOrder(_) => HELIX_WORK_ORDER_ICON,
            Self::Tickets | Self::Ticket(_) => HELIX_TICKETS_ICON,
        }
    }
}
