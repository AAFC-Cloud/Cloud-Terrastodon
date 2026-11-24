use crate::app::MyApp;
use crate::portal_page::PortalPage;
use crate::portal_pane_id::PortalPaneId;
use crate::portal_split_side::PortalSplitSide;
use crate::portal_tab_drag::PortalTabDrag;
use crate::portal_tab_group_id::PortalTabGroupId;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;
use eframe::egui::Color32;
use eframe::egui::Modifiers;
use eframe::egui::Rect;
use eframe::egui::Ui;

/// Sidebar actions are applied after rendering the current tab order.
pub(crate) enum PortalTabAction {
    Open,
    OpenInPane(PortalPaneId),
    OpenBelow(PortalTabId),
    OpenInGroup(PortalTabGroupId),
    Select(PortalTabId, Modifiers),
    KeepOpen(PortalTabId),
    ExitSplit(PortalTabId),
    Split(PortalTabId, PortalSplitSide),
    Refresh(PortalTabId),
    Duplicate(PortalTabId),
    Close(PortalTabId),
    CloseOthers(PortalTabId),
    CloseBelow(PortalTabId),
    CloseGroup(PortalTabGroupId),
    Reopen,
    SetPinned(PortalTabId, bool),
    MoveToTop(PortalTabId),
    MoveToBottom(PortalTabId),
    MoveBefore(PortalTabId, PortalTabId),
    MoveAfter(PortalTabId, PortalTabId),
    MoveOutsideGroups(PortalTabId),
    MoveToPane(PortalTabDrag, PortalPaneId),
    MoveSelectedBeside {
        anchor: PortalTabId,
        target: PortalTabId,
        before: bool,
    },
    MoveSelectedToEnd(PortalTabId),
    MoveGroup(PortalTabGroupId, PortalTabId, bool),
    MoveGroupToEnd(PortalTabGroupId),
    GroupWith(PortalTabId, PortalTabId),
    CreateGroup(PortalTabId),
    MoveToGroup(PortalTabId, PortalTabGroupId),
    RemoveFromGroup(PortalTabId),
    Ungroup(PortalTabGroupId),
    ToggleGroup(PortalTabGroupId),
    RenameGroup(PortalTabGroupId, String),
    SetGroupColor(PortalTabGroupId, Color32),
}

