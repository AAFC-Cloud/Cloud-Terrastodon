use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use eframe::egui::Button;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Link;
use eframe::egui::Margin;
use eframe::egui::RichText;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

/// Returns whether the user opened the inventory linked from the icon and type.
pub(crate) fn resource_detail_header(
    ui: &mut Ui,
    name: &str,
    resource_type: &str,
    inventory: PortalPage,
) -> bool {
    Frame::NONE
        .inner_margin(Margin {
            left: 12,
            right: 12,
            top: 6,
            bottom: 12,
        })
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 12.0;
                let icon = Image::new(page_icon(inventory)).fit_to_exact_size(Vec2::splat(38.0));
                let icon = ui
                    .add(Button::image(icon).frame_when_inactive(false))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text(inventory.title());
                icon.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), inventory.title())
                });
                let type_link = ui
                    .vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        let name_color = ui
                            .visuals()
                            .text_color()
                            .lerp_to_gamma(ui.visuals().weak_text_color(), 0.35);
                        ui.label(RichText::new(name).strong().size(19.0).color(name_color));
                        ui.add(Link::new(RichText::new(resource_type).size(12.0)))
                            .on_hover_text(inventory.title())
                            .clicked()
                    })
                    .inner;
                icon.clicked() || type_link
            })
            .inner
        })
        .inner
}
