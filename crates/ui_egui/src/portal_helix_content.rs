use crate::app::MyApp;
use crate::portal_helix_server_form::PortalHelixServerForm;
use crate::portal_helix_server_view::PortalHelixServerView;
use crate::portal_helix_work::PortalHelixWork;
use crate::portal_helix_work_key::PortalHelixWorkKey;
use crate::portal_helix_work_order_lookup::PortalHelixWorkOrderLookup;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountArgument;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_bmc_helix::BmcHelixServerCatalog;
use cloud_terrastodon_bmc_helix::BmcHelixServerStore;
use cloud_terrastodon_bmc_helix::BmcHelixWorkOrderListRequest;
use cloud_terrastodon_bmc_helix_types::BmcHelixServer;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerName;
use cloud_terrastodon_bmc_helix_types::BmcHelixServerUrl;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use cloud_terrastodon_execution::ExecutableRequest;
use eframe::egui::Context;
use eframe::egui::Ui;
use eyre::Result;
use eyre::WrapErr;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

/// Tracked Helix servers and tab-local execution identity and results.
#[derive(Default)]
pub(crate) struct PortalHelixContent {
    pub servers: Loadable<BmcHelixServerCatalog>,
    pub server_form: Option<PortalHelixServerForm>,
    pub input_error: Option<DiagnosticReport>,
    catalogue_error: Option<DiagnosticReport>,
    account_revision: u64,
    selections: HashMap<(PortalTabId, BmcHelixServerUrl), CloudTerrastodonAccountId>,
    work: HashMap<PortalHelixWorkKey, Loadable<PortalHelixWork>>,
    work_order_lookups: HashMap<PortalHelixWorkKey, PortalHelixWorkOrderLookup>,
}

impl PortalHelixContent {
    pub fn show_catalogue_error(&self, ui: &mut Ui) {
        if let Some(error) = &self.catalogue_error {
            ui.collapsing("Helix server operation failed", |ui| {
                DiagnosticReportView::new(error).show(ui)
            });
        }
    }

    pub fn server_url(&self, name: &BmcHelixServerName) -> Option<&BmcHelixServerUrl> {
        self.servers.as_loaded()?.servers.get(name)
    }

    pub fn server(&self, name: &BmcHelixServerName) -> Option<BmcHelixServer> {
        self.servers.as_loaded()?.get(Some(name)).ok()
    }

    pub fn selected_account<'a>(
        &self,
        tab: PortalTabId,
        server: &BmcHelixServerUrl,
        accounts: &'a CloudTerrastodonAccountCatalogue,
    ) -> Option<&'a CloudTerrastodonAccountProfile> {
        let selected = self
            .selections
            .get(&(tab, server.clone()))
            .map(|&id| CloudTerrastodonAccountArgument::Id(id));
        accounts.resolve_bmc_helix(server, selected.as_ref()).ok()
    }

    pub fn select_account(
        &mut self,
        tab: PortalTabId,
        server: &BmcHelixServerUrl,
        account_id: CloudTerrastodonAccountId,
    ) {
        self.selections.insert((tab, server.clone()), account_id);
    }

    pub fn seed_tab(&mut self, tab: PortalTabId, source: PortalTabId) {
        let copied: Vec<_> = self
            .selections
            .iter()
            .filter(|((owner, _), _)| *owner == source)
            .map(|((_, server), &id)| (server.clone(), id))
            .collect();
        for (server, account) in copied {
            self.selections.entry((tab, server)).or_insert(account);
        }
    }

    fn seed_ticket_inventory(
        &mut self,
        tab: PortalTabId,
        source: PortalTabId,
        server: &BmcHelixServer,
        account: CloudTerrastodonAccountId,
    ) {
        let view = PortalHelixServerView::Tickets;
        let Loadable::Loaded {
            value: PortalHelixWork::Tickets(tickets),
            started_at,
            finished_at,
        } = self.work(source, server, account, &view)
        else {
            return;
        };
        let snapshot = Loadable::Loaded {
            value: PortalHelixWork::Tickets(Arc::clone(tickets)),
            started_at: *started_at,
            finished_at: *finished_at,
        };
        self.work
            .entry(PortalHelixWorkKey::new(tab, server, account, &view))
            .or_insert(snapshot);
    }

    pub fn retain_tabs(&mut self, is_retained: impl Fn(PortalTabId) -> bool) {
        self.selections.retain(|(tab, _), _| is_retained(*tab));
        self.work.retain(|key, _| is_retained(key.tab));
        self.work_order_lookups
            .retain(|key, _| is_retained(key.tab));
    }

    pub fn is_loading(&self) -> bool {
        matches!(self.servers, Loadable::Loading { .. })
            || self
                .work
                .values()
                .any(|work| matches!(work, Loadable::Loading { .. }))
            || self
                .work_order_lookups
                .values()
                .any(PortalHelixWorkOrderLookup::is_loading)
    }

    pub fn invalidate_accounts(&mut self) {
        self.account_revision = self.account_revision.wrapping_add(1);
        self.work.clear();
        self.work_order_lookups.clear();
    }

    pub fn work_order_lookup(
        &self,
        key: &PortalHelixWorkKey,
    ) -> Option<&PortalHelixWorkOrderLookup> {
        self.work_order_lookups.get(key)
    }

    pub fn work(
        &self,
        tab: PortalTabId,
        server: &BmcHelixServer,
        account: CloudTerrastodonAccountId,
        view: &PortalHelixServerView,
    ) -> &Loadable<PortalHelixWork> {
        self.work
            .get(&PortalHelixWorkKey::new(tab, server, account, view))
            .unwrap_or(&Loadable::NotLoaded)
    }
}

