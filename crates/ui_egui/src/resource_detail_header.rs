use crate::portal_tab_open::PortalTabOpen;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Button;
use eframe::egui::CursorIcon;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::ImageSource;
use eframe::egui::Margin;
use eframe::egui::RichText;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

/// Link to the inventory from the resource icon and type label.
pub(crate) fn resource_detail_header(
    ui: &mut Ui,
    name: &str,
    resource_type: &str,
    inventory_title: &str,
    icon: ImageSource<'static>,
    inventory: &PortalTabTarget,
) -> Option<PortalTabOpen> {
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
                let icon = Image::new(icon).fit_to_exact_size(Vec2::splat(38.0));
                let icon = ui
                    .add(Button::image(icon).frame_when_inactive(false))
                    .on_hover_cursor(CursorIcon::PointingHand)
                    .on_hover_text(inventory_title);
                icon.widget_info(|| {
                    WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), inventory_title)
                });
                let type_link = ui
                    .vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 2.0;
                        ui.label(RichText::new(name).strong().size(22.0));
                        ui.add(
                            Button::new(RichText::new(resource_type).size(12.0).weak())
                                .small()
                                .frame(false),
                        )
                        .on_hover_cursor(CursorIcon::PointingHand)
                        .on_hover_text(inventory_title)
                    })
                    .inner;
                PortalTabOpen::from_response(&icon, || inventory.clone())
                    .or_else(|| PortalTabOpen::from_response(&type_link, || inventory.clone()))
            })
            .inner
        })
        .inner
}
