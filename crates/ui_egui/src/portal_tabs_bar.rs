use crate::app::MyApp;
use crate::icons::portal_target_icon;
use crate::portal_page::PortalPage;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Align;
use eframe::egui::Atom;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::Frame;
use eframe::egui::Image;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Margin;
use eframe::egui::Modifiers;
use eframe::egui::Panel;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::ScrollArea;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;
use std::borrow::Cow;

const NEW_TAB_SHORTCUT: KeyboardShortcut = KeyboardShortcut::new(Modifiers::CTRL, Key::T);

impl MyApp {
    pub(crate) fn portal_tabs_bar(&mut self, ui: &mut Ui) {
        let mut new_tab = ui.input_mut(|input| input.consume_shortcut(&NEW_TAB_SHORTCUT));
        let mut select = None;
        let mut keep = None;
        let mut close = None;
        Panel::left("portal_vertical_tabs")
            .default_size(188.0)
            .size_range(142.0..=320.0)
            .resizable(true)
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(242, 246, 249))
                    .inner_margin(Margin::same(6)),
            )
            .show(ui, |ui| {
                ScrollArea::vertical().stick_to_bottom(true).show(ui, |ui| {
                    ui.spacing_mut().item_spacing = Vec2::splat(3.0);
                    for (index, tab, active) in self.tabs.iter() {
                        ui.push_id(index, |ui| {
                            ui.horizontal(|ui| {
                                let title = self.portal_tab_title(tab.target());
                                let text = RichText::new(title.as_ref());
                                let text = if tab.preview { text.italics() } else { text };
                                let icon = Image::new(portal_target_icon(tab.target()))
                                    .fit_to_exact_size(Vec2::splat(16.0));
                                let row_size = Vec2::new(ui.available_width(), 32.0);
                                let row_rect = Rect::from_min_size(
                                    ui.available_rect_before_wrap().min,
                                    row_size,
                                );
                                let close_id = ui.make_persistent_id("close_tab");
                                let show_close = active
                                    || ui.rect_contains_pointer(row_rect)
                                    || ui.memory(|memory| memory.has_focus(close_id));
                                let close_size = Vec2::new(26.0, row_size.y);
                                let button_size = if show_close {
                                    Vec2::new(
                                        (row_size.x - close_size.x - ui.spacing().item_spacing.x)
                                            .max(0.0),
                                        row_size.y,
                                    )
                                } else {
                                    row_size
                                };
                                let response = ui
                                    .add_sized(
                                        button_size,
                                        Button::selectable(active, (icon, text, Atom::grow()))
                                            .min_size(button_size)
                                            .gap(8.0)
                                            .truncate(),
                                    )
                                    .on_hover_text(title.as_ref());
                                if response.clicked() {
                                    select = Some(index);
                                }
                                if tab.preview && response.double_clicked() {
                                    keep = Some(index);
                                }
                                if response.middle_clicked() {
                                    close = Some(index);
                                }
                                response.context_menu(|ui| {
                                    if tab.preview && ui.button("Keep open").clicked() {
                                        keep = Some(index);
                                        ui.close();
                                    }
                                    if ui.button("Close").clicked() {
                                        close = Some(index);
                                        ui.close();
                                    }
                                });
                                if show_close {
                                    let (_, rect) = ui.allocate_space(close_size);
                                    let label = format!("Close {title}");
                                    let button = ui.interact(rect, close_id, Sense::click());
                                    button.widget_info(|| {
                                        WidgetInfo::labeled(
                                            WidgetType::Button,
                                            ui.is_enabled(),
                                            &label,
                                        )
                                    });
                                    let visuals = ui.style().interact(&button);
                                    if button.hovered() || button.has_focus() {
                                        ui.painter().rect_filled(
                                            rect,
                                            visuals.corner_radius,
                                            visuals.weak_bg_fill,
                                        );
                                    }
                                    let center = rect.center();
                                    let stroke = visuals.fg_stroke;
                                    for direction in [-1.0, 1.0] {
                                        let arm = Vec2::new(3.5, direction * 3.5);
                                        ui.painter()
                                            .line_segment([center - arm, center + arm], stroke);
                                    }
                                    let button = button.on_hover_text(label);
                                    if button.clicked() || button.middle_clicked() {
                                        close = Some(index);
                                    }
                                }
                            });
                        });
                    }
                    ui.separator();
                    let shortcut = ui.ctx().format_shortcut(&NEW_TAB_SHORTCUT);
                    new_tab |= ui
                        .add_sized(
                            [ui.available_width(), 32.0],
                            Button::new(("+", "New tab"))
                                .shortcut_text(shortcut)
                                .frame_when_inactive(false),
                        )
                        .clicked();
                    if new_tab {
                        ui.scroll_to_cursor(Some(Align::Max));
                    }
                });
            });
        if let Some(index) = select {
            self.tabs.select(index);
        }
        if let Some(index) = keep {
            self.tabs.keep_open(index);
        }
        if let Some(index) = close {
            if let Some(target) = self.tabs.close(index)
                && !self.tabs.contains(&target)
            {
                match target {
                    PortalTabTarget::LocalPath(path) => {
                        self.egui_config.open_dirs.remove(&path);
                    }
                    PortalTabTarget::Page(PortalPage::Accounts) => {
                        self.account_manager = Default::default();
                    }
                    PortalTabTarget::Page(PortalPage::AzureTenants) => {
                        self.tenant_manager.close_view();
                    }
                    _ => {}
                }
            }
            self.retain_open_tab_data();
        }
        if new_tab {
            self.tabs.open_new_tab();
            self.resource_search.clear();
        }
        if new_tab || select.is_some() || keep.is_some() || close.is_some() {
            ui.ctx().request_repaint();
        }
    }

    pub(crate) fn portal_tab_title<'a>(&'a self, target: &'a PortalTabTarget) -> Cow<'a, str> {
        if let PortalTabTarget::ScopeView { scope, view } = target {
            return Cow::Owned(format!(
                "{} · {}",
                self.portal_tab_title(&scope.overview()),
                view.label()
            ));
        }
        if let PortalTabTarget::ManagementGroup(id) = target
            && let Some(title) = self.management_group_title(id)
        {
            return Cow::Borrowed(title);
        }
        if let PortalTabTarget::Subscription(id) = target {
            let name = self
                .subscriptions
                .as_loaded()
                .and_then(|subscriptions| {
                    subscriptions
                        .iter()
                        .find(|subscription| subscription.id == *id)
                        .map(|subscription| subscription.name.as_str())
                })
                .or_else(|| {
                    self.resource_groups
                        .as_loaded()?
                        .values()
                        .find(|group| group.id.subscription_id == *id)
                        .map(|group| group.subscription_name.as_str())
                });
            if let Some(name) = name {
                return Cow::Borrowed(name);
            }
        }
        target.title()
    }
}
