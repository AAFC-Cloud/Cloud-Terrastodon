use crate::app::MyApp;
use crate::icons::portal_target_icon_in;
use crate::portal_azure_content::PortalAzureContent;
use crate::portal_azure_devops_project_view::PortalAzureDevOpsProjectView;
use crate::portal_pane_id::PortalPaneId;
use crate::portal_tab::PortalTab;
use crate::portal_tab_action::PortalTabAction;
use crate::portal_tab_drag::PortalTabDrag;
use crate::portal_tab_group_ui::show_tab_group_header;
use crate::portal_tab_menu::CLOSE_TAB_SHORTCUT;
use crate::portal_tab_menu::DUPLICATE_TAB_SHORTCUT;
use crate::portal_tab_menu::NEW_TAB_SHORTCUT;
use crate::portal_tab_menu::REFRESH_TAB_SHORTCUT;
use crate::portal_tab_menu::REOPEN_TAB_SHORTCUT;
use crate::portal_tab_target::PortalTabTarget;
use crate::resource_output::format_label;
use crate::sidebar_expansion_toggle::sidebar_expansion_toggle;
use eframe::egui::Align;
use eframe::egui::Area;
use eframe::egui::Atom;
use eframe::egui::Button;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::DragAndDrop;
use eframe::egui::Frame;
use eframe::egui::Id;
use eframe::egui::Image;
use eframe::egui::Margin;
use eframe::egui::Order;
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

impl MyApp {
    pub(crate) fn handle_portal_tab_shortcuts(&mut self, ui: &mut Ui) {
        let active = self.tabs.active_id();
        let action = ui.input_mut(|input| {
            if input.consume_shortcut(&REOPEN_TAB_SHORTCUT) {
                Some(PortalTabAction::Reopen)
            } else if input.consume_shortcut(&NEW_TAB_SHORTCUT) {
                Some(PortalTabAction::Open)
            } else if input.consume_shortcut(&DUPLICATE_TAB_SHORTCUT) {
                Some(PortalTabAction::Duplicate(active))
            } else if input.consume_shortcut(&CLOSE_TAB_SHORTCUT) {
                Some(PortalTabAction::Close(active))
            } else if input.consume_shortcut(&REFRESH_TAB_SHORTCUT) {
                Some(PortalTabAction::Refresh(active))
            } else {
                None
            }
        });
        if let Some(action) = action {
            self.apply_portal_tab_action(ui, action);
        }
    }

