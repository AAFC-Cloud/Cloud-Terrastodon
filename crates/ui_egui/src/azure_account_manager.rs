use crate::azure_account_action::AzureAccountAction;
use crate::azure_account_confirmation::AzureAccountConfirmation;
use crate::azure_account_details::AzureAccountDetails;
use crate::azure_account_inventory_row::AzureAccountInventoryRow;
use crate::azure_account_manager_page::AzureAccountManagerPage;
use crate::reflected_table::ReflectedTable;
use crate::reflected_table_column::ReflectedTableColumn;
use cloud_terrastodon_auth::CloudTerrastodonAccountCatalogue;
use cloud_terrastodon_auth_types::CloudTerrastodonAccountId;
use cloud_terrastodon_auth_types::CloudTerrastodonCredentialSource;
use cloud_terrastodon_azure::AzureTenantAlias;
use cloud_terrastodon_azure::AzureTenantId;
use eframe::egui::Id;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eyre::Result;
use std::collections::HashMap;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::gui::DiagnosticReportView;

/// Manage local account registrations and their credentials from the GUI.
#[derive(Default)]
pub(crate) struct AzureAccountManager {
    page: AzureAccountManagerPage,
    error: Option<DiagnosticReport>,
}

impl AzureAccountManager {
    pub(crate) fn cli_command(&self) -> Option<String> {
        match &self.page {
            AzureAccountManagerPage::Accounts => Some("ct account list".into()),
            AzureAccountManagerPage::Account(details) => {
                Some(format!("ct account show {}", details.account_id()))
            }
            AzureAccountManagerPage::Add(_) => None,
        }
    }

    pub fn open(&mut self) {
        self.navigate(AzureAccountManagerPage::Accounts);
    }

    pub fn open_add(&mut self) {
        self.navigate(AzureAccountManagerPage::Add(Default::default()));
    }

    pub fn open_account(&mut self, id: CloudTerrastodonAccountId) {
        self.navigate(AzureAccountManagerPage::Account(AzureAccountDetails::new(
            id,
        )));
    }

    pub fn open_reauthentication(
        &mut self,
        id: CloudTerrastodonAccountId,
        source: CloudTerrastodonCredentialSource,
    ) {
        self.navigate(AzureAccountManagerPage::Account(
            AzureAccountDetails::with_confirmation(
                id,
                AzureAccountConfirmation::Reauthenticate(source),
            ),
        ));
    }

    pub fn show_error(&mut self, error: DiagnosticReport) {
        self.error = Some(error);
    }

    fn navigate(&mut self, page: AzureAccountManagerPage) {
        self.page = page;
        self.error = None;
    }

    pub fn show(
        &mut self,
        ui: &mut Ui,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        busy: bool,
        tracked_tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
        open_tenants: &mut bool,
    ) -> Option<AzureAccountAction> {
        if let Some(error) = &self.error {
            ui.collapsing(error.report.to_string(), |ui| {
                DiagnosticReportView::new(error).show(ui);
            });
        }
        let response = ui
            .add_enabled_ui(!busy, |ui| {
                self.show_page(ui, catalogue, tracked_tenants, open_tenants)
            })
            .inner;
        match response {
            Ok(Some(next)) => {
                self.error = None;
                Some(next)
            }
            Ok(None) => None,
            Err(error) => {
                self.show_error(DiagnosticReport::new(error));
                None
            }
        }
    }

    fn show_page(
        &mut self,
        ui: &mut Ui,
        catalogue: Option<&CloudTerrastodonAccountCatalogue>,
        tracked_tenants: &[(AzureTenantId, Vec<AzureTenantAlias>)],
        open_tenants: &mut bool,
    ) -> Result<Option<AzureAccountAction>> {
        if matches!(self.page, AzureAccountManagerPage::Accounts) {
            ui.horizontal_wrapped(|ui| {
                if ui.button("Add account").clicked() {
                    self.open_add();
                }
                if ui.button("Tenants").clicked() {
                    *open_tenants = true;
                }
            });
            ui.add_space(8.0);
        } else if ui.link("Back to accounts").clicked() {
            self.open();
        }

        if matches!(self.page, AzureAccountManagerPage::Accounts) {
            if let Some(catalogue) = catalogue {
                if let Some(account) = show_accounts_table(ui, catalogue) {
                    self.open_account(account);
                }
            } else {
                ui.label("Could not load accounts. Use the toolbar’s refresh button to try again.");
            }
            return Ok(None);
        }

        ScrollArea::vertical()
            .id_salt("account-management-form")
            .show(ui, |ui| {
                ui.set_max_width(ui.available_width().min(820.0));
                match &mut self.page {
                    AzureAccountManagerPage::Accounts => Ok(None),
                    AzureAccountManagerPage::Add(form) => {
                        ui.heading("Add account");
                        form.show(ui, tracked_tenants, open_tenants)
                    }
                    AzureAccountManagerPage::Account(details) => {
                        let Some(catalogue) = catalogue else {
                            // Do not retain a secret while the catalogue is unavailable.
                            details.clear_drafts();
                            return Ok(None);
                        };
                        details.show(ui, catalogue)
                    }
                }
            })
            .inner
    }
}

