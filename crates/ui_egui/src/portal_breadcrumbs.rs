use crate::app::MyApp;
use crate::icons::portal_target_icon;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Button;
use eframe::egui::CursorIcon;
use eframe::egui::Image;
use eframe::egui::PointerButton;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Stroke;
use eframe::egui::TextWrapMode;
use eframe::egui::Ui;
use eframe::egui::Vec2;

impl MyApp {
    pub(crate) fn handle_mouse_history(&mut self, ui: &Ui) {
        let (back, forward) = ui.input(|input| {
            (
                input.pointer.button_pressed(PointerButton::Extra1),
                input.pointer.button_pressed(PointerButton::Extra2),
            )
        });
        if (back && self.tabs.back()) || (forward && self.tabs.forward()) {
            self.after_history_navigation();
        }
    }

    /// Returns whether a breadcrumb moved to a different destination.
    pub(crate) fn show_portal_breadcrumbs(
        &mut self,
        ui: &mut Ui,
        target: &PortalTabTarget,
    ) -> bool {
        let (visits, current) = self.tabs.history();
        if visits.len() > 1 {
            let jump = ui
                .horizontal(|ui| {
                    ui.small("History");
                    ScrollArea::horizontal()
                        .id_salt("portal-history")
                        .stick_to_right(true)
                        .show(ui, |ui| {
                            ui.horizontal(|ui| {
                                let mut index = None;
                                for (position, visit) in visits[..=current].iter().enumerate() {
                                    if position > 0 {
                                        ui.label("›");
                                    }
                                    if self.portal_breadcrumb(ui, visit, position == current) {
                                        index = Some(position);
                                    }
                                }
                                index
                            })
                            .inner
                        })
                        .inner
                })
                .inner;
            if let Some(jump) = jump
                && self.tabs.go_to(jump)
            {
                self.after_history_navigation();
                return true;
            }
        }

        let hierarchy = self.portal_hierarchy(target);
        if hierarchy.len() < 2 {
            return false;
        }
        let parent = ui
            .horizontal(|ui| {
                ui.small("Hierarchy");
                ScrollArea::horizontal()
                    .id_salt("portal-hierarchy")
                    .stick_to_right(true)
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            let mut parent = None;
                            for (position, scope) in hierarchy.into_iter().enumerate() {
                                if position > 0 {
                                    ui.label("›");
                                }
                                if self.portal_breadcrumb(ui, &scope, &scope == target) {
                                    parent = Some(scope);
                                }
                            }
                            parent
                        })
                        .inner
                    })
                    .inner
            })
            .inner;
        if let Some(parent) = parent {
            self.navigate_tab(parent);
            return true;
        }
        false
    }

    fn portal_breadcrumb(&self, ui: &mut Ui, target: &PortalTabTarget, current: bool) -> bool {
        const ICON_SIZE: f32 = 14.0;
        const ICON_GAP: f32 = 4.0;
        let title = self.portal_tab_title(target);
        let icon = Image::new(portal_target_icon(target)).fit_to_exact_size(Vec2::splat(ICON_SIZE));
        if current {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = ICON_GAP;
                ui.add(icon);
                ui.strong(title.as_ref());
            });
            false
        } else {
            let color = ui.visuals().hyperlink_color;
            let label = RichText::new(title.as_ref()).color(color);
            let response = ui
                .add(
                    Button::new((icon, label))
                        .small()
                        .frame(false)
                        .gap(ICON_GAP)
                        .wrap_mode(TextWrapMode::Extend),
                )
                .on_hover_cursor(CursorIcon::PointingHand);
            if response.hovered() || response.has_focus() {
                let mut label_rect = response.rect;
                label_rect.min.x += ICON_SIZE + ICON_GAP;
                ui.painter().hline(
                    label_rect.x_range(),
                    label_rect.bottom(),
                    Stroke::new(1.0, color),
                );
            }
            response.clicked()
        }
    }

    pub(crate) fn after_history_navigation(&mut self) {
        self.resource_search.clear();
        self.retain_open_tab_data();
    }
}