impl MyApp {
    pub(crate) fn seed_helix_content(&mut self, tab: PortalTabId, source: PortalTabId) {
        self.helix.seed_tab(tab, source);
        if let Some(PortalTabTarget::HelixServer { server, view }) =
            self.tabs.tab(tab).map(|tab| tab.target())
            && matches!(
                view,
                PortalHelixServerView::Tickets | PortalHelixServerView::Ticket(_)
            )
            && let Some(server) = self.helix.server(server)
            && let Some(account) = self.ready_helix_account(source, &server.url)
        {
            self.helix
                .seed_ticket_inventory(tab, source, &server, account);
        }
    }

    pub(crate) fn helix_ticket_display_id(
        &self,
        tab: PortalTabId,
        target: &PortalTabTarget,
    ) -> Option<&str> {
        let PortalTabTarget::HelixServer {
            server,
            view: view @ PortalHelixServerView::Ticket(id),
        } = target
        else {
            return None;
        };
        let server = self.helix.server(server)?;
        let account = self.ready_helix_account(tab, &server.url)?;
        let PortalHelixWork::Tickets(tickets) =
            self.helix.work(tab, &server, account, view).as_loaded()?
        else {
            return None;
        };
        tickets
            .iter()
            .find(|ticket| ticket.id == *id)?
            .display_id
            .as_deref()
            .filter(|value| !value.trim().is_empty())
    }

    pub(crate) fn ready_helix_account(
        &self,
        tab: PortalTabId,
        server: &BmcHelixServerUrl,
    ) -> Option<CloudTerrastodonAccountId> {
        if self.account_operation.is_some() {
            return None;
        }
        let accounts = self.account_picker.catalogue.as_ref().ok()?;
        let account = self.helix.selected_account(tab, server, accounts)?;
        matches!(
            accounts.bmc_helix_session(account.id, server),
            CloudTerrastodonAccountSession::Active { .. },
        )
        .then_some(account.id)
    }

    pub(crate) fn refresh_helix_catalogue(&mut self) {
        self.modify_helix_catalogue("Load Helix servers", |_| Ok(()));
    }