    pub(crate) fn portal_tabs_bar(&mut self, ui: &mut Ui, pane: PortalPaneId) {
        let expanded = self.tabs.pane_expanded(pane);
        let mut next_expansion = expanded;
        let mut action = None;
        let panel_id = ui.make_persistent_id(("portal_vertical_tabs", pane, expanded));
        let panel = if expanded {
            let max_width = (ui.available_width() * 0.62).clamp(0.0, 320.0);
            Panel::left(panel_id)
                .default_size(188.0_f32.min(max_width))
                .size_range(142.0_f32.min(max_width)..=max_width)
                .resizable(true)
        } else {
            Panel::left(panel_id)
                .exact_size(44.0_f32.min(ui.available_width()))
                .resizable(false)
        };
        panel
            .frame(
                Frame::new()
                    .fill(Color32::from_rgb(242, 246, 249))
                    .inner_margin(Margin::same(6)),
            )
            .show(ui, |ui| {
                sidebar_expansion_toggle(ui, &mut next_expansion, "browser tabs");
                ui.spacing_mut().item_spacing = Vec2::splat(3.0);
                self.show_pinned_tabs(ui, pane, &mut action);
                let list_bounds = ui.available_rect_before_wrap();
                let list = ScrollArea::vertical()
                    .id_salt("portal-tab-order")
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        self.show_regular_tabs(ui, pane, expanded, &mut action);
                        let end_y = ui.cursor().top();
                        ui.separator();
                        self.show_new_tab_button(ui, pane, expanded, &mut action);
                        end_y
                    });
                if let Some(drop) = PortalTabDrag::drop_at_end(
                    ui,
                    list.inner_rect.intersect(list_bounds),
                    list.inner,
                    &self.tabs,
                    pane,
                ) {
                    action = Some(drop);
                }
            });
        if expanded != next_expansion {
            self.tabs.set_pane_expanded(pane, next_expansion);
        }
        if let Some(action) = action {
            self.apply_portal_tab_action(ui, action);
        }
    }

    fn show_pinned_tabs(
        &self,
        ui: &mut Ui,
        pane: PortalPaneId,
        action: &mut Option<PortalTabAction>,
    ) {
        let mut pinned = self
            .tabs
            .iter_in_pane(pane)
            .take_while(|(_, tab, _)| tab.pinned)
            .peekable();
        if pinned.peek().is_some() {
            ScrollArea::vertical()
                .id_salt("pinned-portal-tabs")
                .auto_shrink([false, true])
                .max_height((ui.available_height() / 3.0).clamp(0.0, 102.0))
                .show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for (_, tab, active) in pinned {
                            self.show_tab_button(ui, tab, active, false, Vec2::splat(30.0), action);
                        }
                    });
                });
            ui.separator();
        }
    }

    fn show_regular_tabs(
        &self,
        ui: &mut Ui,
        pane: PortalPaneId,
        expanded: bool,
        action: &mut Option<PortalTabAction>,
    ) {
        let active_group = self
            .tabs
            .pane_active(pane)
            .and_then(|id| self.tabs.tab(id))
            .and_then(|tab| tab.group);
        let mut previous_group = None;
        for (_, tab, active) in self
            .tabs
            .iter_in_pane(pane)
            .skip_while(|(_, tab, _)| tab.pinned)
        {
            let group = tab.group.and_then(|id| self.tabs.group(id));
            if let Some(group) = group
                && tab.group != previous_group
            {
                show_tab_group_header(
                    ui,
                    group,
                    tab.id,
                    expanded,
                    active_group == Some(group.id),
                    &self.tabs,
                    action,
                );
            }
            previous_group = tab.group;
            if group.is_some_and(|group| group.collapsed) {
                continue;
            }
            ui.push_id(("regular-row", tab.id), |ui| {
                ui.horizontal(|ui| {
                    let row_rect = Rect::from_min_size(
                        ui.available_rect_before_wrap().min,
                        Vec2::new(ui.available_width(), 32.0),
                    );
                    if let Some(group) = group {
                        ui.painter().rect_filled(
                            Rect::from_min_size(row_rect.min, Vec2::new(2.0, row_rect.height())),
                            1.0,
                            group.color,
                        );
                        ui.add_space(3.0);
                    }
                    let close_id = ui.make_persistent_id("close_tab");
                    let show_close = expanded
                        && !DragAndDrop::has_payload_of_type::<PortalTabDrag>(ui.ctx())
                        && (active
                            || ui.rect_contains_pointer(row_rect)
                            || ui.memory(|memory| memory.has_focus(close_id)));
                    let close_width = if show_close {
                        26.0 + ui.spacing().item_spacing.x
                    } else {
                        0.0
                    };
                    let size = Vec2::new(
                        (ui.available_width() - close_width).max(0.0),
                        row_rect.height(),
                    );
                    self.show_tab_button(ui, tab, active, expanded, size, action);
                    if show_close {
                        let title = self.tab_rail_title(tab);
                        if tab_close_button(ui, close_id, title.as_ref(), row_rect.height()) {
                            *action = Some(PortalTabAction::Close(tab.id));
                        }
                    }
                });
            });
        }
    }

    fn show_tab_button(
        &self,
        ui: &mut Ui,
        tab: &PortalTab,
        active: bool,
        labels_visible: bool,
        size: Vec2,
        action: &mut Option<PortalTabAction>,
    ) {
        ui.push_id(("tab", tab.id), |ui| {
            let title = self.tab_rail_title(tab);
            let selected = self.tabs.is_selected(tab.id);
            let icon = Image::new(portal_target_icon_in(
                self.azure_content_for_tab(tab.id),
                tab.target(),
            ))
            .fit_to_exact_size(Vec2::splat(16.0));
            let mut button = if labels_visible {
                let title = RichText::new(title.as_ref());
                let title = if tab.preview { title.italics() } else { title };
                Button::selectable(active || selected, (icon, title, Atom::grow()))
                    .gap(8.0)
                    .truncate()
            } else {
                Button::selectable(active || selected, icon)
            };
            if selected && !active {
                button = button.fill(ui.visuals().selection.bg_fill.gamma_multiply(0.5));
            }
            let is_dragging = DragAndDrop::payload::<PortalTabDrag>(ui.ctx()).is_some_and(|drag| {
                drag.tab_id() == Some(tab.id)
                    || matches!(*drag, PortalTabDrag::Tabs(anchor) if selected && self.tabs.is_selected(anchor))
            });
            let response = ui
                .scope(|ui| {
                    if is_dragging {
                        ui.set_opacity(0.0);
                    }
                    ui.add_sized(size, button.min_size(size).sense(Sense::click_and_drag()))
                })
                .inner;
            let response = if is_dragging {
                response
            } else {
                response.on_hover_text(title.as_ref())
            };
            if !labels_visible {
                response.widget_info(|| {
                    WidgetInfo::selected(
                        WidgetType::Button,
                        response.enabled(),
                        active || selected,
                        title.as_ref(),
                    )
                });
            }
            if response.drag_started() {
                let drag = if self.tabs.dragged_tabs(tab.id).len() > 1 {
                    PortalTabDrag::Tabs(tab.id)
                } else if tab.pinned {
                    PortalTabDrag::PinnedTab(tab.id)
                } else {
                    PortalTabDrag::Tab(tab.id)
                };
                response.dnd_set_drag_payload(drag);
            }
            if response.dragged() {
                ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
            }
            if let Some(drop) = PortalTabDrag::drop_on_tab(ui, &response, tab, &self.tabs) {
                *action = Some(drop);
            }
            if response.clicked() {
                let modifiers = ui.input(|input| input.modifiers);
                *action = Some(if tab.preview && response.double_clicked() && !modifiers.any() {
                    PortalTabAction::KeepOpen(tab.id)
                } else {
                    PortalTabAction::Select(tab.id, modifiers)
                });
            }
            if response.middle_clicked() {
                *action = Some(PortalTabAction::Close(tab.id));
            }
            response.context_menu(|ui| self.show_portal_tab_menu(ui, tab, action));
        });
    }

    pub(crate) fn show_portal_tab_drag_preview(&self, ui: &mut Ui) {
        let Some(drag) = DragAndDrop::payload::<PortalTabDrag>(ui.ctx()) else {
            return;
        };
        let dragging = drag.tabs(&self.tabs);
        let Some(tab) = drag
            .tab_id()
            .or_else(|| dragging.first().copied())
            .and_then(|id| self.tabs.tab(id))
        else {
            return;
        };
        let Some(pointer) = ui.input(|input| input.pointer.interact_pos()) else {
            return;
        };
        Area::new(Id::new(("portal-tab-drag-preview", tab.id)))
            .order(Order::Tooltip)
            .current_pos(pointer + Vec2::new(12.0, 10.0))
            .interactable(false)
            .fade_in(false)
            .show(ui, |ui| {
                Frame::popup(ui.style()).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.add(
                            Image::new(portal_target_icon_in(
                                self.azure_content_for_tab(tab.id),
                                tab.target(),
                            ))
                            .fit_to_exact_size(Vec2::splat(16.0)),
                        );
                        let title = RichText::new(self.tab_rail_title(tab));
                        ui.label(if tab.preview { title.italics() } else { title });
                        if dragging.len() > 1 {
                            ui.label(format!("{} tabs", dragging.len()));
                        }
                    });
                });
            });
    }

    fn show_new_tab_button(
        &self,
        ui: &mut Ui,
        pane: PortalPaneId,
        expanded: bool,
        action: &mut Option<PortalTabAction>,
    ) {
        let shortcut = ui.ctx().format_shortcut(&NEW_TAB_SHORTCUT);
        let button = if expanded {
            Button::new(("+", "New tab")).shortcut_text(shortcut.as_str())
        } else {
            Button::new("+")
        };
        let response = ui
            .add_sized(
                [ui.available_width(), 32.0],
                button.frame_when_inactive(false),
            )
            .on_hover_text(format!("New tab ({shortcut})"));
        if !expanded {
            response.widget_info(|| {
                WidgetInfo::labeled(WidgetType::Button, response.enabled(), "New tab")
            });
        }
        if response.clicked() {
            *action = Some(PortalTabAction::OpenInPane(pane));
            ui.scroll_to_cursor(Some(Align::Max));
        }
        response.context_menu(|ui| self.show_reopen_tab_item(ui, action));
    }

    pub(crate) fn portal_tab_title<'a>(&'a self, target: &'a PortalTabTarget) -> Cow<'a, str> {
        self.portal_tab_title_in(&self.azure, target)
    }

    fn tab_rail_title<'a>(&'a self, tab: &'a PortalTab) -> Cow<'a, str> {
        let target = tab.target();
        self.azure_content_for_tab(tab.id).map_or_else(
            || target.title(),
            |azure| self.portal_tab_title_in(azure, target),
        )
    }

    fn portal_tab_title_in<'a>(
        &'a self,
        azure: &'a PortalAzureContent,
        target: &'a PortalTabTarget,
    ) -> Cow<'a, str> {
        if let PortalTabTarget::AzureDevOpsProject {
            organization,
            project,
            view,
        } = target
            && let Some(name) = self.azure_devops_project_name_in(azure, organization, *project)
            && !name.trim().is_empty()
            && (!self.egui_config.demo_mode || !name.contains('@'))
        {
            return if *view == PortalAzureDevOpsProjectView::Overview {
                Cow::Borrowed(name)
            } else {
                Cow::Owned(format!("{name} · {}", view.label()))
            };
        }
        let directory_name = match target {
            PortalTabTarget::EntraUser(id) => azure.entra_directory.user_name(*id),
            PortalTabTarget::EntraGroup(id) => azure.entra_directory.group_name(*id),
            PortalTabTarget::AzureTenant(id) => self.tenant_manager.tenant_name(*id),
            PortalTabTarget::EntraAppRegistration(id) => {
                azure.entra_directory.app_registration_name(*id)
            }
            _ => None,
        };
        if let Some(name) = directory_name.filter(|name| {
            !name.trim().is_empty() && (!self.egui_config.demo_mode || !name.contains('@'))
        }) {
            return Cow::Borrowed(name);
        }
        if let PortalTabTarget::ResourceOutput { resource, format } = target {
            return Cow::Owned(format!(
                "{} · {}",
                self.portal_tab_title_in(azure, &resource.overview()),
                format_label(*format)
            ));
        }
        if let PortalTabTarget::ScopeView { scope, view } = target {
            return Cow::Owned(format!(
                "{} · {}",
                self.portal_tab_title_in(azure, &scope.overview()),
                view.label()
            ));
        }
        if let PortalTabTarget::ManagementGroup(id) = target
            && let Some(title) = azure.management_group_title(id)
        {
            return Cow::Borrowed(title);
        }
        if let PortalTabTarget::Resource(id) = target
            && let Some(resource) = azure.inventory_resource(id)
        {
            return Cow::Borrowed(resource.display_name().unwrap_or(&resource.name));
        }
        if let PortalTabTarget::Subscription(id) = target {
            let name = azure
                .subscriptions
                .as_loaded()
                .and_then(|subscriptions| {
                    subscriptions
                        .iter()
                        .find(|subscription| subscription.id == *id)
                        .map(|subscription| subscription.name.as_str())
                })
                .or_else(|| {
                    azure
                        .resource_groups
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

fn tab_close_button(ui: &mut Ui, id: Id, title: &str, height: f32) -> bool {
    let (_, rect) = ui.allocate_space(Vec2::new(26.0, height));
    let label = format!("Close {title}");
    let response = ui.interact(rect, id, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, response.enabled(), &label));
    let visuals = ui.style().interact(&response);
    if response.hovered() || response.has_focus() {
        ui.painter()
            .rect_filled(rect, visuals.corner_radius, visuals.weak_bg_fill);
    }
    let center = rect.center();
    for direction in [-1.0, 1.0] {
        let arm = Vec2::new(3.5, direction * 3.5);
        ui.painter()
            .line_segment([center - arm, center + arm], visuals.fg_stroke);
    }
    let response = response.on_hover_text(label);
    response.clicked() || response.middle_clicked()
}
