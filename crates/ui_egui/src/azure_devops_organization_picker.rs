use crate::app::MyApp;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::ComboBox;
use eframe::egui::Id;
use eframe::egui::Key;
use eframe::egui::TextEdit;
use eframe::egui::Ui;

/// Explicit project-browsing scope and credentials for this window.
#[derive(Default)]
pub(crate) struct AzureDevOpsOrganizationPicker {
    input: String,
    entered_organizations: Vec<AzureDevOpsOrganizationUrl>,
    personal_access_account: Option<CloudTerrastodonAccountId>,
    error: Option<String>,
    pub generation: u64,
}

impl AzureDevOpsOrganizationPicker {
    pub(crate) fn known_organizations<'a>(
        &'a self,
        current: Option<&'a AzureDevOpsOrganizationUrl>,
        catalogue: Option<&'a CloudTerrastodonAccountCatalogue>,
    ) -> Vec<&'a AzureDevOpsOrganizationUrl> {
        let mut organizations = self.entered_organizations.iter().collect::<Vec<_>>();
        for organization in catalogue
            .into_iter()
            .flat_map(|catalogue| &catalogue.accounts)
            .filter_map(|account| match &account.namespace {
                CloudTerrastodonAccountNamespace::AzureDevOps { organization } => {
                    Some(organization)
                }
                _ => None,
            })
            .chain(current)
        {
            if !organizations.contains(&organization) {
                organizations.push(organization);
            }
        }
        organizations.sort_by_cached_key(|org| org.to_string());
        organizations
    }

    pub(crate) fn organization_input(&mut self, ui: &mut Ui) -> Option<AzureDevOpsOrganizationUrl> {
        let mut organization = None;
        ui.horizontal(|ui| {
            let response =
                ui.add(TextEdit::singleline(&mut self.input).hint_text("Organization name or URL"));
            let enter = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            if ui.button("Use organization").clicked() || enter {
                match self.input.trim().parse::<AzureDevOpsOrganizationUrl>() {
                    Ok(parsed) => {
                        organization = Some(parsed);
                        self.input.clear();
                        self.error = None;
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
        });
        if let Some(error) = &self.error {
            ui.label(error);
        }
        organization
    }

    fn select_organization(
        &mut self,
        current: &mut Option<AzureDevOpsOrganizationUrl>,
        organization: AzureDevOpsOrganizationUrl,
    ) -> bool {
        self.error = None;
        if !self.entered_organizations.contains(&organization) {
            self.entered_organizations.push(organization.clone());
        }
        if current.as_ref() == Some(&organization) {
            return false;
        }
        self.personal_access_account = None;
        if let Some(previous) = current.replace(organization)
            && !self.entered_organizations.contains(&previous)
        {
            self.entered_organizations.push(previous);
        }
        true
    }

    fn show(
        &mut self,
        ui: &mut Ui,
        current: &mut Option<AzureDevOpsOrganizationUrl>,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
    ) -> bool {
        let mut selected = None;
        ui.horizontal_wrapped(|ui| {
            let label = ui.label("Organization");
            ComboBox::from_id_salt("devops_organization")
                .selected_text(
                    current
                        .as_ref()
                        .map_or_else(|| "Choose…".to_owned(), ToString::to_string),
                )
                .show_ui(ui, |ui| {
                    for organization in self.known_organizations(current.as_ref(), catalogue) {
                        if ui
                            .selectable_label(
                                current.as_ref() == Some(organization),
                                organization.to_string(),
                            )
                            .clicked()
                        {
                            selected = Some(organization.clone());
                        }
                    }
                })
                .response
                .labelled_by(label.id);
        });
        let changed_org = self
            .organization_input(ui)
            .or(selected)
            .is_some_and(|organization| self.select_organization(current, organization));

        let Some(organization) = current.as_ref() else {
            return changed_org;
        };
        let previous_account = self.personal_access_account;
        let label = self.personal_access_account.map_or_else(
            || "Azure account".to_owned(),
            |id| {
                catalogue
                    .and_then(|catalogue| catalogue.by_id(id).ok())
                    .map_or_else(
                        || "Saved PAT account".to_owned(),
                        |account| format!("{} (PAT)", account.alias),
                    )
            },
        );
        ui.horizontal_wrapped(|ui| {
            let label_response = ui.label("Authentication");
            ComboBox::from_id_salt("devops_organization_authentication")
                .selected_text(label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.personal_access_account, None, "Azure account");
                    for account in catalogue
                        .into_iter()
                        .flat_map(|catalogue| &catalogue.accounts)
                        .filter(|account| Self::has_active_pat(account, organization))
                    {
                        ui.selectable_value(
                            &mut self.personal_access_account,
                            Some(account.id),
                            format!("{} (PAT)", account.alias),
                        );
                    }
                })
                .response
                .labelled_by(label_response.id);
        });
        changed_org || previous_account != self.personal_access_account
    }

    fn has_active_pat(
        account: &CloudTerrastodonAccountProfile,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> bool {
        matches!(&account.namespace, CloudTerrastodonAccountNamespace::AzureDevOps {
            organization: scope,
        } if scope == organization)
            && account
                .registration(CloudTerrastodonCredentialSource::PersonalAccessToken)
                .is_ok_and(|registration| {
                    matches!(
                        registration.session,
                        CloudTerrastodonAccountSession::Active { .. }
                    )
                })
    }
}

impl MyApp {
    pub(crate) fn show_azure_devops_organization_picker(&mut self, ui: &mut Ui) {
        if self.azure_devops_organization_picker.show(
            ui,
            &mut self.azure_devops_organization,
            self.account_picker.catalogue.as_ref().ok(),
        ) {
            self.clear_azure_devops_projects();
        }
    }

    pub(crate) fn select_azure_devops_organization(
        &mut self,
        organization: AzureDevOpsOrganizationUrl,
    ) {
        if self
            .azure_devops_organization_picker
            .select_organization(&mut self.azure_devops_organization, organization)
        {
            self.clear_azure_devops_projects();
        }
    }

    fn clear_azure_devops_projects(&mut self) {
        if let Some(projects) = self.azure_devops_projects.as_loaded() {
            for project in projects.iter() {
                self.checkboxes.remove(&Id::new(project.id));
            }
        }
        self.azure_devops_projects = Loadable::NotLoaded;
        self.azure_devops_organization_picker.generation += 1;
    }

    pub(crate) fn ready_azure_devops_authentication(&self) -> Option<AzureDevOpsAuthSelection> {
        if self.account_operation.is_some() {
            return None;
        }
        let organization = self.azure_devops_organization.as_ref()?;
        match self
            .azure_devops_organization_picker
            .personal_access_account
        {
            None => self
                .ready_azure_authentication()
                .map(AzureDevOpsAuthSelection::Bearer),
            Some(account_id) => {
                let account = self
                    .account_picker
                    .catalogue
                    .as_ref()
                    .ok()?
                    .by_id(account_id)
                    .ok()?;
                AzureDevOpsOrganizationPicker::has_active_pat(account, organization)
                    .then_some(AzureDevOpsAuthSelection::PersonalAccessToken { account_id })
            }
        }
    }

    pub(crate) fn show_azure_devops_authentication_required(&mut self, ui: &mut Ui) -> bool {
        if self.ready_azure_devops_authentication().is_some() {
            return false;
        }
        if self
            .azure_devops_organization_picker
            .personal_access_account
            .is_none()
        {
            return self.show_azure_authentication_required(ui);
        }
        if self.account_operation.is_some() {
            ui.spinner();
        } else if ui.button("Manage PAT account").clicked() {
            self.account_manager.open();
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
    use eyre::Result;

    #[test]
    fn organizations_merge_registered_and_entered_scopes() -> Result<()> {
        let registered: AzureDevOpsOrganizationUrl = "registered-org".parse()?;
        let entered: AzureDevOpsOrganizationUrl = "entered-org".parse()?;
        let mut picker = AzureDevOpsOrganizationPicker::default();
        let mut current = Some(registered.clone());
        assert!(picker.select_organization(&mut current, entered.clone()));
        assert!(!picker.select_organization(&mut current, entered));

        let catalogue = CloudTerrastodonAccountCatalogue {
            accounts: vec![CloudTerrastodonAccountProfile::new(
                "00000000-0000-0000-0000-000000000001".parse()?,
                "test-account".parse()?,
                CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                    organization: registered,
                },
            )],
            ..Default::default()
        };
        let names: Vec<_> = picker
            .known_organizations(current.as_ref(), Some(&catalogue))
            .into_iter()
            .map(AzureDevOpsOrganizationUrl::name)
            .collect();
        assert_eq!(names, ["entered-org", "registered-org"]);
        Ok(())
    }

    #[test]
    fn pat_selection_requires_an_active_session_for_the_selected_organization() -> Result<()> {
        let organization: AzureDevOpsOrganizationUrl = "test-organization".parse()?;
        let other: AzureDevOpsOrganizationUrl = "other-organization".parse()?;
        let mut account = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000001".parse()?,
            "test-account".parse()?,
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                organization: organization.clone(),
            },
        );
        assert!(!AzureDevOpsOrganizationPicker::has_active_pat(
            &account,
            &organization
        ));
        account
            .registration_mut(CloudTerrastodonCredentialSource::PersonalAccessToken)?
            .session = CloudTerrastodonAccountSession::Active {
            generation: "00000000-0000-0000-0000-000000000002".parse()?,
        };
        assert!(AzureDevOpsOrganizationPicker::has_active_pat(
            &account,
            &organization
        ));
        assert!(!AzureDevOpsOrganizationPicker::has_active_pat(
            &account, &other
        ));
        Ok(())
    }
}
