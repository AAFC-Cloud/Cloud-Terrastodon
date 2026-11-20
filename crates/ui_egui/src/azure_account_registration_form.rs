use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_registration::AzureAccountRegistration;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialFile;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantArgument;
use cloud_terrastodon_azure::AzureTenantId;
use cloud_terrastodon_azure::EntraApplicationClientId;
use eframe::egui::ComboBox;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eyre::Result;
use eyre::WrapErr;
use eyre::ensure;
use std::path::Path;

/// Editable inputs for registration. Validation converts them to an account action.
pub(crate) struct AzureAccountRegistrationForm {
    source: CloudTerrastodonCredentialSource,
    alias: String,
    tenant: String,
    client_id: String,
    assertion_file: String,
    organization: String,
    instance: String,
    tea_login: String,
    tea_configuration: String,
}

impl Default for AzureAccountRegistrationForm {
    fn default() -> Self {
        Self {
            source: CloudTerrastodonCredentialSource::Browser,
            alias: String::new(),
            tenant: String::new(),
            client_id: String::new(),
            assertion_file: String::new(),
            organization: String::new(),
            instance: String::new(),
            tea_login: String::new(),
            tea_configuration: String::new(),
        }
    }
}

impl AzureAccountRegistrationForm {
    pub fn show(
        &mut self,
        ui: &mut Ui,
        tracked_tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
        open_tenants: &mut bool,
    ) -> Result<Option<AzureAccountAction>> {
        let registration_source_label = |source| match source {
            CloudTerrastodonCredentialSource::AzureCli => "Azure CLI (supports device code)",
            _ => source_label(source),
        };
        ui.label("Authentication source");
        ComboBox::from_id_salt("account_registration_source")
            .selected_text(registration_source_label(self.source))
            .show_ui(ui, |ui| {
                for source in [
                    CloudTerrastodonCredentialSource::Browser,
                    CloudTerrastodonCredentialSource::AzureCli,
                    CloudTerrastodonCredentialSource::WorkloadIdentity,
                    CloudTerrastodonCredentialSource::PersonalAccessToken,
                    CloudTerrastodonCredentialSource::Gitea,
                ] {
                    ui.selectable_value(
                        &mut self.source,
                        source,
                        registration_source_label(source),
                    );
                }
            });
        text_input(ui, "Account alias", &mut self.alias);
        if matches!(
            self.source,
            CloudTerrastodonCredentialSource::Browser
                | CloudTerrastodonCredentialSource::AzureCli
                | CloudTerrastodonCredentialSource::WorkloadIdentity
        ) {
            text_input(ui, "Tenant UUID or tracked alias", &mut self.tenant);
            ui.horizontal_wrapped(|ui| {
                if !tracked_tenants.is_empty() {
                    ui.menu_button("Choose tracked tenant", |ui| {
                        ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                            for (id, aliases) in tracked_tenants {
                                let label = aliases.first().map_or_else(
                                    || id.to_string(),
                                    |alias| format!("{alias} · {id}"),
                                );
                                if ui.button(label).clicked() {
                                    self.tenant = id.to_string();
                                    ui.close();
                                }
                            }
                        });
                    });
                }
                if ui.button("Manage tenants").clicked() {
                    *open_tenants = true;
                }
            });
        }
        match self.source {
            CloudTerrastodonCredentialSource::Browser => {
                text_input(ui, "Application client ID", &mut self.client_id);
            }
            CloudTerrastodonCredentialSource::AzureCli => {}
            CloudTerrastodonCredentialSource::WorkloadIdentity => {
                text_input(ui, "Application client ID", &mut self.client_id);
                text_input(ui, "Assertion file path", &mut self.assertion_file);
            }
            CloudTerrastodonCredentialSource::PersonalAccessToken => {
                text_input(
                    ui,
                    "Azure DevOps organization name or URL",
                    &mut self.organization,
                );
            }
            CloudTerrastodonCredentialSource::Gitea => {
                text_input(ui, "Gitea instance URL", &mut self.instance);
                text_input(ui, "Tea login profile", &mut self.tea_login);
                text_input(
                    ui,
                    "Tea configuration file path",
                    &mut self.tea_configuration,
                );
            }
        }
        if ui.button("Register account").clicked() {
            self.action().map(Some)
        } else {
            Ok(None)
        }
    }

    fn action(&self) -> Result<AzureAccountAction> {
        let alias = self
            .alias
            .trim()
            .parse()
            .wrap_err("Enter a valid account alias")?;
        let registration = match self.source {
            CloudTerrastodonCredentialSource::AzureCli => {
                AzureAccountRegistration::AzureCli(self.tenant_argument()?)
            }
            CloudTerrastodonCredentialSource::Browser => AzureAccountRegistration::Browser {
                tenant: self.tenant_argument()?,
                client_id: parse_client_id(&self.client_id)?,
            },
            CloudTerrastodonCredentialSource::WorkloadIdentity => {
                AzureAccountRegistration::WorkloadIdentity {
                    tenant: self.tenant_argument()?,
                    client_id: parse_client_id(&self.client_id)?,
                    assertion_file: credential_file(&self.assertion_file)
                        .wrap_err("Enter an assertion file path")?,
                }
            }
            CloudTerrastodonCredentialSource::PersonalAccessToken => {
                AzureAccountRegistration::Service(
                    CloudTerrastodonAccountCredential::AzureDevOpsPersonalAccessToken {
                        organization: self.organization.trim().parse()?,
                    },
                )
            }
            CloudTerrastodonCredentialSource::Gitea => {
                AzureAccountRegistration::Service(CloudTerrastodonAccountCredential::Gitea {
                    instance: self.instance.trim().parse()?,
                    login: self.tea_login.trim().to_owned(),
                    configuration: credential_file(&self.tea_configuration)
                        .wrap_err("Enter a tea configuration file path")?,
                })
            }
        };
        Ok(AzureAccountAction::Add {
            alias,
            registration,
        })
    }

    fn tenant_argument(&self) -> Result<AzureTenantArgument<'static>> {
        let tenant = self
            .tenant
            .trim()
            .parse()
            .wrap_err("Enter a tenant UUID or tracked alias")?;
        ensure!(
            !matches!(tenant, AzureTenantArgument::Default),
            "'default' selects Cloud Terrastodon's configured tenant. Enter the tenant UUID or tracked alias for this account."
        );
        Ok(tenant)
    }
}

