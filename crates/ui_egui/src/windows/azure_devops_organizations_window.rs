use crate::app::MyApp;
use crate::icons::DEVOPS_ICON;
use crate::inventory_table::show_inventory_table;
use crate::portal_page::PortalPage;
use crate::table_row_background::TableRowBackground;
use eframe::egui::Image;
use eframe::egui::Label;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use egui_extras::Column;

pub(crate) fn azure_devops_organizations_ui(app: &mut MyApp, ui: &mut Ui) {
    let mut selected = app.azure_devops_organization_picker.organization_input(ui);
    let organizations = app.azure_devops_organization_picker.known_organizations(
        app.azure_devops_organization.as_ref(),
        app.account_picker.catalogue.as_ref().ok(),
    );
    if organizations.is_empty() && selected.is_none() {
        ui.label("No known Azure DevOps organizations.");
        return;
    }

    if selected.is_none() {
        show_inventory_table(ui, |table| {
            table
                .id_salt("azure_devops_organizations")
                .column(Column::initial(220.0).at_least(120.0).clip(true))
                .column(Column::remainder().at_least(130.0).clip(true))
                .header(26.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Organization");
                    });
                    header.col(|ui| {
                        ui.strong("URL");
                    });
                })
                .body(|mut body| {
                    let background = TableRowBackground::new(&mut body);
                    body.rows(28.0, organizations.len(), |mut row| {
                        let organization = organizations[row.index()];
                        let is_selected =
                            app.azure_devops_organization.as_ref() == Some(organization);
                        background.first_cell(&mut row, |ui| {
                            ui.add(Image::new(DEVOPS_ICON).fit_to_exact_size(Vec2::splat(16.0)));
                            if ui
                                .link(organization.name())
                                .on_hover_text("Browse projects")
                                .clicked()
                            {
                                selected = Some(organization.clone());
                            }
                            is_selected
                        });
                        row.col(|ui| {
                            ui.add(Label::new(organization.to_string()).truncate());
                        });
                    });
                });
        });
    }

    if let Some(organization) = selected {
        app.select_azure_devops_organization(organization);
        app.navigate(PortalPage::AzureDevOpsProjects);
    }
}
