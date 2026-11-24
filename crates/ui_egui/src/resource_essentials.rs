use crate::portal_section_tabs::SECTION_TAB_HEIGHT;
use crate::portal_section_tabs::portal_section_tabs;
use crate::portal_tab_open::PortalTabOpen;
use crate::resource_essentials_format::ResourceEssentialsFormat;
use eframe::egui::Align;
use eframe::egui::CollapsingHeader;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::FontId;
use eframe::egui::Layout;
use eframe::egui::Response;
use eframe::egui::RichText;
use eframe::egui::Stroke;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use std::sync::Arc;

/// Show resource controls and the selected representation in a collapsible section.
pub(crate) fn resource_essentials_ui(
    ui: &mut Ui,
    resource_id: &str,
    show: impl FnOnce(&mut Ui, ResourceEssentialsFormat, Option<Response>) -> Option<PortalTabOpen>,
) -> Option<PortalTabOpen> {
    let format_id = ui.make_persistent_id(("resource-essentials-format", resource_id));
    let saved_format: ResourceEssentialsFormat =
        ui.data(|data| data.get_temp(format_id)).unwrap_or_default();
    let mut format = saved_format;

    let body_style = Arc::clone(ui.style());
    ui.scope(|ui| {
        ui.spacing_mut().interact_size.y = 28.0;
        ui.spacing_mut().button_padding.y = 4.0;
        let visuals = ui.visuals_mut();
        // Framed headers use the full content width for painting and hit testing.
        visuals.collapsing_header_frame = true;
        for (style, fill) in [
            (&mut visuals.widgets.inactive, Color32::TRANSPARENT),
            (&mut visuals.widgets.hovered, Color32::from_gray(240)),
            (&mut visuals.widgets.active, Color32::from_gray(232)),
        ] {
            style.expansion = 0.0;
            style.weak_bg_fill = fill;
            style.bg_stroke = Stroke::NONE;
        }
        let section = CollapsingHeader::new(
            RichText::new("Essentials")
                .size(13.0)
                .color(Color32::from_rgb(50, 49, 48)),
        )
        .id_salt(("resource-essentials", resource_id))
        .default_open(true)
        .show_unindented(ui, |ui| {
            ui.set_style(body_style);
            ui.style_mut().override_font_id = Some(FontId::proportional(13.0));
            let ink = Color32::from_rgb(41, 40, 39);
            ui.visuals_mut().override_text_color = Some(ink);
            let open_output = ui
                .allocate_ui_with_layout(
                    Vec2::new(ui.available_width(), SECTION_TAB_HEIGHT),
                    Layout::right_to_left(Align::Center),
                    |ui| {
                        if ui.link("Copy ID").clicked() {
                            ui.ctx().copy_text(resource_id.to_owned());
                        }
                        let open = matches!(format, ResourceEssentialsFormat::Cli(_))
                            .then(|| ui.small_button("Open output in tab"));
                        ui.with_layout(
                            Layout::left_to_right(Align::Center).with_main_wrap(true),
                            |ui| {
                                portal_section_tabs(
                                    ui,
                                    &mut format,
                                    ResourceEssentialsFormat::CHOICES,
                                    ResourceEssentialsFormat::label,
                                );
                            },
                        );
                        open
                    },
                )
                .inner;
            if format != saved_format {
                ui.data_mut(|data| data.insert_temp(format_id, format));
            }
            show(ui, format, open_output)
        });
        section
            .header_response
            .on_hover_cursor(CursorIcon::PointingHand);
        section.body_returned.flatten()
    })
    .inner
}