fn show_accounts_table(
    ui: &mut Ui,
    catalogue: &CloudTerrastodonAccountCatalogue,
) -> Option<CloudTerrastodonAccountId> {
    let columns = [
        ReflectedTableColumn::new("Alias", "alias")
            .with_width(174.0)
            .with_identifier("id")
            .with_row_link(),
        ReflectedTableColumn::new("Scope", "scope").groupable(),
        ReflectedTableColumn::new("Preferred source", "preferred_source")
            .with_width(122.0)
            .groupable(),
        ReflectedTableColumn::new("Preferred session", "session").with_width(132.0),
        ReflectedTableColumn::new("Service default", "service_default").with_width(116.0),
    ];
    let rows = catalogue
        .accounts
        .iter()
        .map(|account| {
            let is_default = catalogue
                .defaults
                .iter()
                .any(|default| default.account_id == account.id);
            AzureAccountInventoryRow::new(account, is_default)
        })
        .collect::<Vec<_>>();
    let mut activated = None;
    let _ = ReflectedTable::new("accounts", &columns)
        .without_selection()
        .with_search_hint("Search accounts")
        .with_result_count()
        .show_explorable_with_activation(
            ui,
            &rows,
            |row| Id::new(row.id),
            &mut HashMap::new(),
            |row| activated = Some(row.id),
        );
    activated
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountAlias;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountCredential;
    use cloud_terrastodon_auth_types::CloudTerrastodonAccountProfile;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::Modifiers;
    use eframe::egui::PointerButton;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::Vec2;

    #[test]
    fn accounts_with_the_same_alias_open_by_id_after_filtering_by_scope() -> Result<()> {
        let first_id = "00000000-0000-0000-0000-000000000001".parse()?;
        let second_id = "00000000-0000-0000-0000-000000000002".parse()?;
        let first_tenant = "00000000-0000-0000-0000-000000000003".parse::<AzureTenantId>()?;
        let second_tenant = "00000000-0000-0000-0000-000000000004".parse::<AzureTenantId>()?;
        let alias = "shared-alias".parse::<CloudTerrastodonAccountAlias>()?;
        let account = |id, tenant_id| {
            CloudTerrastodonAccountProfile::new(
                id,
                alias.clone(),
                CloudTerrastodonAccountCredential::AzureCli { tenant_id },
            )
        };
        let catalogue = CloudTerrastodonAccountCatalogue {
            accounts: vec![
                account(first_id, first_tenant),
                account(second_id, second_tenant),
            ],
            ..Default::default()
        };
        let context = Context::default();
        let draw = |events| {
            let mut opened = None;
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(950.0, 320.0))),
                    events,
                    ..Default::default()
                },
                |ui| {
                    let search_id = ui.id().with(Id::new("accounts")).with("search");
                    ui.data_mut(|data| data.insert_temp(search_id, second_tenant.to_string()));
                    opened = show_accounts_table(ui, &catalogue);
                },
            );
            output.textures_delta.clear();
            (output, opened)
        };
        let (first_frame, _) = draw(Vec::new());
        let positions: Vec<_> = first_frame
            .shapes
            .iter()
            .filter_map(|clipped| match &clipped.shape {
                Shape::Text(shape) if shape.galley.text() == "shared-alias" => {
                    Some(shape.visual_bounding_rect().center())
                }
                _ => None,
            })
            .collect();
        assert_eq!(positions.len(), 1);
        let pos = positions[0];
        let click = |pressed| {
            vec![
                Event::PointerMoved(pos),
                Event::PointerButton {
                    pos,
                    button: PointerButton::Primary,
                    pressed,
                    modifiers: Modifiers::NONE,
                },
            ]
        };
        draw(click(true));
        let (_, opened) = draw(click(false));
        assert_eq!(opened, Some(second_id));
        Ok(())
    }
}
