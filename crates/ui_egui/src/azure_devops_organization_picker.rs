mod tracking;

use crate::app::MyApp;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_page::PortalPage;
use crate::portal_tab_id::PortalTabId;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::AuthSource;
use cloud_terrastodon_auth_types::AzureDevOpsAuthSelection;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountNamespace;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountSession;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_azure_devops::TrackedAzureDevOpsOrganization;
use cloud_terrastodon_command::app_work::Loadable;
use eframe::egui::ComboBox;
use eframe::egui::Key;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use std::collections::HashMap;

/// Shared catalogue of tracked and previously entered Azure DevOps organizations.
#[derive(Default)]
pub(crate) struct AzureDevOpsOrganizationPicker {
    entered_organizations: Vec<AzureDevOpsOrganizationUrl>,
    tracked_organizations: Loadable<Vec<TrackedAzureDevOpsOrganization>>,
}

impl AzureDevOpsOrganizationPicker {
    pub(crate) fn new(initial: Option<&AzureDevOpsOrganizationUrl>) -> Self {
        Self {
            entered_organizations: initial.into_iter().cloned().collect(),
            ..Default::default()
        }
    }

    pub(crate) fn is_loading_tracked_organizations(&self) -> bool {
        matches!(self.tracked_organizations, Loadable::Loading { .. })
    }

