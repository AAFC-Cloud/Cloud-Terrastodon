use crate::azure_tenant_management_action::AzureTenantManagementAction;
use crate::azure_tenant_management_target::AzureTenantManagementTarget;
use crate::azure_tenant_management_update::AzureTenantManagementResult;
use crate::azure_tenant_management_update::AzureTenantManagementUpdate;
use crate::azure_tracked_tenant_catalogue::AzureTrackedTenantCatalogue;
use crate::azure_tracked_tenant_row::AzureTrackedTenantRow;
use crate::gui_work_cache::GuiWorkCache;
use crate::icons::page_icon;
use crate::inventory_table::show_inventory_result_count;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::MicrosoftGraphOrganization;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use eframe::egui::Button;
use eframe::egui::CollapsingHeader;
use eframe::egui::Color32;
use eframe::egui::Context;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::collections::HashMap;
use std::sync::Arc;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;
use tokio::runtime::Handle;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError;
use tracing::Instrument;

/// Manage the local tenant catalogue shared with `ct az tenant`.
#[derive(Default)]
pub struct AzureTenantManager {
    catalogue: Option<AzureTrackedTenantCatalogue>,
    pending: Option<(
        AzureTenantManagementTarget,
        oneshot::Receiver<AzureTenantManagementResult>,
    )>,
    error: Option<(AzureTenantManagementTarget, DiagnosticReport)>,
    tenant_input: String,
    details: HashMap<
        (AzureTenantId, Option<AzureBearerAuthenticationSelection>),
        Arc<MicrosoftGraphOrganization>,
    >,
}

impl AzureTenantManager {
    /// Clear directory results tied to the previous account or session.
    pub fn account_selection_changed(&mut self) {
        self.details.clear();
        if matches!(
            self.pending,
            Some((AzureTenantManagementTarget::Directory { .. }, _))
        ) {
            // Dropping the reply channel signals the tracked worker to cancel and drain.
            self.pending = None;
        }
        if matches!(
            self.error,
            Some((AzureTenantManagementTarget::Directory { .. }, _))
        ) {
            self.error = None;
        }
    }

    pub(crate) fn tracked_tenants(&self) -> &[(AzureTenantId, Vec<AzureTenantAlias>)] {
        self.catalogue
            .as_ref()
            .map(|catalogue| catalogue.tenants.as_slice())
            .unwrap_or_default()
    }

    pub(crate) fn tenant_name(&self, tenant: AzureTenantId) -> Option<&str> {
        self.tracked_tenants()
            .iter()
            .find(|(id, _)| *id == tenant)?
            .1
            .first()
            .map(|alias| alias.as_str())
    }

