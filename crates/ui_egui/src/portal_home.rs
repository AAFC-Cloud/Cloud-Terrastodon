use crate::app::MyApp;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use crate::windows::resources_window::resources_ui;
use eframe::egui::Color32;
use eframe::egui::FontId;
use eframe::egui::Image;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

/// Service shortcuts and the real resource inventory share the landing page.
pub fn portal_home(app: &mut MyApp, ui: &mut Ui) {
    ui.label(RichText::new("Azure services").strong().size(19.0));
    ui.add_space(16.0);
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(8.0, 8.0);
        for page in [
            PortalPage::Subscriptions,
            PortalPage::Resources,
            PortalPage::ResourceGroups,
            PortalPage::StorageAccounts,
            PortalPage::ContainerRegistries,
            PortalPage::KeyVaults,
        ] {
            let (rect, response) = ui.allocate_exact_size(Vec2::new(112.0, 100.0), Sense::click());
            response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), page.title())
            });
            if response.hovered() || response.has_focus() {
                ui.painter()
                    .rect_filled(rect, 2.0, Color32::from_rgb(243, 248, 252));
            }
            Image::new(page_icon(page)).paint_at(
                ui,
                Rect::from_center_size(
                    Pos2::new(rect.center().x, rect.top() + 25.0),
                    Vec2::splat(34.0),
                ),
            );
            let title = ui.painter().layout(
                page.title().to_owned(),
                FontId::proportional(13.0),
                Color32::from_rgb(0, 95, 184),
                104.0,
            );
            ui.painter().galley(
                Pos2::new(rect.center().x - title.size().x / 2.0, rect.top() + 52.0),
                title,
                Color32::from_rgb(0, 95, 184),
            );
            if response
                .on_hover_text(format!("Browse {}", page.title().to_lowercase()))
                .clicked()
            {
                app.navigate(page);
            }
        }
    });
    ui.add_space(24.0);
    ui.label(RichText::new("Resources").strong().size(19.0));
    ui.add_space(12.0);
    resources_ui(app, ui);
}