    pub(crate) fn tracked_tenant(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Option<AzureTenantId> {
        self.tracked_organization(organization)?.tenant_id
    }

    pub(crate) fn tracked_organization(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Option<&TrackedAzureDevOpsOrganization> {
        self.tracked_organizations
            .as_loaded()?
            .iter()
            .find(|tracked| &tracked.url == organization)
    }

    pub(crate) fn known_organizations<'a>(
        &'a self,
        current: Option<&'a AzureDevOpsOrganizationUrl>,
        catalogue: Option<&'a CloudTerrastodonAccountCatalogue>,
    ) -> Vec<(&'a AzureDevOpsOrganizationUrl, Option<AzureTenantId>)> {
        let mut organizations = self
            .tracked_organizations
            .as_loaded()
            .into_iter()
            .flatten()
            .map(|organization| (&organization.url, organization.tenant_id))
            .collect::<HashMap<_, _>>();
        for organization in catalogue
            .into_iter()
            .flat_map(|catalogue| &catalogue.accounts)
            .filter_map(|account| match &account.namespace {
                CloudTerrastodonAccountNamespace::AzureDevOps { organization } => {
                    Some(organization)
                }
                _ => None,
            })
            .chain(&self.entered_organizations)
            .chain(current)
        {
            organizations.entry(organization).or_insert(None);
        }
        let mut organizations = organizations.into_iter().collect::<Vec<_>>();
        organizations.sort_by_cached_key(|(org, _)| org.to_string());
        organizations
    }

    pub(crate) fn organization_input(&mut self, ui: &mut Ui) -> Option<AzureDevOpsOrganizationUrl> {
        let id = ui.make_persistent_id("azure-devops-organization-input");
        let (mut input, mut error) = ui.data_mut(|data| {
            data.remove_temp::<(String, Option<String>)>(id)
                .unwrap_or_default()
        });
        let mut organization = None;
        ui.horizontal(|ui| {
            let response =
                ui.add(TextEdit::singleline(&mut input).hint_text("Organization name or URL"));
            let enter = response.lost_focus() && ui.input(|input| input.key_pressed(Key::Enter));
            if ui.button("Use organization").clicked() || enter {
                match input.trim().parse::<AzureDevOpsOrganizationUrl>() {
                    Ok(parsed) => {
                        if !self.entered_organizations.contains(&parsed) {
                            self.entered_organizations.push(parsed.clone());
                        }
                        organization = Some(parsed);
                        input.clear();
                        error = None;
                    }
                    Err(report) => error = Some(report.to_string()),
                }
            }
        });
        if let Some(error) = &error {
            ui.label(error);
        }
        ui.data_mut(|data| data.insert_temp(id, (input, error)));
        organization
    }

    fn select_organization(
        current: &mut Option<AzureDevOpsOrganizationUrl>,
        organization: AzureDevOpsOrganizationUrl,
    ) -> bool {
        if current.as_ref() == Some(&organization) {
            return false;
        }
        *current = Some(organization);
        true
    }

    fn show_organization_scope(
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
                    for (organization, _) in self.known_organizations(current.as_ref(), catalogue) {
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
        self.organization_input(ui)
            .or(selected)
            .is_some_and(|organization| Self::select_organization(current, organization))
    }

    fn show_credentials(
        ui: &mut Ui,
        organization: &AzureDevOpsOrganizationUrl,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        personal_access_accounts: &mut HashMap<
            AzureDevOpsOrganizationUrl,
            CloudTerrastodonAccountId,
        >,
    ) {
        let previous_account = personal_access_accounts.get(organization).copied();
        let mut selected_account = previous_account;
        let label = selected_account.map_or_else(
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
            ComboBox::from_id_salt(("devops_organization_authentication", organization))
                .selected_text(label)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut selected_account, None, "Azure account");
                    for account in catalogue
                        .into_iter()
                        .flat_map(|catalogue| &catalogue.accounts)
                        .filter(|account| Self::has_active_pat(account, organization))
                    {
                        ui.selectable_value(
                            &mut selected_account,
                            Some(account.id),
                            format!("{} (PAT)", account.alias),
                        );
                    }
                })
                .response
                .labelled_by(label_response.id);
        });
        if selected_account != previous_account {
            if let Some(account) = selected_account {
                personal_access_accounts.insert(organization.clone(), account);
            } else {
                personal_access_accounts.remove(organization);
            }
        }
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
    pub(crate) fn seed_azure_devops_organization_account(
        &mut self,
        tab: PortalTabId,
        organization: &AzureDevOpsOrganizationUrl,
    ) {
        let selection = self
            .azure_devops_organization_picker
            .tracked_organizations
            .as_loaded()
            .and_then(|_| self.account_picker.catalogue.as_ref().ok())
            .and_then(|catalogue| {
                initial_organization_account(
                    catalogue,
                    organization,
                    self.azure_devops_organization_picker
                        .tracked_tenant(organization),
                )
            });
        let (bearer, pat) = match selection {
            Some(AzureDevOpsAuthSelection::Bearer(selection)) => (Some(selection), None),
            Some(AzureDevOpsAuthSelection::PersonalAccessToken { account_id }) => {
                (None, Some(account_id))
            }
            None => (None, None),
        };
        let Some(content) = self.azure_content_for_tab_mut(tab) else {
            return;
        };
        let previous_bearer = std::mem::replace(&mut content.authentication, bearer);
        let previous_pat = match pat {
            Some(pat) => content
                .devops_pat_accounts
                .insert(organization.clone(), pat),
            None => content.devops_pat_accounts.remove(organization),
        };
        self.account_picker.set_selection_error(tab, None);
        if previous_bearer != bearer || previous_pat != pat {
            self.reset_azure_content_for_tab(tab);
        }
    }

    /// Choose the audit scope independently of the credentials used in this view.
    pub(crate) fn show_azure_devops_organization_scope_picker(&mut self, ui: &mut Ui) {
        self.show_tracked_azure_devops_organizations_status(ui);
        if self
            .azure_devops_organization_picker
            .show_organization_scope(
                ui,
                &mut self.azure.azure_devops_organization,
                self.account_picker.catalogue.as_ref().ok(),
            )
        {
            self.azure.devops_organization_generation += 1;
        }
    }

    pub(crate) fn show_azure_devops_organization_picker(&mut self, ui: &mut Ui) {
        self.show_azure_devops_organization_scope_picker(ui);
        if let Some(organization) = &self.azure.azure_devops_organization {
            AzureDevOpsOrganizationPicker::show_credentials(
                ui,
                organization,
                self.account_picker.catalogue.as_ref().ok(),
                &mut self.azure.devops_pat_accounts,
            );
        }
    }

    pub(crate) fn show_azure_devops_credentials_for(
        &mut self,
        ui: &mut Ui,
        organization: &AzureDevOpsOrganizationUrl,
    ) {
        AzureDevOpsOrganizationPicker::show_credentials(
            ui,
            organization,
            self.account_picker.catalogue.as_ref().ok(),
            &mut self.azure.devops_pat_accounts,
        );
    }

    pub(crate) fn ready_azure_devops_authentication_for(
        &self,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Option<AzureDevOpsAuthSelection> {
        self.ready_azure_devops_authentication_in(&self.azure, organization)
    }

    pub(crate) fn ready_azure_devops_authentication_in(
        &self,
        content: &PortalAzureContent,
        organization: &AzureDevOpsOrganizationUrl,
    ) -> Option<AzureDevOpsAuthSelection> {
        if self.account_operation.is_some() {
            return None;
        }
        match content.devops_pat_accounts.get(organization).copied() {
            None => self
                .ready_azure_authentication_for(content.authentication)
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

    pub(crate) fn show_azure_devops_authentication_required(
        &mut self,
        ui: &mut Ui,
        organization: &AzureDevOpsOrganizationUrl,
    ) {
        if self.azure.devops_pat_accounts.contains_key(organization) {
            if self.account_operation.is_some() {
                ui.spinner();
            } else if ui.button("Manage PAT account").clicked() {
                self.account_manager.open();
                self.navigate(PortalPage::Accounts);
            }
        } else {
            self.show_azure_authentication_required(ui);
        }
    }
}

fn initial_organization_account(
    catalogue: &CloudTerrastodonAccountCatalogue,
    organization: &AzureDevOpsOrganizationUrl,
    tenant_id: Option<AzureTenantId>,
) -> Option<AzureDevOpsAuthSelection> {
    if let Some(tenant_id) = tenant_id {
        let profile = catalogue
            .resolve(
                None,
                Some(&CloudTerrastodonAccountNamespace::Entra { tenant_id }),
            )
            .ok()?;
        return profile
            .azure_bearer_selection(AuthSource::Auto)
            .ok()
            .map(AzureDevOpsAuthSelection::Bearer);
    }
    let profile = catalogue
        .resolve(
            None,
            Some(&CloudTerrastodonAccountNamespace::AzureDevOps {
                organization: organization.clone(),
            }),
        )
        .ok()?;
    (profile.select_source(AuthSource::Auto).ok()?
        == CloudTerrastodonCredentialSource::PersonalAccessToken)
        .then_some(AzureDevOpsAuthSelection::PersonalAccessToken {
            account_id: profile.id,
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth::CloudTerrastodonAccountDefault;
    use cloud_terrastodon_auth_types::AzureBearerAuthSource;
    use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
    use eyre::Result;
    use std::time::Instant;

    #[test]
    fn organization_uses_the_default_for_its_saved_tenant() -> Result<()> {
        let organization: AzureDevOpsOrganizationUrl = "test-organization".parse()?;
        let tenant_id: AzureTenantId = "00000000-0000-0000-0000-000000000010".parse()?;
        let other_tenant: AzureTenantId = "00000000-0000-0000-0000-000000000020".parse()?;
        let tenant_account = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000001".parse()?,
            "tenant-account".parse()?,
            CloudTerrastodonAccountCredential::AzureCli { tenant_id },
        );
        let other_account = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000002".parse()?,
            "other-tenant-account".parse()?,
            CloudTerrastodonAccountCredential::AzureCli {
                tenant_id: other_tenant,
            },
        );
        let pat_account = CloudTerrastodonAccountProfile::new(
            "00000000-0000-0000-0000-000000000003".parse()?,
            "org-pat".parse()?,
            CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                organization: organization.clone(),
            },
        );
        let mut catalogue = CloudTerrastodonAccountCatalogue {
            accounts: vec![
                other_account.clone(),
                pat_account.clone(),
                tenant_account.clone(),
            ],
            defaults: vec![
                CloudTerrastodonAccountDefault {
                    namespace: other_account.namespace,
                    account_id: other_account.id,
                },
                CloudTerrastodonAccountDefault {
                    namespace: pat_account.namespace,
                    account_id: pat_account.id,
                },
            ],
            ..Default::default()
        };
        assert_eq!(
            initial_organization_account(&catalogue, &organization, Some(tenant_id)),
            None,
        );
        catalogue.defaults.push(CloudTerrastodonAccountDefault {
            namespace: tenant_account.namespace,
            account_id: tenant_account.id,
        });
        assert_eq!(
            initial_organization_account(&catalogue, &organization, Some(tenant_id)),
            Some(AzureDevOpsAuthSelection::Bearer(
                AzureBearerAuthenticationSelection {
                    account_id: tenant_account.id,
                    tenant_id,
                    auth_source: AzureBearerAuthSource::AzureCli,
                }
            )),
        );
        assert_eq!(
            initial_organization_account(&catalogue, &organization, None),
            Some(AzureDevOpsAuthSelection::PersonalAccessToken {
                account_id: pat_account.id,
            }),
        );
        Ok(())
    }

    #[test]
    fn organizations_merge_registered_and_entered_scopes() -> Result<()> {
        let registered: AzureDevOpsOrganizationUrl = "registered-org".parse()?;
        let entered: AzureDevOpsOrganizationUrl = "entered-org".parse()?;
        let tracked_only: AzureDevOpsOrganizationUrl = "tracked-org".parse()?;
        let tenant = "00000000-0000-0000-0000-000000000010".parse()?;
        let now = Instant::now();
        let mut picker = AzureDevOpsOrganizationPicker {
            tracked_organizations: Loadable::Loaded {
                value: vec![
                    TrackedAzureDevOpsOrganization {
                        url: registered.clone(),
                        tenant_id: Some(tenant),
                    },
                    TrackedAzureDevOpsOrganization {
                        url: tracked_only.clone(),
                        tenant_id: None,
                    },
                ],
                started_at: now,
                finished_at: now,
            },
            ..AzureDevOpsOrganizationPicker::new(Some(&entered))
        };
        let mut current = Some(registered.clone());
        assert!(AzureDevOpsOrganizationPicker::select_organization(
            &mut current,
            tracked_only.clone()
        ));
        assert!(!AzureDevOpsOrganizationPicker::select_organization(
            &mut current,
            tracked_only
        ));

        let catalogue = CloudTerrastodonAccountCatalogue {
            accounts: vec![CloudTerrastodonAccountProfile::new(
                "00000000-0000-0000-0000-000000000001".parse()?,
                "test-account".parse()?,
                CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                    organization: registered.clone(),
                },
            )],
            ..Default::default()
        };
        let organizations: Vec<_> = picker
            .known_organizations(current.as_ref(), Some(&catalogue))
            .into_iter()
            .map(|(url, tenant)| (url.name(), tenant))
            .collect();
        assert_eq!(
            organizations,
            [
                ("entered-org", None),
                ("registered-org", Some(tenant)),
                ("tracked-org", None),
            ]
        );

        // After switching away, a removed tracked organization does not linger as a manual entry.
        assert!(AzureDevOpsOrganizationPicker::select_organization(
            &mut current,
            registered
        ));
        picker.tracked_organizations = Loadable::Loaded {
            value: Vec::new(),
            started_at: now,
            finished_at: now,
        };
        let names: Vec<_> = picker
            .known_organizations(current.as_ref(), Some(&catalogue))
            .into_iter()
            .map(|(url, _)| url.name())
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
