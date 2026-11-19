use crate::app::MyApp;
use crate::icons::ABOUT_ICON;
use crate::icons::page_icon;
use crate::portal_page::PortalPage;
use eframe::egui::Align2;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::FontId;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Panel;
use eframe::egui::Rect;
use eframe::egui::ScrollArea;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

impl MyApp {
    /// A compact rail can expand to show destination labels without taking content space by default.
    pub fn portal_navigation(&mut self, ui: &mut Ui) {
        Panel::left("portal_navigation")
            .exact_size(if self.navigation_expanded {
                200.0
            } else {
                44.0
            })
            .resizable(false)
            .frame(Frame::new().fill(Color32::from_rgb(250, 250, 250)))
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                ui.spacing_mut().button_padding = Vec2::new(12.0, 8.0);
                if ui
                    .add_sized(
                        [ui.available_width(), 32.0],
                        Button::new(if self.navigation_expanded { "«" } else { "»" }).frame(false),
                    )
                    .on_hover_text("Expand or collapse navigation")
                    .clicked()
                {
                    self.navigation_expanded = !self.navigation_expanded;
                }
                ScrollArea::vertical().show(ui, |ui| {
                    for page in [
                        PortalPage::Home,
                        PortalPage::Resources,
                        PortalPage::Subscriptions,
                        PortalPage::ResourceGroups,
                        PortalPage::StorageAccounts,
                        PortalPage::ContainerRegistries,
                        PortalPage::KeyVaults,
                    ] {
                        self.portal_navigation_item(ui, page);
                    }
                    ui.add_space(12.0);
                    ui.separator();
                    ui.add_space(8.0);
                    for page in [
                        PortalPage::StartingPoints,
                        PortalPage::SelectedItems,
                        PortalPage::WorkDirs,
                        PortalPage::FileDrop,
                        PortalPage::Logs,
                    ] {
                        self.portal_navigation_item(ui, page);
                    }
                    ui.add_space(8.0);
                    ui.separator();
                    if ui
                        .add_sized(
                            [ui.available_width(), 40.0],
                            if self.navigation_expanded {
                                Button::image_and_text(
                                    Image::new(ABOUT_ICON).fit_to_exact_size(Vec2::splat(20.0)),
                                    "About Cloud Terrastodon",
                                )
                            } else {
                                Button::image(
                                    Image::new(ABOUT_ICON).fit_to_exact_size(Vec2::splat(20.0)),
                                )
                            }
                            .frame(false),
                        )
                        .on_hover_text("About Cloud Terrastodon")
                        .clicked()
                    {
                        self.about_open = !self.about_open;
                    }
                });
            });
    }

    fn portal_navigation_item(&mut self, ui: &mut Ui, page: PortalPage) {
        let response = ui.add_sized([ui.available_width(), 40.0], Button::new("").frame(false));
        response
            .widget_info(|| WidgetInfo::labeled(WidgetType::Button, ui.is_enabled(), page.title()));
        if self.page == page || response.hovered() || response.has_focus() {
            ui.painter().rect_filled(
                response.rect,
                0.0,
                if self.page == page {
                    Color32::from_rgb(229, 241, 251)
                } else {
                    Color32::from_rgb(237, 235, 233)
                },
            );
        }
        let icon_rect = Rect::from_center_size(
            response.rect.left_center() + Vec2::new(22.0, 0.0),
            Vec2::splat(20.0),
        );
        Image::new(page_icon(page)).paint_at(ui, icon_rect);
        if self.navigation_expanded {
            ui.painter().text(
                response.rect.left_center() + Vec2::new(44.0, 0.0),
                Align2::LEFT_CENTER,
                page.title(),
                FontId::proportional(13.0),
                Color32::from_rgb(50, 49, 48),
            );
        }
        if self.page == page {
            ui.painter().rect_filled(
                Rect::from_min_size(response.rect.min, Vec2::new(3.0, response.rect.height())),
                0.0,
                Color32::from_rgb(0, 120, 212),
            );
        }
        if response.on_hover_text(page.title()).clicked() {
            self.navigate(page);
        }
    }
}