impl MyApp {
    pub(crate) fn apply_portal_tab_action(&mut self, ui: &mut Ui, action: PortalTabAction) {
        self.sync_azure_content();
        let mut closed = Vec::new();
        match action {
            PortalTabAction::Open => {
                self.tabs.open_new_tab();
                self.record_portal_visit();
            }
            PortalTabAction::OpenInPane(pane) => {
                if let Some(index) = self
                    .tabs
                    .pane_active(pane)
                    .and_then(|tab| self.tabs.index_of(tab))
                {
                    self.tabs.select(index);
                    self.sync_azure_content();
                    self.tabs.open_new_tab();
                    self.record_portal_visit();
                }
            }
            PortalTabAction::OpenBelow(tab) => {
                self.tabs.new_tab_below(tab);
                self.seed_azure_content(self.tabs.active_id(), tab);
                self.record_portal_visit();
            }
            PortalTabAction::OpenInGroup(group) => {
                let source = self
                    .tabs
                    .group_pane(group)
                    .and_then(|pane| self.tabs.pane_active(pane))
                    .unwrap_or_else(|| self.tabs.active_id());
                self.tabs.open_in_group(group);
                self.seed_azure_content(self.tabs.active_id(), source);
                self.record_portal_visit();
            }
            PortalTabAction::Select(tab, modifiers) => self.tabs.select_from_rail(tab, modifiers),
            PortalTabAction::KeepOpen(tab) => {
                if let Some(index) = self.tabs.index_of(tab) {
                    self.tabs.keep_open(index);
                }
            }
            PortalTabAction::ExitSplit(tab) => self.tabs.exit_split(tab),
            PortalTabAction::Split(tab, side) => {
                if let Some(pane) = self.tabs.tab(tab).map(|tab| tab.pane) {
                    self.tabs.split_tabs(&[tab], pane, pane, side);
                }
            }
            PortalTabAction::Refresh(tab) => {
                let target = self
                    .tabs
                    .iter()
                    .find(|(_, candidate, _)| candidate.id == tab)
                    .map(|(_, tab, _)| tab.target().clone());
                if let Some(target) = target {
                    let previous = self.tabs.active_id();
                    self.tabs.set_active_for_render(tab);
                    self.sync_azure_content();
                    self.refresh_portal_view(ui, &target);
                    self.tabs.set_active_for_render(previous);
                }
            }
            PortalTabAction::Duplicate(tab) => {
                self.tabs.duplicate(tab);
                self.seed_azure_content(self.tabs.active_id(), tab);
            }
            PortalTabAction::Close(tab) => {
                if let Some(target) = self
                    .tabs
                    .index_of(tab)
                    .and_then(|index| self.tabs.close(index))
                {
                    closed.push(target);
                }
            }
            PortalTabAction::CloseOthers(tab) => closed = self.tabs.close_others(tab),
            PortalTabAction::CloseBelow(tab) => closed = self.tabs.close_below(tab),
            PortalTabAction::CloseGroup(group) => closed = self.tabs.close_group(group),
            PortalTabAction::Reopen => {
                if self.tabs.reopen_closed()
                    && let PortalTabTarget::LocalPath(path) = self.tabs.active()
                {
                    self.egui_config
                        .open_dirs
                        .entry(path.clone())
                        .or_insert(Rect::ZERO);
                }
            }
            PortalTabAction::SetPinned(tab, pinned) => self.tabs.set_pinned(tab, pinned),
            PortalTabAction::MoveToTop(tab) => self.tabs.move_to_edge(tab, true),
            PortalTabAction::MoveToBottom(tab) => self.tabs.move_to_edge(tab, false),
            PortalTabAction::MoveBefore(source, target) => self.tabs.move_tab(source, target, true),
            PortalTabAction::MoveAfter(source, target) => self.tabs.move_tab(source, target, false),
            PortalTabAction::MoveOutsideGroups(tab) => self.tabs.move_to_ungrouped_end(tab),
            PortalTabAction::MoveToPane(drag, pane) => {
                self.tabs.move_tabs_to_pane(&drag.tabs(&self.tabs), pane);
            }
            PortalTabAction::MoveSelectedBeside {
                anchor,
                target,
                before,
            } => {
                let mut selected = self.tabs.dragged_tabs(anchor);
                if !before {
                    selected.reverse();
                }
                for tab in selected {
                    self.tabs.move_tab(tab, target, before);
                }
            }
            PortalTabAction::MoveSelectedToEnd(anchor) => {
                for tab in self.tabs.dragged_tabs(anchor) {
                    self.tabs.move_to_ungrouped_end(tab);
                }
            }
            PortalTabAction::MoveGroup(group, target, before) => {
                self.tabs.move_group(group, target, before);
            }
            PortalTabAction::MoveGroupToEnd(group) => self.tabs.move_group_to_end(group),
            PortalTabAction::GroupWith(source, target) => self.tabs.group_tabs(source, target),
            PortalTabAction::CreateGroup(tab) => self.tabs.create_group_for(tab),
            PortalTabAction::MoveToGroup(tab, group) => self.tabs.move_tab_to_group(tab, group),
            PortalTabAction::RemoveFromGroup(tab) => self.tabs.remove_from_group(tab),
            PortalTabAction::Ungroup(group) => self.tabs.ungroup(group),
            PortalTabAction::ToggleGroup(group) => self.tabs.toggle_group(group),
            PortalTabAction::RenameGroup(group, name) => self.tabs.set_group_name(group, name),
            PortalTabAction::SetGroupColor(group, color) => self.tabs.set_group_color(group, color),
        }
        self.sync_azure_content();
        if !closed.is_empty() {
            for target in closed {
                if self.tabs.contains(&target) {
                    continue;
                }
                match target {
                    PortalTabTarget::LocalPath(path) => {
                        self.egui_config.open_dirs.remove(&path);
                    }
                    PortalTabTarget::Page(PortalPage::Accounts) => {
                        self.account_manager = Default::default();
                    }
                    _ => {}
                }
            }
            self.retain_open_tab_data();
        }
        ui.ctx().request_repaint();
    }
}