fn credential_file(value: &str) -> Result<CloudTerrastodonCredentialFile> {
    let value = value.trim();
    ensure!(!value.is_empty(), "A file path is required");
    std::path::absolute(Path::new(value))?.try_into()
}

pub(crate) fn parse_client_id(value: &str) -> Result<EntraApplicationClientId> {
    value
        .trim()
        .parse()
        .wrap_err("Enter a valid application client ID")
}

pub(crate) fn source_label(source: CloudTerrastodonCredentialSource) -> &'static str {
    match source {
        CloudTerrastodonCredentialSource::AzureCli => "Azure CLI",
        CloudTerrastodonCredentialSource::Browser => "Browser",
        CloudTerrastodonCredentialSource::WorkloadIdentity => "Workload identity",
        CloudTerrastodonCredentialSource::PersonalAccessToken => "Azure DevOps PAT",
        CloudTerrastodonCredentialSource::Gitea => "Gitea (tea)",
    }
}

pub(crate) fn text_input(ui: &mut Ui, label: &str, value: &mut String) {
    let label = ui.label(label);
    ui.text_edit_singleline(value).labelled_by(label.id);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registration_requires_an_explicit_entra_tenant() -> Result<()> {
        let mut form = AzureAccountRegistrationForm {
            source: CloudTerrastodonCredentialSource::AzureCli,
            alias: "synthetic-account".to_owned(),
            tenant: " DeFaUlT ".to_owned(),
            ..Default::default()
        };
        assert!(matches!(
            form.action(),
            Err(error) if error.to_string().contains("configured tenant")
        ));

        form.tenant = "00000000-0000-0000-0000-000000000001".to_owned();
        assert!(matches!(
            form.action()?,
            AzureAccountAction::Add {
                registration: AzureAccountRegistration::AzureCli(AzureTenantArgument::Id(_)),
                ..
            }
        ));
        form.tenant = "synthetic-tenant".to_owned();
        assert!(matches!(
            form.action()?,
            AzureAccountAction::Add {
                registration: AzureAccountRegistration::AzureCli(AzureTenantArgument::Alias(_)),
                ..
            }
        ));
        Ok(())
    }
}
