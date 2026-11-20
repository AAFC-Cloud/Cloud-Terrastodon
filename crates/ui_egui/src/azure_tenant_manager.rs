use crate::azure_tenant_management_action::AzureTenantManagementAction;
use crate::azure_tenant_management_target::AzureTenantManagementTarget;
use crate::azure_tenant_management_update::AzureTenantManagementResult;
use crate::azure_tenant_management_update::AzureTenantManagementUpdate;
use crate::azure_tracked_tenant_catalogue::AzureTrackedTenantCatalogue;
use crate::gui_work_cache::GuiWorkCache;
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
use eframe::egui::ScrollArea;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
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
    alias_input: String,
    selected: Option<AzureTenantId>,
    forget_confirmation: Option<AzureTenantId>,
    details: Option<Arc<MicrosoftGraphOrganization>>,
}

impl AzureTenantManager {
    pub(crate) fn close_view(&mut self) {
        self.forget_confirmation = None;
        self.account_selection_changed();
    }

    /// Clear directory results tied to the previous account or session.
    pub fn account_selection_changed(&mut self) {
        self.details = None;
        if matches!(
            self.pending,
            Some((AzureTenantManagementTarget::Directory, _))
        ) {
            // Dropping the reply channel signals the tracked worker to cancel and drain.
            self.pending = None;
        }
        if matches!(
            self.error,
            Some((AzureTenantManagementTarget::Directory, _))
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

    pub(crate) fn show(
        &mut self,
        ui: &mut Ui,
        tracker: &AppWorkTracker,
        executor: &CloudTerrastodonExecutor,
        work_cache: &GuiWorkCache,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) {
        self.load_catalogue_if_needed(tracker, executor, work_cache, ui.ctx());
        let action = ScrollArea::vertical()
            .id_salt("tenant-management")
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(820.0));
                if self.pending.is_some() {
                    ui.horizontal(|ui| {
                        ui.spinner();
                        ui.label("Updating tenants…");
                    });
                }
                if let Some((_, error)) = &self.error {
                    ui.colored_label(Color32::DARK_RED, "Tenant operation failed");
                    ui.collapsing("Error details", |ui| {
                        DiagnosticReportView::new(error).show(ui);
                    });
                }
                ui.add_enabled_ui(self.pending.is_none(), |ui| {
                    self.show_panel(ui, authentication)
                })
                .inner
            })
            .inner;
        if let Some(action) = action {
            self.start(action, tracker, executor, work_cache, ui.ctx());
        }
    }

    fn show_panel(
        &mut self,
        ui: &mut Ui,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Option<AzureTenantManagementAction> {
        if let Some(action) = self.show_toolbar(ui) {
            return Some(action);
        }
        self.show_catalogue(ui);
        let tenant = self.selected?;
        let is_default = self
            .catalogue
            .as_ref()?
            .default
            .as_ref()
            .is_ok_and(|default| *default == Some(tenant));
        if let Some(action) = self.show_tenant_actions(ui, tenant, is_default, authentication) {
            return Some(action);
        }
        if self.forget_confirmation.is_some() {
            // Leave aliases hidden while the confirmation is awaiting a decision.
            return None;
        }
        let catalogue = self.catalogue.as_ref()?;
        let (_, aliases) = catalogue.tenants.iter().find(|(id, _)| *id == tenant)?;
        Self::show_aliases(ui, tenant, aliases, &mut self.alias_input)
    }

    fn show_toolbar(&mut self, ui: &mut Ui) -> Option<AzureTenantManagementAction> {
        let action = ui
            .horizontal_wrapped(|ui| {
                if ui.button("Reload").clicked() {
                    return Some(AzureTenantManagementAction::Reload);
                }
                if ui.button("Discover from Azure CLI").clicked() {
                    return Some(AzureTenantManagementAction::Discover);
                }
                None
            })
            .inner;
        if action.is_some() {
            return action;
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

    fn show_catalogue(&mut self, ui: &mut Ui) {
        let Some(catalogue) = &self.catalogue else {
            return;
        };
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
        if catalogue.tenants.is_empty() {
            ui.label("No tracked tenants");
            return;
        }
        ScrollArea::vertical()
            .id_salt("tracked-tenants")
            .max_height(170.0)
            .show(ui, |ui| {
                for (tenant, aliases) in &catalogue.tenants {
                    let mut label = if let Some(alias) = aliases.first() {
                        format!("{alias} · {tenant}")
                    } else {
                        tenant.to_string()
                    };
                    if default == Some(*tenant) {
                        label.push_str(" · Default");
                    }
                    if ui
                        .selectable_label(self.selected == Some(*tenant), label)
                        .clicked()
                        && self.selected != Some(*tenant)
                    {
                        self.selected = Some(*tenant);
                        self.forget_confirmation = None;
                        self.alias_input.clear();
                        self.details = None;
                    }
                }
            });
    }

    fn show_tenant_actions(
        &mut self,
        ui: &mut Ui,
        tenant: AzureTenantId,
        is_default: bool,
        authentication: Option<AzureBearerAuthenticationSelection>,
    ) -> Option<AzureTenantManagementAction> {
        ui.separator();
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
                    self.forget_confirmation = Some(tenant);
                }
                None
            })
            .inner;
        if action.is_some() {
            return action;
        }
        if let Some(details) = &self.details
            && details.entity.id == tenant
        {
            ui.label(&details.display_name);
            ui.monospace(details.entity.id.to_string());
            if let Some(deleted_at) = details.directory_object.deleted_date_time {
                ui.label(format!("Deleted at {deleted_at}"));
            }
        }
        if let Some(confirmed) = self.forget_confirmation {
            ui.label(format!("Forget {confirmed} and remove its local aliases?"));
            if is_default {
                ui.label("This tenant is your configured default.");
            }
            return ui
                .horizontal(|ui| {
                    if ui.button("Forget").clicked() {
                        self.forget_confirmation = None;
                        return Some(AzureTenantManagementAction::Forget(confirmed));
                    }
                    if ui.button("Cancel").clicked() {
                        self.forget_confirmation = None;
                    }
                    None
                })
                .inner;
        }
        None
    }

    fn show_aliases(
        ui: &mut Ui,
        tenant: AzureTenantId,
        aliases: &[AzureTenantAlias],
        alias_input: &mut String,
    ) -> Option<AzureTenantManagementAction> {
        ui.heading("Aliases");
        let remove = ScrollArea::vertical()
            .id_salt("tracked-tenant-aliases")
            .max_height(150.0)
            .show(ui, |ui| {
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
                None
            })
            .inner;
        if remove.is_some() {
            return remove;
        }
        ui.horizontal(|ui| {
            ui.add(TextEdit::singleline(alias_input).hint_text("New alias"));
            let alias = alias_input.trim().parse::<AzureTenantAlias>().ok();
            if ui
                .add_enabled(alias.is_some(), Button::new("Add alias"))
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
                let contains = |id| catalogue.tenants.iter().any(|(tenant, _)| *tenant == id);
                self.selected = self
                    .selected
                    .filter(|id| contains(*id))
                    .or_else(|| {
                        catalogue
                            .default
                            .as_ref()
                            .ok()
                            .copied()
                            .flatten()
                            .filter(|id| contains(*id))
                    })
                    .or_else(|| catalogue.tenants.first().map(|(id, _)| *id));
                self.catalogue = Some(catalogue);
                self.details = None;
            }
            Ok(Ok(AzureTenantManagementUpdate::Details(details))) => self.details = Some(details),
        }
        self.pending = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switching_accounts_or_closing_tenants_preserves_catalogue_work() -> eyre::Result<()> {
        let (directory, receive) = oneshot::channel();
        let mut manager = AzureTenantManager {
            pending: Some((AzureTenantManagementTarget::Directory, receive)),
            ..Default::default()
        };
        manager.account_selection_changed();
        assert!(directory.is_closed());

        let (catalogue, receive) = oneshot::channel();
        manager.pending = Some((AzureTenantManagementTarget::Catalogue, receive));
        manager.account_selection_changed();
        assert!(!catalogue.is_closed());

        manager.forget_confirmation = Some("00000000-0000-0000-0000-000000000001".parse()?);
        manager.close_view();
        assert!(manager.forget_confirmation.is_none());
        assert!(!catalogue.is_closed());
        Ok(())
    }
}
