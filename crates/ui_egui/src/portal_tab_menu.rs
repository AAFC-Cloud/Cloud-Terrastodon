use crate::app::MyApp;
use crate::portal_split_side::PortalSplitSide;
use crate::portal_tab::PortalTab;
use crate::portal_tab_action::PortalTabAction;
use eframe::egui::Button;
use eframe::egui::Key;
use eframe::egui::KeyboardShortcut;
use eframe::egui::Modifiers;
use eframe::egui::Ui;

pub(crate) const NEW_TAB_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::T);
pub(crate) const CLOSE_TAB_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::W);
pub(crate) const REOPEN_TAB_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::T);
pub(crate) const DUPLICATE_TAB_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL.plus(Modifiers::SHIFT), Key::K);
pub(crate) const REFRESH_TAB_SHORTCUT: KeyboardShortcut =
    KeyboardShortcut::new(Modifiers::CTRL, Key::R);

impl MyApp {
    pub(crate) fn show_portal_tab_menu(
        &self,
        ui: &mut Ui,
        tab: &PortalTab,
        action: &mut Option<PortalTabAction>,
    ) {
        let id = tab.id;
        if menu_item(ui, "New tab below", None) {
            *action = Some(PortalTabAction::OpenBelow(id));
        }
        if !tab.pinned {
            ui.menu_button("Add tab to group", |ui| {
                if menu_item(ui, "New group with a Home tab", None) {
                    *action = Some(PortalTabAction::CreateGroup(id));
                }
                for group in self.tabs.groups().filter(|group| {
                    Some(group.id) != tab.group && self.tabs.group_pane(group.id) == Some(tab.pane)
                }) {
                    if menu_item(ui, group.label(), None) {
                        *action = Some(PortalTabAction::MoveToGroup(id, group.id));
                    }
                }
            });
        }
        if tab.group.is_some() && menu_item(ui, "Remove from group", None) {
            *action = Some(PortalTabAction::RemoveFromGroup(id));
        }
        ui.separator();
        let can_refresh = self
            .azure_content_for_tab(id)
            .is_some_and(|content| self.can_refresh_portal_view_in(tab.target(), content));
        ui.add_enabled_ui(can_refresh, |ui| {
            if menu_item(ui, "Refresh", Some(REFRESH_TAB_SHORTCUT)) {
                *action = Some(PortalTabAction::Refresh(id));
            }
        });
        if menu_item(ui, "Duplicate tab", Some(DUPLICATE_TAB_SHORTCUT)) {
            *action = Some(PortalTabAction::Duplicate(id));
        }
        ui.menu_button("Move tab to", |ui| {
            if menu_item(ui, "Top", None) {
                *action = Some(PortalTabAction::MoveToTop(id));
            }
            if menu_item(ui, "Bottom", None) {
                *action = Some(PortalTabAction::MoveToBottom(id));
            }
        });
        if menu_item(ui, if tab.pinned { "Unpin tab" } else { "Pin tab" }, None) {
            *action = Some(PortalTabAction::SetPinned(id, !tab.pinned));
        }
        if tab.preview && menu_item(ui, "Keep open", None) {
            *action = Some(PortalTabAction::KeepOpen(id));
        }
        if menu_item(ui, "Split tab right", None) {
            *action = Some(PortalTabAction::Split(id, PortalSplitSide::Right));
        }
        if menu_item(ui, "Split tab below", None) {
            *action = Some(PortalTabAction::Split(id, PortalSplitSide::Bottom));
        }
        if self.tabs.is_split() && menu_item(ui, "Exit split view", None) {
            *action = Some(PortalTabAction::ExitSplit(id));
        }
        ui.separator();
        if menu_item(ui, "Close tab", Some(CLOSE_TAB_SHORTCUT)) {
            *action = Some(PortalTabAction::Close(id));
        }
        ui.add_enabled_ui(
            self.tabs
                .iter_in_pane(tab.pane)
                .any(|(_, other, _)| other.id != id && !other.pinned),
            |ui| {
                if menu_item(ui, "Close other tabs", None) {
                    *action = Some(PortalTabAction::CloseOthers(id));
                }
            },
        );
        ui.add_enabled_ui(
            self.tabs
                .iter_in_pane(tab.pane)
                .skip_while(|(_, other, _)| other.id != id)
                .skip(1)
                .any(|(_, other, _)| !other.pinned),
            |ui| {
                if menu_item(ui, "Close tabs below", None) {
                    *action = Some(PortalTabAction::CloseBelow(id));
                }
            },
        );
        ui.separator();
        self.show_reopen_tab_item(ui, action);
    }

    pub(crate) fn show_reopen_tab_item(&self, ui: &mut Ui, action: &mut Option<PortalTabAction>) {
        ui.add_enabled_ui(self.tabs.can_reopen_closed(), |ui| {
            if menu_item(ui, "Reopen closed tab", Some(REOPEN_TAB_SHORTCUT)) {
                *action = Some(PortalTabAction::Reopen);
            }
        });
    }
}

pub(crate) fn menu_item(ui: &mut Ui, label: &str, shortcut: Option<KeyboardShortcut>) -> bool {
    let mut button = Button::new(label).frame_when_inactive(false);
    if let Some(shortcut) = shortcut {
        button = button.shortcut_text(ui.ctx().format_shortcut(&shortcut));
    }
    if ui.add(button).clicked() {
        ui.close();
        true
    } else {
        false
    }
}
