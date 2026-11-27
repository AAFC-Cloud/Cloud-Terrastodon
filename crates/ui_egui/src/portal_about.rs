use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use crate::portal_typography;
use crate::scope_navigation::navigation_tile;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use eframe::egui::CentralPanel;
use eframe::egui::Frame;
use eframe::egui::Margin;
use eframe::egui::Panel;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;

pub(crate) fn show_portal_about(app: &mut MyApp, ui: &mut Ui, target: &PortalTabTarget) {
    match about_navigation(ui, target, &mut app.resource_navigation_expanded) {
        Some(PortalTabOpen::Current(target)) => {
            app.navigate_current_tab(target);
            return;
        }
        Some(PortalTabOpen::Background(target)) => {
            app.open_background_tab(target);
        }
        None => {}
    }

    CentralPanel::default()
        .frame(Frame::NONE.inner_margin(Margin::symmetric(16, 12)))
        .show(ui, |ui| {
            ui.label(
                RichText::new(app.portal_tab_title(target).as_ref())
                    .strong()
                    .size(22.0),
            );
            ui.add_space(12.0);
            match target {
                PortalTabTarget::Page(PortalPage::Logs) => app.show_portal_logs(ui),
                PortalTabTarget::Page(PortalPage::BackgroundWork) => app.show_background_work(ui),
                PortalTabTarget::Page(PortalPage::Accounts) => app.show_accounts_page(ui),
                PortalTabTarget::Page(PortalPage::AzureTenants) => {
                    app.show_azure_tenants_page(ui, None)
                }
                PortalTabTarget::AzureTenant(id) => app.show_azure_tenants_page(ui, Some(*id)),
                _ => {
                    ui.label(&app.app_info);
                    ui.add_space(10.0);
                    ui.hyperlink_to("GitHub", "https://github.com/AAFC-Cloud/Cloud-Terrastodon");
                    ui.add_space(10.0);
                    ui.collapsing("Allo font license", |ui| {
                        ScrollArea::vertical().max_height(240.0).show(ui, |ui| {
                            ui.label(portal_typography::LICENSE);
                        });
                    });
                }
            }
        });
}

fn about_navigation(
    ui: &mut Ui,
    current: &PortalTabTarget,
    expanded: &mut bool,
) -> Option<PortalTabOpen> {
    let show_labels = *expanded;
    // egui uses panel IDs as-is, even when the containing Ui is scoped.
    Panel::left(ui.make_persistent_id("portal-about-navigation"))
        .exact_size(if show_labels { 174.0 } else { 44.0 })
        .resizable(false)
        .frame(Frame::NONE)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            sidebar_expansion_toggle(ui, expanded, "about navigation");
            let mut destination = None;
            for page in [
                PortalPage::About,
                PortalPage::Logs,
                PortalPage::BackgroundWork,
                PortalPage::Accounts,
                PortalPage::AzureTenants,
            ] {
                if let Some(open) = navigation_tile(
                    ui,
                    page.title(),
                    page_icon(page),
                    *current == PortalTabTarget::Page(page),
                    show_labels,
                    || PortalTabTarget::Page(page),
                ) {
                    destination = Some(open);
                }
            }
            destination
        })
        .inner
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Context;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Rect;
    use eframe::egui::Shape;
    use eframe::egui::UiBuilder;
    use eframe::egui::Vec2;

    #[test]
    fn logs_and_background_work_can_render_their_navigation_together() {
        let context = Context::default();
        context.options_mut(|options| options.warn_on_id_clash = true);
        let input = RawInput {
            screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(650.0, 530.0))),
            ..Default::default()
        };
        let mut output = context.run_ui(input, |ui| {
            let mut expanded = true;
            for (page, top) in [(PortalPage::Logs, 0.0), (PortalPage::BackgroundWork, 265.0)] {
                let bounds = Rect::from_min_size(Pos2::new(0.0, top), Vec2::new(650.0, 265.0));
                let mut pane =
                    ui.new_child(UiBuilder::new().id_salt(page.title()).max_rect(bounds));
                about_navigation(&mut pane, &PortalTabTarget::Page(page), &mut expanded);
            }
        });
        output.textures_delta.clear();

        let labels: Vec<_> = output
            .shapes
            .iter()
            .filter_map(|clipped| {
                if let Shape::Text(text) = &clipped.shape {
                    Some(text.galley.text())
                } else {
                    None
                }
            })
            .collect();
        for page in [
            PortalPage::About,
            PortalPage::Accounts,
            PortalPage::AzureTenants,
        ] {
            assert_eq!(
                labels
                    .iter()
                    .filter(|&&label| label == page.title())
                    .count(),
                2
            );
        }
        assert!(!labels.iter().any(|label| label.contains("use of Panel ID")));
    }
}
