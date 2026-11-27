use crate::azure_devops_organization_user_row::AzureDevOpsOrganizationUserRow;
use crate::azure_devops_organization_user_row::license_label;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseRuleAssignmentSource;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseRuleGitHubLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseRuleSource;
use cloud_terrastodon_azure_devops::AzureDevOpsLicenseType;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::AzureDevOpsUserId;
use cloud_terrastodon_azure_devops::AzureDevOpsUserLicenseEntitlement;
use eframe::egui::Button;
use eframe::egui::ComboBox;
use eframe::egui::Id;
use eframe::egui::Modal;
use eframe::egui::Ui;

/// A pending edit bound to the user and the license shown when editing began.
#[derive(Clone)]
pub(crate) struct AzureDevOpsUserLicenseDraft {
    pub user_id: AzureDevOpsUserId,
    pub expected_license: AzureDevOpsLicenseType,
    pub requested_license: Option<AzureDevOpsLicenseType>,
}

impl AzureDevOpsUserLicenseDraft {
    pub fn new(entitlement: &AzureDevOpsUserLicenseEntitlement) -> Self {
        Self {
            user_id: entitlement.id,
            expected_license: entitlement.license(),
            requested_license: None,
        }
    }

    /// Returns the requested license after Apply, or whether to dismiss the draft.
    pub fn show(
        &mut self,
        ui: &Ui,
        modal_id: Id,
        organization: &AzureDevOpsOrganizationUrl,
        entitlement: Option<&AzureDevOpsUserLicenseEntitlement>,
        demo_mode: bool,
        pending: bool,
    ) -> (Option<AzureDevOpsLicenseType>, bool) {
        let modal = Modal::new(modal_id).show(ui.ctx(), |ui| {
            ui.set_min_width(360.0);
            ui.heading("Change access level");
            ui.weak(organization.to_string());
            let Some(entitlement) = entitlement else {
                ui.label("This user is no longer listed. Reload the Users page to try again.");
                return (None, ui.button("Close").clicked());
            };
            let row = AzureDevOpsOrganizationUserRow::new(entitlement, demo_mode);
            ui.strong(row.name);
            ui.label(row.sign_in);
            ui.horizontal(|ui| {
                ui.label("Current access level");
                ui.strong(row.license);
            });
            let current = entitlement.license();
            if current != self.expected_license {
                ui.label(
                    "This user's access level changed. Reopen the editor to use the latest value.",
                );
                return (None, ui.button("Close").clicked());
            }

            if entitlement.access_level.assignment_source
                == AzureDevOpsLicenseRuleAssignmentSource::GroupRule
            {
                ui.label("A group rule may continue to grant a higher effective access level.");
            }
            if matches!(
                entitlement.access_level.licensing_source,
                AzureDevOpsLicenseRuleSource::Msdn
            ) || matches!(
                &entitlement.access_level.git_hub_license_type,
                Some(AzureDevOpsLicenseRuleGitHubLicenseType::Other(_))
            ) {
                ui.label("A subscription can take precedence over this direct assignment.");
            }

            ui.horizontal(|ui| {
                let label = ui.label("New access level");
                ComboBox::from_id_salt("license")
                    .selected_text(
                        self.requested_license
                            .as_ref()
                            .map_or("Choose access level".into(), license_label),
                    )
                    .show_ui(ui, |ui| {
                        for choice in AzureDevOpsLicenseType::VARIANTS
                            .iter()
                            .filter(|choice| **choice != current)
                        {
                            ui.selectable_value(
                                &mut self.requested_license,
                                Some(choice.clone()),
                                license_label(choice),
                            );
                        }
                    })
                    .response
                    .labelled_by(label.id);
            });
            ui.horizontal(|ui| {
                let apply = ui
                    .add_enabled(
                        !pending && self.requested_license.is_some(),
                        Button::new("Apply"),
                    )
                    .clicked();
                let cancel = ui.button("Cancel").clicked();
                (
                    apply.then(|| self.requested_license.take()).flatten(),
                    cancel,
                )
            })
            .inner
        });
        let close = modal.should_close();
        (modal.inner.0, close || modal.inner.1)
    }
}