    fn directory_name(
        &self,
        tenant: AzureTenantId,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Option<&str> {
        self.details
            .get(&(tenant, authentication))
            .map(|details| details.display_name.as_str())
            .filter(|name| !name.trim().is_empty())
    }

    pub(crate) fn load_catalogue_if_needed(
        &mut self,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        ctx: &Context,
    ) {
        if self.catalogue.is_none() && self.error.is_none() {
            self.reload_catalogue(tracker, executor, work_cache, ctx);
        }
    }

    pub(crate) fn can_reload_catalogue(&self) -> bool {
        self.pending.is_none()
    }

    pub(crate) fn reload_catalogue(
        &mut self,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        ctx: &Context,
    ) {
        if self.can_reload_catalogue() {
            self.start(
                AzureTenantManagementAction::Reload,
                tracker,
                executor,
                work_cache,
                ctx,
            );
        }
    }

    pub(crate) fn show_list(
        &mut self,
        ui: &mut Ui,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
    ) -> Option<PortalTabOpen> {
        self.load_catalogue_if_needed(tracker, executor, work_cache, ui.ctx());
        self.show_status(ui, None);
        if let Some(action) = ui
            .add_enabled_ui(self.pending.is_none(), |ui| self.show_toolbar(ui))
            .inner
        {
            self.start(action, tracker, executor, work_cache, ui.ctx());
        }
        self.show_catalogue(ui)
    }

    pub(crate) fn show_tenant(
        &mut self,
        ui: &mut Ui,
        tenant: AzureTenantId,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) {
        self.load_catalogue_if_needed(tracker, executor, work_cache, ui.ctx());
        let action = ScrollArea::vertical()
            .id_salt(("tenant-management", tenant))
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(820.0));
                ui.horizontal(|ui| {
                    ui.label("Tenant ID");
                    ui.monospace(tenant.to_string());
                });
                self.show_status(ui, Some((tenant, authentication)));
                ui.add_enabled_ui(self.pending.is_none(), |ui| {
                    self.show_tenant_panel(ui, tenant, authentication)
                })
                .inner
            })
            .inner;
        if let Some(action) = action {
            self.start(action, tracker, executor, work_cache, ui.ctx());
        }
    }

    fn show_status(
        &self,
        ui: &mut Ui,
        directory: Option<(AzureTenantId, Option<AzureBearerAuthenticationSelection>)>,
    ) {
        if self.pending.is_some() {
            ui.horizontal(|ui| {
                ui.spinner();
                ui.label("Updating tenants…");
            });
        }
        if let Some((target, error)) = &self.error
            && match target {
                AzureTenantManagementTarget::Catalogue => true,
                AzureTenantManagementTarget::Tenant(id) => {
                    directory.is_some_and(|(tenant, _)| tenant == *id)
                }
                AzureTenantManagementTarget::Directory { tenant, selected } => {
                    directory == Some((*tenant, *selected))
                }
            }
        {
            ui.colored_label(Color32::DARK_RED, "Tenant operation failed");
            ui.collapsing("Error details", |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
    }

    fn show_tenant_panel(
        &self,
        ui: &mut Ui,
        tenant: AzureTenantId,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Option<AzureTenantManagementAction> {
        let catalogue = self.catalogue.as_ref()?;
        let Some((_, aliases)) = catalogue.tenants.iter().find(|(id, _)| *id == tenant) else {
            ui.label("This tenant is no longer tracked.");
            return None;
        };
        let is_default = catalogue
            .default
            .as_ref()
            .is_ok_and(|default| *default == Some(tenant));
        if is_default {
            ui.label("Default tenant");
        }
        if let Some(name) = self.directory_name(tenant, authentication) {
            ui.horizontal(|ui| {
                ui.label("Directory name");
                ui.label(name);
            });
        }
        self.show_tenant_actions(ui, tenant, is_default, authentication, aliases)
    }

    fn show_toolbar(&mut self, ui: &mut Ui) -> Option<AzureTenantManagementAction> {
        if ui.button("Discover from Azure CLI").clicked() {
            return Some(AzureTenantManagementAction::Discover);
        }
        ui.horizontal_wrapped(|ui| {
            ui.label("Tenant ID");
            ui.add(
                TextEdit::singleline(&mut self.tenant_input)
                    .desired_width(ui.available_width().min(310.0)),
            );
            let tenant = self.tenant_input.trim().parse::<AzureTenantId>().ok();
            if ui
                .add_enabled(tenant.is_some(), Button::new("Track tenant"))
                .on_disabled_hover_text("Enter the tenant's UUID")
                .clicked()
            {
                return tenant.map(AzureTenantManagementAction::Track);
            }
            None
        })
        .inner
    }

    fn show_catalogue(&self, ui: &mut Ui) -> Option<PortalTabOpen> {
        let catalogue = self.catalogue.as_ref()?;
        let default = match &catalogue.default {
            Ok(default) => *default,
            Err(error) => {
                ui.colored_label(
                    Color32::DARK_RED,
                    "Could not read the configured tenant default",
                );
                CollapsingHeader::new("Error details")
                    .id_salt("tenant-default-error")
                    .show(ui, |ui| DiagnosticReportView::new(error).show(ui));
                None
            }
        };
        if let Some(default) = default
            && !catalogue
                .tenants
                .iter()
                .any(|(tenant, _)| *tenant == default)
        {
            ui.colored_label(
                Color32::DARK_RED,
                "The configured default is no longer tracked.",
            );
        }
        let id = "tracked-tenants";
        if catalogue.tenants.is_empty() {
            show_inventory_result_count(ui, id, 0, 0);
            ui.label("No tracked tenants");
            return None;
        }
        let columns = [
            ReflectedTableColumn::new("Tenant ID", "tenant_id")
                .with_width(310.0)
                .with_icon(|_, _| Some(page_icon(PortalPage::AzureTenants)))
                .with_link("tenant_id"),
            ReflectedTableColumn::new("Aliases", "aliases"),
            ReflectedTableColumn::new("Default", "default").with_width(96.0),
        ];
        let rows: Vec<_> = catalogue
            .tenants
            .iter()
            .map(|(tenant, aliases)| {
                AzureTrackedTenantRow::new(*tenant, aliases, default == Some(*tenant))
            })
            .collect();
        ReflectedTable::new(id, &columns)
            .without_selection()
            .with_search_hint("Search tenants")
            .with_result_count()
            .show_explorable(ui, &rows, |row| Id::new(row.tenant_id), &mut HashMap::new())
    }

    fn show_tenant_actions(
        &self,
        ui: &mut Ui,
        tenant: AzureTenantId,
        is_default: bool,
        authentication: Option<AzureBearerAuthenticationSelection>,
        aliases: &[AzureTenantAlias],
    ) -> Option<AzureTenantManagementAction> {
        ui.separator();
        let confirmation = ui.make_persistent_id(("forget-tenant-confirmation", tenant));
        let action = ui
            .horizontal_wrapped(|ui| {
                if ui
                    .add_enabled(!is_default, Button::new("Set as default"))
                    .clicked()
                {
                    return Some(AzureTenantManagementAction::SetDefault(tenant));
                }
                if ui.button("Load directory details").clicked() {
                    return Some(AzureTenantManagementAction::Details {
                        tenant,
                        selected: authentication,
                    });
                }
                if ui.button("Forget tenant…").clicked() {
                    ui.data_mut(|data| data.insert_temp(confirmation, true));
                }
                None
            })
            .inner;
        if action.is_some() {
            return action;
        }
        if let Some(deleted_at) = self
            .details
            .get(&(tenant, authentication))
            .and_then(|details| details.directory_object.deleted_date_time)
        {
            ui.label(format!("Deleted at {deleted_at}"));
        }
        if ui.data(|data| data.get_temp::<bool>(confirmation).unwrap_or_default()) {
            ui.label(format!("Forget {tenant} and remove its local aliases?"));
            if is_default {
                ui.label("This tenant is your configured default.");
            }
            return ui
                .horizontal(|ui| {
                    if ui.button("Forget").clicked() {
                        ui.data_mut(|data| data.remove::<bool>(confirmation));
                        return Some(AzureTenantManagementAction::Forget(tenant));
                    }
                    if ui.button("Cancel").clicked() {
                        ui.data_mut(|data| data.remove::<bool>(confirmation));
                    }
                    None
                })
                .inner;
        }
        Self::show_aliases(ui, tenant, aliases)
    }

    fn show_aliases(
        ui: &mut Ui,
        tenant: AzureTenantId,
        aliases: &[AzureTenantAlias],
    ) -> Option<AzureTenantManagementAction> {
        ui.heading("Aliases");
        for alias in aliases {
            let remove = ui
                .push_id(alias.as_str(), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(alias.as_str());
                        ui.small_button("Remove").clicked()
                    })
                    .inner
                })
                .inner;
            if remove {
                return Some(AzureTenantManagementAction::RemoveAlias(
                    tenant,
                    alias.clone(),
                ));
            }
        }
        ui.horizontal(|ui| {
            let id = ui.make_persistent_id(("new-tenant-alias", tenant));
            let mut alias_input = ui.data(|data| data.get_temp::<String>(id).unwrap_or_default());
            if ui
                .add(TextEdit::singleline(&mut alias_input).hint_text("New alias"))
                .changed()
            {
                ui.data_mut(|data| data.insert_temp(id, alias_input.clone()));
            }
            let alias = alias_input.trim().parse::<AzureTenantAlias>().ok();
            if ui
                .add_enabled(
                    alias.as_ref().is_some_and(|alias| !aliases.contains(alias)),
                    Button::new("Add alias"),
                )
                .clicked()
            {
                return alias.map(|alias| AzureTenantManagementAction::AddAlias(tenant, alias));
            }
            None
        })
        .inner
    }

    fn start(
        &mut self,
        action: AzureTenantManagementAction,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        ctx: &Context,
    ) {
        let (mut send, receive) = oneshot::channel();
        let ctx = ctx.clone();
        let job_executor = executor.clone();
        let work_cache = work_cache.clone();
        let description = action.description();
        let target = action.target();
        let span = tracing::info_span!("tenant_operation", operation = description);
        let task = tracker.spawn(
            executor.cancellation(),
            &Handle::current(),
            description.to_owned(),
            async move |scope| {
                let executor = job_executor.with_cancellation_scope(scope);
                let (cancellation, operation) =
                    executor.cancellable_child_scope(async |executor| {
                        action.run(&executor, &work_cache).await
                    });
                let operation = operation.instrument(span);
                tokio::pin!(operation);
                let result = tokio::select! {
                    biased;
                    () = send.closed() => {
                        cancellation.request_cancel("tenant operation result no longer needed");
                        let _ = operation.await;
                        return Ok(());
                    }
                    result = &mut operation => result.map_err(DiagnosticReport::new),
                };
                if let Err(error) = &result {
                    tracing::error!(
                        error = error.as_error(),
                        operation = description,
                        "Tenant operation failed"
                    );
                }
                let _ = send.send(result);
                ctx.request_repaint();
                Ok(())
            },
        );
        match task {
            Ok(()) => {
                self.error = None;
                self.pending = Some((target, receive));
            }
            Err(error) => self.error = Some((target, DiagnosticReport::new(error))),
        }
    }

    /// Consume an outstanding job even when another portal tab is selected.
    pub(crate) fn poll(&mut self) {
        let Some((target, pending)) = &mut self.pending else {
            return;
        };
        match pending.try_recv() {
            Err(TryRecvError::Empty) => return,
            Err(TryRecvError::Closed) => {
                self.error = Some((
                    *target,
                    DiagnosticReport::new(eyre::eyre!("Tenant operation was interrupted")),
                ));
            }
            Ok(Err(error)) => self.error = Some((*target, error)),
            Ok(Ok(AzureTenantManagementUpdate::Catalogue(catalogue))) => {
                self.details
                    .retain(|(id, _), _| catalogue.tenants.iter().any(|(tenant, _)| tenant == id));
                self.catalogue = Some(catalogue);
            }
            Ok(Ok(AzureTenantManagementUpdate::Details(details))) => {
                if let AzureTenantManagementTarget::Directory { tenant, selected } = *target {
                    self.details.insert((tenant, selected), details);
                }
            }
        }
        self.pending = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_azure::MicrosoftGraphDirectoryObject;
    use cloud_terrastodon_azure::MicrosoftGraphEntity;

    #[test]
    fn switching_accounts_preserves_catalogue_work() -> eyre::Result<()> {
        let tenant = "00000000-0000-0000-0000-000000000001".parse()?;
        let (directory, receive) = oneshot::channel();
        let mut manager = AzureTenantManager {
            pending: Some((
                AzureTenantManagementTarget::Directory {
                    tenant,
                    selected: None,
                },
                receive,
            )),
            ..Default::default()
        };
        manager.account_selection_changed();
        assert!(directory.is_closed());

        let (catalogue, receive) = oneshot::channel();
        manager.pending = Some((AzureTenantManagementTarget::Catalogue, receive));
        manager.account_selection_changed();
        assert!(!catalogue.is_closed());

        Ok(())
    }

    #[test]
    fn directory_results_follow_the_requested_tenant_and_account() -> eyre::Result<()> {
        let tenant = "00000000-0000-0000-0000-000000000001".parse()?;
        let first = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000002".parse()?,
            tenant_id: tenant,
            auth_source: AzureBearerAuthSource::AzureCli,
        };
        let second = AzureBearerAuthenticationSelection {
            account_id: "00000000-0000-0000-0000-000000000003".parse()?,
            ..first
        };
        let mut manager = AzureTenantManager::default();
        for (selection, name) in [(first, "First directory"), (second, "Second directory")] {
            let (send, receive) = oneshot::channel();
            manager.pending = Some((
                AzureTenantManagementTarget::Directory {
                    tenant,
                    selected: Some(selection),
                },
                receive,
            ));
            let details = Arc::new(MicrosoftGraphOrganization {
                entity: MicrosoftGraphEntity { id: tenant },
                directory_object: MicrosoftGraphDirectoryObject {
                    deleted_date_time: None,
                },
                display_name: name.to_owned(),
            });
            assert!(
                send.send(Ok(AzureTenantManagementUpdate::Details(details)))
                    .is_ok()
            );
            manager.poll();
        }

        assert_eq!(
            manager.directory_name(tenant, Some(first)),
            Some("First directory")
        );
        assert_eq!(
            manager.directory_name(tenant, Some(second)),
            Some("Second directory")
        );
        assert_eq!(manager.directory_name(tenant, None), None);
        assert_eq!(manager.tenant_name(tenant), None);
        manager.account_selection_changed();
        assert_eq!(manager.directory_name(tenant, Some(first)), None);
        assert_eq!(manager.directory_name(tenant, Some(second)), None);
        Ok(())
    }
}
