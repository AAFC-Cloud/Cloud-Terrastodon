use crate::app::MyApp;
use crate::azure_devops_organization_tracking_form::AzureDevOpsOrganizationTrackingForm;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganization;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganizationStore;
use cloud_terrastodon_command::CancellationScope;
use cloud_terrastodon_command::app_work::Loadable;
use cloud_terrastodon_command::app_work::LoadableWorkBuilder;
use eframe::egui::Button;
use eframe::egui::CollapsingHeader;
use eframe::egui::Color32;
use eframe::egui::RichText;
use eframe::egui::Ui;
use eyre::Result;
use std::future::Future;
use std::time::Instant;
use teamy_tracing::gui::DiagnosticReportView;

impl MyApp {
    pub(crate) fn reload_tracked_azure_devops_organizations(&mut self) {
        self.update_tracked_azure_devops_organizations(
            "Load tracked Azure DevOps organizations",
            async |_, _| Ok(()),
        );
    }

    fn update_tracked_azure_devops_organizations<F>(
        &mut self,
        description: &'static str,
        update: impl FnOnce(TrackedAzureDevOpsOrganizationStore, CancellationScope) -> F
        + Send
        + 'static,
    ) where
        F: Future<Output = Result<()>> + Send + 'static,
    {
        if self
            .azure_devops_organization_picker
            .is_loading_tracked_organizations()
        {
            return;
        }
        let cancellation = self.executor.cancellation().clone();
        let started_at = Instant::now();
        let enqueue_result = LoadableWorkBuilder::<Self, _>::new()
            .description(description)
            .setter(|app, data| app.azure_devops_organization_picker.tracked_organizations = data)
            .work(async move |scope| {
                update(TrackedAzureDevOpsOrganizationStore::configured(), scope).await?;
                TrackedAzureDevOpsOrganizationStore::configured()
                    .list()
                    .await
            })
            .build()
            .enqueue(
                &self.work_tracker.clone(),
                self.tx.clone(),
                self,
                &cancellation,
            );
        if let Err(error) = enqueue_result {
            self.azure_devops_organization_picker.tracked_organizations = Loadable::Failed {
                error: error.into(),
                started_at,
                finished_at: Instant::now(),
            };
        }
    }

    /// Start loading on demand and return whether the local catalogue is available.
    pub(crate) fn show_tracked_azure_devops_organizations_status(&mut self, ui: &mut Ui) -> bool {
        if matches!(
            self.azure_devops_organization_picker.tracked_organizations,
            Loadable::NotLoaded
        ) {
            self.reload_tracked_azure_devops_organizations();
        }
        match &self.azure_devops_organization_picker.tracked_organizations {
            Loadable::Loading { .. } => {
                ui.spinner();
            }
            Loadable::Failed { error, .. } => {
                ui.label("Organization catalogue unavailable.");
                CollapsingHeader::new("Error details")
                    .id_salt("tracked_azure_devops_organizations")
                    .show(ui, |ui| {
                        DiagnosticReportView::new(error).show(ui);
                    });
            }
            Loadable::Loaded { .. } => return true,
            Loadable::NotLoaded => {}
        }
        false
    }

    pub(crate) fn show_track_azure_devops_organization_button(&mut self, ui: &mut Ui) {
        self.show_organization_tracking_dialog(ui, "Track organization", None, None);
    }

    pub(crate) fn show_tracked_azure_devops_organization_actions(
        &mut self,
        ui: &mut Ui,
        organization: &AzureDevOpsOrganizationUrl,
    ) {
        let tracked = self
            .azure_devops_organization_picker
            .tracked_organization(organization);
        let tenant = tracked.and_then(|entry| entry.tenant_id);
        let is_tracked = tracked.is_some();
        ui.add_enabled_ui(
            !self
                .azure_devops_organization_picker
                .is_loading_tracked_organizations(),
            |ui| {
                ui.horizontal_wrapped(|ui| {
                    let label = if is_tracked {
                        "Edit tenant"
                    } else {
                        "Track organization"
                    };
                    self.show_organization_tracking_dialog(ui, label, Some(organization), tenant);
                    if is_tracked {
                        ui.menu_button("Forget organization", |ui| {
                            ui.label(format!(
                                "Remove {} from this device’s catalogue?",
                                organization.name()
                            ));
                            if ui.button("Forget").clicked() {
                                let organization = organization.clone();
                                self.update_tracked_azure_devops_organizations(
                                    "Forget Azure DevOps organization",
                                    async move |store, scope| {
                                        store.remove(&organization, &scope).await.map(drop)
                                    },
                                );
                                ui.close();
                            }
                        });
                    }
                });
            },
        );
    }

    fn show_organization_tracking_dialog(
        &mut self,
        ui: &mut Ui,
        label: &str,
        organization: Option<&AzureDevOpsOrganizationUrl>,
        tenant: Option<AzureTenantId>,
    ) {
        let id = ui.make_persistent_id(("devops-organization-registration", organization));
        let can_open = !self
            .azure_devops_organization_picker
            .is_loading_tracked_organizations();
        let button = if organization.is_none() {
            Button::new(RichText::new(label).color(Color32::WHITE))
                .fill(Color32::from_rgb(0, 120, 212))
        } else {
            Button::new(label)
        };
        if ui.add_enabled(can_open, button).clicked() {
            self.tenant_manager.reload_catalogue(
                &self.work_tracker,
                &self.executor,
                &self.work_cache,
                ui.ctx(),
            );
            ui.data_mut(|data| {
                data.insert_temp(id, AzureDevOpsOrganizationTrackingForm::new(tenant))
            });
        }
        let Some(mut form) =
            ui.data_mut(|data| data.remove_temp::<AzureDevOpsOrganizationTrackingForm>(id))
        else {
            return;
        };
        let (registration, dismissed) = form.show(
            ui,
            id,
            label,
            organization,
            self.tenant_manager.tracked_tenants(),
        );
        if let Some(registration) = registration {
            self.track_azure_devops_organization(registration);
        } else if !dismissed {
            ui.data_mut(|data| data.insert_temp(id, form));
        }
    }

    fn track_azure_devops_organization(&mut self, organization: TrackedAzureDevOpsOrganization) {
        self.update_tracked_azure_devops_organizations(
            "Save tracked Azure DevOps organization",
            async move |store, scope| store.add(organization, &scope).await.map(drop),
        );
    }
}
