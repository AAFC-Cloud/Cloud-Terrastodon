use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicket;
use cloud_terrastodon_bmc_helix_types::BmcHelixTicketRecordId;
use facet::Facet;
use jiff::Timestamp;

/// Searchable ticket columns and the native inspection destination.
#[derive(Facet)]
pub(crate) struct PortalHelixTicketRow {
    pub id: BmcHelixTicketRecordId,
    pub display_id: String,
    pub title: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub created_at: Option<Timestamp>,
    pub last_activity_at: Option<Timestamp>,
    pub target: PortalTabTarget,
}

impl PortalHelixTicketRow {
    pub fn new(server: &BmcHelixServerName, ticket: &BmcHelixTicket) -> Self {
        Self {
            id: ticket.id.clone(),
            display_id: ticket
                .display_id
                .clone()
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| ticket.id.to_string()),
            title: ticket.title.clone(),
            status: ticket.status.clone(),
            assignee: ticket.assignee.clone(),
            created_at: ticket.created_at,
            last_activity_at: ticket.last_activity_at,
            target: PortalHelixServerView::Ticket(ticket.id.clone()).target(server),
        }
    }
}
