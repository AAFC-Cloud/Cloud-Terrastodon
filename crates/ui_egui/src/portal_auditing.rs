use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_azure_audit::azure_audit_ui;
use crate::portal_azure_audit::storage_account_audit_ui;
use crate::portal_azure_devops_audit::show_azure_devops_audit;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::Ui;

pub(crate) fn show_portal_auditing(app: &mut MyApp, ui: &mut Ui, current: PortalPage) {
    let expanded = app.resource_navigation_expanded;
    let destination = Panel::left(ui.make_persistent_id("audit-navigation"))
        .exact_size(if expanded { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(
                ui,
                &mut app.resource_navigation_expanded,
                "audit navigation",
            );
            let mut destination = None;
            for (page, label) in [
                (PortalPage::Auditing, "Azure"),
                (PortalPage::AzureDevOpsAudit, "Azure DevOps"),
                (PortalPage::StorageAccountAudit, "Storage accounts"),
            ] {
                if let Some(open) = navigation_tile(
                    ui,
                    label,
                    page_icon(page),
                    page == current,
                    expanded,
                    || PortalTabTarget::Page(page),
                ) {
                    destination = Some(open);
                }
            }
            destination
        })
        .inner;
    if destination.is_some_and(|open| open.apply(app)) {
        return;
    }

    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 12)))
        .show(ui, |ui| {
            let (title, show): (_, fn(&mut MyApp, &mut Ui)) = match current {
                PortalPage::Auditing => ("Azure audit", azure_audit_ui),
                PortalPage::AzureDevOpsAudit => (current.title(), show_azure_devops_audit),
                PortalPage::StorageAccountAudit => (current.title(), storage_account_audit_ui),
                _ => return,
            };
            ui.label(RichText::new(title).strong().size(22.0));
            ui.add_space(12.0);
            show(app, ui);
        });
}