    pub(crate) fn modify_helix_catalogue(
        &mut self,
        description: &'static str,
        operation: impl FnOnce(&BmcHelixServerStore) -> Result<()> + Send + 'static,
    ) {
        if matches!(self.helix.servers, Loadable::Loading { .. }) {
            return;
        }
        self.helix.catalogue_error = None;
        let previous = match &self.helix.servers {
            Loadable::Loaded {
                value,
                started_at,
                finished_at,
            } => Some((value.clone(), *started_at, *finished_at)),
            _ => None,
        };
        let started_at = Instant::now();
        let cancellation = self.executor.cancellation().clone();
        let queued = LoadableWorkBuilder::<Self, _>::new()
            .description(description)
            .setter(move |app, outcome| {
                app.helix.servers = match (outcome, &previous) {
                    (Loadable::Failed { error, .. }, Some((previous, started_at, finished_at))) => {
                        app.helix.catalogue_error = Some(error);
                        Loadable::Loaded {
                            value: previous.clone(),
                            started_at: *started_at,
                            finished_at: *finished_at,
                        }
                    }
                    (outcome, _) => outcome,
                };
            })
            .work(async move |scope| {
                scope
                    .deferred(
                        "finish Helix server catalogue operation",
                        async move |_protected| {
                            tokio::task::spawn_blocking(move || {
                                let store = BmcHelixServerStore::configured();
                                operation(&store)?;
                                store.load()
                            })
                            .await?
                        },
                    )
                    .await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = queued {
            if self.helix.servers.as_loaded().is_some() {
                self.helix.catalogue_error = Some(error.into());
            } else {
                self.helix.servers = Loadable::Failed {
                    error: error.into(),
                    started_at,
                    finished_at: Instant::now(),
                };
            }
        }
    }

    pub(crate) fn load_helix_work(
        &mut self,
        server: &BmcHelixServer,
        account: CloudTerrastodonAccountId,
        view: &PortalHelixServerView,
        refresh: bool,
    ) {
        let tab = self.tabs.active_id();
        if matches!(
            self.helix.work(tab, server, account, view),
            Loadable::Loading { .. }
        ) {
            return;
        }
        let key = PortalHelixWorkKey::new(tab, server, account, view);
        let revision = self.helix.account_revision;
        let result_key = key.clone();
        let refresh_lookup = refresh && *view == PortalHelixServerView::WorkOrders;
        let work_server = server.clone();
        let work_view = view.clone();
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let started_at = Instant::now();
        let queued = LoadableWorkBuilder::<Self, _>::new()
            .description(match view {
                PortalHelixServerView::WorkOrder(id) => format!("Load Helix work order {id}"),
                PortalHelixServerView::Tickets | PortalHelixServerView::Ticket(_) => {
                    "Load Helix tickets".to_owned()
                }
                _ => "Load Helix work orders".to_owned(),
            })
            .setter(move |app, outcome| {
                if revision == app.helix.account_revision && app.tabs.retains_tab(tab) {
                    if refresh_lookup
                        && outcome.as_loaded().is_some()
                        && let Some(lookup) = app.helix.work_order_lookups.get_mut(&result_key)
                    {
                        lookup.refresh();
                    }
                    app.helix.work.insert(result_key.clone(), outcome);
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                PortalHelixWork::load(work_server, account, work_view, refresh, &executor).await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = queued {
            self.helix.work.insert(
                key,
                Loadable::Failed {
                    error: error.into(),
                    started_at,
                    finished_at: Instant::now(),
                },
            );
        }
    }

    pub(crate) fn search_helix_work_orders(
        &mut self,
        server: &BmcHelixServer,
        account: CloudTerrastodonAccountId,
        search: &str,
        ctx: &Context,
    ) {
        let tab = self.tabs.active_id();
        let key = PortalHelixWorkKey::new(tab, server, account, &PortalHelixServerView::WorkOrders);
        let started_at = Instant::now();
        let Some((generation, id)) = self
            .helix
            .work_order_lookups
            .entry(key.clone())
            .or_default()
            .search_due(search, started_at, ctx)
        else {
            return;
        };
        let request =
            BmcHelixWorkOrderListRequest::for_work_order_id(server.url.clone(), account, &id);
        let revision = self.helix.account_revision;
        let result_key = key.clone();
        let executor = self.executor.clone();
        let work_cache = self.work_cache.clone();
        let cancellation = executor.cancellation().clone();
        let queued = LoadableWorkBuilder::<Self, _>::new()
            .description(format!("Search Helix for {id}"))
            .setter(move |app, result| {
                if revision == app.helix.account_revision
                    && let Some(lookup) = app.helix.work_order_lookups.get_mut(&result_key)
                {
                    lookup.set_result(generation, result);
                }
            })
            .work(async move |scope| {
                let executor = work_cache.job_executor(&executor.with_cancellation_scope(scope))?;
                request
                    .execute(&executor)
                    .await
                    .wrap_err_with(|| format!("Searching Helix for work order {id}"))
            })
            .build()
            .enqueue_cancellable(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Some(lookup) = self.helix.work_order_lookups.get_mut(&key) {
            match queued {
                Ok(cancellation) => lookup.track(cancellation),
                Err(error) => lookup.set_result(
                    generation,
                    Loadable::Failed {
                        error: error.into(),
                        started_at,
                        finished_at: Instant::now(),
                    },
                ),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::CloudTerrastodonAccountDefault;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;

    #[test]
    fn tabs_can_use_different_accounts_for_the_same_helix_server() -> Result<()> {
        let server: BmcHelixServerUrl = "https://helix.example.invalid".parse()?;
        let first = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000001".parse()?,
            "first".parse()?,
            CloudTerrastodonAccountCredential::BmcHelix {
                server: None,
                username: "first".parse()?,
            },
        );
        let second = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000002".parse()?,
            "second".parse()?,
            CloudTerrastodonAccountCredential::BmcHelix {
                server: None,
                username: "second".parse()?,
            },
        );
        let first_id = first.id;
        let second_id = second.id;
        let mut catalogue = CloudTerrastodonAccountCatalogue {
            defaults: vec![CloudTerrastodonAccountDefault {
                namespace: CloudTerrastodonAccountNamespace::BmcHelix {
                    server: Some(server.clone()),
                },
                account_id: first.id,
            }],
            accounts: vec![first, second],
            ..Default::default()
        };
        let mut content = PortalHelixContent::default();
        let first_tab = PortalTabId(1);
        let second_tab = PortalTabId(2);
        content.select_account(second_tab, &server, second_id);
        assert_eq!(
            content
                .selected_account(first_tab, &server, &catalogue)
                .map(|account| account.id),
            Some(first_id),
        );
        assert_eq!(
            content
                .selected_account(second_tab, &server, &catalogue)
                .map(|account| account.id),
            Some(second_id),
        );
        catalogue.accounts.retain(|account| account.id != second_id);
        assert!(
            content
                .selected_account(second_tab, &server, &catalogue)
                .is_none()
        );
        Ok(())
    }
}
