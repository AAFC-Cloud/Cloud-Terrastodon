use crate::app::MyApp;
use crate::azure_devops_organization_users_ui::show_azure_devops_organization_users;
use crate::icons::DEVOPS_ICON;
use crate::icons::DEVOPS_PROJECT_ICON;
use crate::icons::page_icon;
use crate::portal_azure_devops_organization_view::PortalAzureDevOpsOrganizationView;
use crate::portal_azure_devops_project::show_azure_devops_project;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_detail_header::resource_detail_header;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use crate::windows::azure_devops_projects_window::azure_devops_projects_ui;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Grid;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::Ui;

pub(crate) fn show_azure_devops_organization(
    app: &mut MyApp,
    ui: &mut Ui,
    organization: &AzureDevOpsOrganizationUrl,
    view: PortalAzureDevOpsOrganizationView,
) {
    use PortalAzureDevOpsOrganizationView::Overview;
    use PortalAzureDevOpsOrganizationView::Project;
    use PortalAzureDevOpsOrganizationView::Projects;
    use PortalAzureDevOpsOrganizationView::Users;

    if let Project(id) = view {
        show_azure_devops_project(
            app,
            ui,
            organization,
            id,
            PortalAzureDevOpsProjectView::Overview,
        );
        return;
    }

    let inventory = PortalTabTarget::Page(PortalPage::AzureDevOpsOrganizations);
    if resource_detail_header(
        ui,
        organization.name(),
        "Azure DevOps organization",
        PortalPage::AzureDevOpsOrganizations.title(),
        DEVOPS_ICON,
        &inventory,
    )
    .is_some_and(|open| open.apply(app))
    {
        return;
    }

    let expanded = app.resource_navigation_expanded;
    let open = Panel::left(ui.make_persistent_id("devops-organization-navigation"))
        .exact_size(if expanded { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "organization navigation",
            );
            let overview = navigation_tile(
                ui,
                "Overview",
                DEVOPS_ICON,
                view == Overview,
                expanded,
                || Overview.target(organization),
            );
            let projects = navigation_tile(
                ui,
                "Projects",
                DEVOPS_PROJECT_ICON,
                view == Projects,
                expanded,
                || Projects.target(organization),
            );
            let users = navigation_tile(
                ui,
                "Users",
                page_icon(PortalPage::EntraUsers),
                view == Users,
                expanded,
                || Users.target(organization),
            );
            overview.or(projects).or(users)
        })
        .inner;
    match open {
        Some(PortalTabOpen::Current(next)) => {
            app.tabs.replace_current(next);
            app.record_portal_visit();
            app.after_history_navigation();
            ui.ctx().request_repaint();
            return;
        }
        Some(open) => {
            open.apply(app);
        }
        None => {}
    }

    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 8)))
        .show(ui, |ui| {
            if view == Overview {
                app.show_tracked_azure_devops_organizations_status(ui);
                let tenant = app
                    .azure_devops_organization_picker
                    .tracked_tenant(organization);
                Grid::new("devops-organization-overview").show(ui, |ui| {
                    ui.label("Organization URL");
                    ui.hyperlink(organization.to_string());
                    ui.end_row();
                    if let Some(tenant) = tenant {
                        ui.label("Tenant ID");
                        ui.label(tenant.to_string());
                        ui.end_row();
                    }
                });
                app.show_tracked_azure_devops_organization_actions(ui, organization);
            } else if view == Users {
                show_azure_devops_organization_users(app, ui, organization);
            } else {
                azure_devops_projects_ui(app, ui, organization);
            }
        });
}
