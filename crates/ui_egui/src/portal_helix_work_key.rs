use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_tab_id::PortalTabId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;

/// Identifies a response within the requesting tab, server, and account.
#[derive(Clone, Eq, Hash, PartialEq)]
pub(crate) struct PortalHelixWorkKey {
    pub tab: PortalTabId,
    server: BmcHelixServerUrl,
    innovation_suite_url: Option<BmcHelixServerUrl>,
    account: CloudTerrastodonAccountId,
    view: PortalHelixServerView,
}

impl PortalHelixWorkKey {
    pub fn new(
        tab: PortalTabId,
        server: &BmcHelixServer,
        account: CloudTerrastodonAccountId,
        view: &PortalHelixServerView,
    ) -> Self {
        let view = match view {
            PortalHelixServerView::Ticket(_) => PortalHelixServerView::Tickets,
            view => view.clone(),
        };
        Self {
            tab,
            server: server.url.clone(),
            // Ticket data belongs to the configured API origin as well as the login origin.
            innovation_suite_url: (view == PortalHelixServerView::Tickets)
                .then(|| server.innovation_suite_origin().clone()),
            account,
            view,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_the_innovation_suite_origin_only_changes_the_ticket_inventory_key()
    -> eyre::Result<()> {
        let account = "00000000-0000-0000-0000-000000000001".parse()?;
        let tab = PortalTabId(1);
        let mut server = BmcHelixServer {
            name: "demo".parse()?,
            url: "https://ar.example.invalid".parse()?,
            innovation_suite_url: None,
        };
        let tickets =
            PortalHelixWorkKey::new(tab, &server, account, &PortalHelixServerView::Tickets);
        let work_orders =
            PortalHelixWorkKey::new(tab, &server, account, &PortalHelixServerView::WorkOrders);

        server.innovation_suite_url = Some("https://innovation.example.invalid".parse()?);

        assert!(
            tickets
                != PortalHelixWorkKey::new(tab, &server, account, &PortalHelixServerView::Tickets)
        );
        assert!(
            work_orders
                == PortalHelixWorkKey::new(
                    tab,
                    &server,
                    account,
                    &PortalHelixServerView::WorkOrders
                )
        );
        Ok(())
    }

    #[test]
    fn a_ticket_detail_reuses_its_console_inventory_for_the_selected_account() -> eyre::Result<()> {
        let server = BmcHelixServer {
            name: "demo".parse()?,
            url: "https://helix.example.invalid".parse()?,
            innovation_suite_url: None,
        };
        let tab = PortalTabId(1);
        let account = "00000000-0000-0000-0000-000000000001".parse()?;
        let other_account = "00000000-0000-0000-0000-000000000002".parse()?;
        let detail = PortalHelixServerView::Ticket("example-ticket".to_owned().try_into()?);
        let list_key =
            PortalHelixWorkKey::new(tab, &server, account, &PortalHelixServerView::Tickets);

        assert!(PortalHelixWorkKey::new(tab, &server, account, &detail) == list_key);
        assert!(PortalHelixWorkKey::new(tab, &server, other_account, &detail) != list_key);
        Ok(())
    }
}
