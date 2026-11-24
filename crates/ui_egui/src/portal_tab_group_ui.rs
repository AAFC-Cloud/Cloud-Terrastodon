use crate::portal_tab_action::PortalTabAction;
use crate::portal_tab_drag::PortalTabDrag;
use crate::portal_tab_group::PortalTabGroup;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_menu::menu_item;
use crate::portal_tabs::PortalTabs;
use eframe::egui::Atom;
use eframe::egui::Button;
use eframe::egui::Rect;
use eframe::egui::RichText;
use eframe::egui::Sense;
use eframe::egui::TextEdit;
use eframe::egui::Ui;
use eframe::egui::Vec2;
use eframe::egui::WidgetInfo;
use eframe::egui::WidgetType;

pub(crate) fn show_tab_group_header(
    ui: &mut Ui,
    group: &PortalTabGroup,
    first_member: PortalTabId,
    expanded: bool,
    contains_active: bool,
    tabs: &PortalTabs,
    action: &mut Option<PortalTabAction>,
) {
    ui.push_id(group.id, |ui| {
        ui.horizontal(|ui| {
            let controls_width = if expanded {
                30.0 + ui.spacing().item_spacing.x
            } else {
                0.0
            };
            let arrow = if group.collapsed { "▸" } else { "▾" };
            let button = if expanded {
                Button::new((arrow, RichText::new(group.label()).strong(), Atom::grow()))
                    .gap(5.0)
                    .truncate()
            } else {
                Button::new(arrow)
            };
            let response = ui
                .add_sized(
                    [(ui.available_width() - controls_width).max(0.0), 30.0],
                    button.sense(Sense::click_and_drag()).fill(
                        group
                            .color
                            .gamma_multiply(if contains_active { 0.24 } else { 0.13 }),
                    ),
                )
                .on_hover_text(group.label());
            response.widget_info(|| {
                WidgetInfo::selected(
                    WidgetType::Button,
                    response.enabled(),
                    !group.collapsed,
                    group.label(),
                )
            });
            ui.painter().rect_filled(
                Rect::from_min_size(response.rect.min, Vec2::new(2.0, response.rect.height())),
                1.0,
                group.color,
            );
            if response.clicked() {
                *action = Some(PortalTabAction::ToggleGroup(group.id));
            }
            response.dnd_set_drag_payload(PortalTabDrag::Group(group.id));
            if let Some(drop) =
                PortalTabDrag::drop_on_group(ui, &response, group, first_member, tabs)
            {
                *action = Some(drop);
            }
            response.context_menu(|ui| show_group_menu(ui, group, action));
            if expanded {
                ui.menu_button("⋯", |ui| show_group_menu(ui, group, action))
                    .response
                    .on_hover_text("Tab group settings");
            }
        });
    });
}

fn show_group_menu(ui: &mut Ui, group: &PortalTabGroup, action: &mut Option<PortalTabAction>) {
    let mut name = group.name.clone();
    let rename = ui
        .add(
            TextEdit::singleline(&mut name)
                .hint_text("Tab group")
                .desired_width(178.0),
        )
        .on_hover_text("Group name");
    if rename.changed() {
        *action = Some(PortalTabAction::RenameGroup(group.id, name));
    }
    ui.horizontal(|ui| {
        let titles = [
            "Blue", "Purple", "Magenta", "Red", "Orange", "Gold", "Teal", "Slate",
        ];
        for (color, title) in PortalTabGroup::COLORS.into_iter().zip(titles) {
            let swatch = Button::selectable(group.color == color, RichText::new("●").color(color));
            if ui.add(swatch).on_hover_text(title).clicked() {
                *action = Some(PortalTabAction::SetGroupColor(group.id, color));
            }
        }
    });
    ui.separator();
    if menu_item(ui, "New tab in group", None) {
        *action = Some(PortalTabAction::OpenInGroup(group.id));
    }
    if menu_item(ui, "Ungroup tabs", None) {
        *action = Some(PortalTabAction::Ungroup(group.id));
    }
    if menu_item(ui, "Close grouped tabs", None) {
        *action = Some(PortalTabAction::CloseGroup(group.id));
    }
}
