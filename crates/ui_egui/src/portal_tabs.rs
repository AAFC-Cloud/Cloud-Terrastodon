use crate::portal_page::PortalPage;
use crate::portal_pane::PortalPane;
use crate::portal_pane_id::PortalPaneId;
use crate::portal_pane_layout::PortalPaneLayout;
use crate::portal_resource_output_target::PortalResourceOutputTarget;
use crate::portal_resource_scope::PortalResourceScope;
use crate::portal_split_side::PortalSplitSide;
use crate::portal_tab::PortalTab;
use crate::portal_tab_group::PortalTabGroup;
use crate::portal_tab_group_id::PortalTabGroupId;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tab_target::PortalTabTarget;
use cloud_terrastodon_azure::AzureStorageAccountId;
use eframe::egui::Color32;
use eframe::egui::Modifiers;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use std::collections::HashMap;
use std::collections::HashSet;
use std::mem;
use std::ops::Range;
use std::path::Path;

const MAX_CLOSED_TABS: usize = 32;

/// Owns the open tabs and selection. Closing the last tab opens a fresh Home preview.
pub(crate) struct PortalTabs {
    tabs: Vec<PortalTab>,
    active: usize,
    layout: PortalPaneLayout,
    panes: HashMap<PortalPaneId, PortalPane>,
    selected: HashSet<PortalTabId>,
    // Pinned tabs occupy the first positions. Each tab group's members are contiguous.
    groups: Vec<PortalTabGroup>,
    closed: Vec<(usize, PortalTab)>,
    next_tab_id: u64,
    next_pane_id: u64,
    next_group_id: u64,
}

impl Default for PortalTabs {
    fn default() -> Self {
        Self {
            tabs: vec![PortalTab::new(
                PortalTabId(0),
                PortalTabTarget::Page(PortalPage::Home),
                true,
            )],
            active: 0,
            layout: PortalPaneLayout::Pane(PortalPaneId::default()),
            panes: HashMap::from([(PortalPaneId::default(), PortalPane::new(PortalTabId(0)))]),
            selected: HashSet::new(),
            groups: Vec::new(),
            closed: Vec::new(),
            next_tab_id: 1,
            next_pane_id: 1,
            next_group_id: 0,
        }
    }
}

impl PortalTabs {
    pub fn active(&self) -> &PortalTabTarget {
        self.tabs[self.active].target()
    }

    pub fn active_id(&self) -> PortalTabId {
        self.tabs[self.active].id
    }

    /// Find the next or previous tab in the active pane's rail order, wrapping at its ends.
    pub fn adjacent_tab(&self, backwards: bool) -> Option<PortalTabId> {
        let pane = self.active_pane();
        let mut order = (self.active + 1..self.tabs.len()).chain(0..self.active);
        let in_active_pane = |&index: &usize| self.tabs[index].pane == pane;
        let index = if backwards {
            order.rfind(in_active_pane)
        } else {
            order.find(in_active_pane)
        };
        index.map(|index| self.tabs[index].id)
    }

    pub fn index_of(&self, id: PortalTabId) -> Option<usize> {
        self.tabs.iter().position(|tab| tab.id == id)
    }

    pub fn tab(&self, id: PortalTabId) -> Option<&PortalTab> {
        self.tabs.iter().find(|tab| tab.id == id)
    }

    pub fn retains_tab(&self, id: PortalTabId) -> bool {
        self.tab(id).is_some() || self.closed.iter().any(|(_, tab)| tab.id == id)
    }

    pub fn layout(&self) -> &PortalPaneLayout {
        &self.layout
    }

    pub fn is_split(&self) -> bool {
        matches!(self.layout, PortalPaneLayout::Split { .. })
    }

    pub fn active_pane(&self) -> PortalPaneId {
        self.tabs
            .get(self.active)
            .map_or(PortalPaneId::default(), |tab| tab.pane)
    }

    pub fn pane_active(&self, pane: PortalPaneId) -> Option<PortalTabId> {
        self.panes.get(&pane).map(|pane| pane.active)
    }

    pub fn pane_expanded(&self, pane: PortalPaneId) -> bool {
        self.panes.get(&pane).is_some_and(|pane| pane.expanded)
    }

    pub fn set_pane_expanded(&mut self, pane: PortalPaneId, expanded: bool) {
        if let Some(pane) = self.panes.get_mut(&pane) {
            pane.expanded = expanded;
        }
    }

    pub fn iter_in_pane(
        &self,
        pane: PortalPaneId,
    ) -> impl Iterator<Item = (usize, &PortalTab, bool)> {
        let active = self.pane_active(pane);
        self.tabs
            .iter()
            .enumerate()
            .filter(move |(_, tab)| tab.pane == pane)
            .map(move |(index, tab)| (index, tab, Some(tab.id) == active))
    }

    pub fn is_selected(&self, tab: PortalTabId) -> bool {
        self.selected.contains(&tab)
    }

    pub fn select_from_rail(&mut self, tab: PortalTabId, modifiers: Modifiers) {
        let Some(index) = self.index_of(tab) else {
            return;
        };
        let pane = self.tabs[index].pane;
        if modifiers.shift {
            let first = self
                .pane_active(pane)
                .and_then(|active| self.index_of(active))
                .unwrap_or(index);
            self.selected.clear();
            self.selected.extend(
                self.tabs[first.min(index)..=first.max(index)]
                    .iter()
                    .filter(|tab| tab.pane == pane)
                    .map(|tab| tab.id),
            );
        } else if modifiers.command || modifiers.ctrl {
            if !self.selected.remove(&tab) {
                if self.selected.is_empty() && Some(tab) != self.pane_active(pane) {
                    self.selected.extend(self.pane_active(pane));
                }
                self.selected.insert(tab);
            }
        } else {
            self.select(index);
        }
    }

    pub fn dragged_tabs(&self, tab: PortalTabId) -> Vec<PortalTabId> {
        if self.is_selected(tab) {
            self.tabs
                .iter()
                .filter(|tab| self.selected.contains(&tab.id))
                .map(|tab| tab.id)
                .collect()
        } else {
            vec![tab]
        }
    }

    pub fn split_tabs(
        &mut self,
        tabs: &[PortalTabId],
        source: PortalPaneId,
        destination: PortalPaneId,
        side: PortalSplitSide,
    ) {
        let Some(first) = tabs
            .iter()
            .copied()
            .find(|&tab| self.index_of(tab).is_some())
        else {
            return;
        };
        let pane = PortalPane {
            active: first,
            expanded: self.pane_expanded(source),
        };
        let new = PortalPaneId(self.next_pane_id);
        if !self.layout.split(destination, new, side) {
            return;
        }
        self.next_pane_id += 1;
        self.panes.insert(new, pane);
        if !self
            .tabs
            .iter()
            .any(|tab| tab.pane == destination && !tabs.contains(&tab.id))
        {
            let mut home = self.new_tab(PortalTabTarget::Page(PortalPage::Home), true);
            home.pane = destination;
            self.tabs.push(home);
        }
        self.move_tabs_to_pane(tabs, new);
    }

    pub fn move_tabs_to_pane(&mut self, tabs: &[PortalTabId], destination: PortalPaneId) {
        if !self.panes.contains_key(&destination) {
            return;
        }
        let moving = |tab: &PortalTab| tabs.contains(&tab.id) && tab.pane != destination;
        let Some(first) = self.tabs.iter().find(|tab| moving(tab)).map(|tab| tab.id) else {
            return;
        };
        let (mut moved, mut remaining): (Vec<_>, Vec<_>) =
            mem::take(&mut self.tabs).into_iter().partition(moving);
        let partial_groups: HashSet<_> = remaining.iter().filter_map(|tab| tab.group).collect();
        for tab in &mut moved {
            tab.pane = destination;
            tab.group = tab.group.filter(|group| !partial_groups.contains(group));
        }
        let pinned = remaining.iter().take_while(|tab| tab.pinned).count();
        let moved_pinned = moved.iter().take_while(|tab| tab.pinned).count();
        remaining.splice(pinned..pinned, moved.drain(..moved_pinned));
        remaining.append(&mut moved);
        self.tabs = remaining;
        self.remove_singleton_groups();
        self.repair_panes();
        if let Some(index) = self.index_of(first) {
            self.select_index(index);
        }
    }

    pub fn resize_split(&mut self, marker: PortalPaneId, bounds: Rect, pointer: Pos2) {
        if let Some(split) = self.layout.divider_mut(marker) {
            split.resize(bounds, pointer);
        }
    }

    pub fn exit_split(&mut self, focus: PortalTabId) {
        if let Some(index) = self.index_of(focus) {
            let pane = self.tabs[index].pane;
            for tab in &mut self.tabs {
                tab.pane = pane;
            }
            self.repair_panes();
            self.select(index);
        }
    }

    /// Give tab content its own navigation context for the duration of a draw.
    /// This leaves the visible panes and groups in the tab rails alone.
    pub fn set_active_for_render(&mut self, id: PortalTabId) {
        if let Some(index) = self.index_of(id) {
            self.active = index;
        }
    }

    pub fn group(&self, id: PortalTabGroupId) -> Option<&PortalTabGroup> {
        self.groups.iter().find(|group| group.id == id)
    }

    pub fn group_pane(&self, id: PortalTabGroupId) -> Option<PortalPaneId> {
        self.tabs
            .iter()
            .find(|tab| tab.group == Some(id))
            .map(|tab| tab.pane)
    }

    pub fn group_members(&self, id: PortalTabGroupId) -> Vec<PortalTabId> {
        self.tabs
            .iter()
            .filter(|tab| tab.group == Some(id))
            .map(|tab| tab.id)
            .collect()
    }

    pub fn groups(&self) -> impl Iterator<Item = &PortalTabGroup> {
        self.groups.iter()
    }

    pub fn history(&self) -> (&[PortalTabTarget], usize) {
        self.tabs[self.active].history()
    }

    pub fn back(&mut self) -> bool {
        self.history()
            .1
            .checked_sub(1)
            .is_some_and(|position| self.go_to(position))
    }

    pub fn forward(&mut self) -> bool {
        self.go_to(self.history().1 + 1)
    }

    /// Move to a visit in this tab, even if another tab has the same destination.
    pub fn go_to(&mut self, position: usize) -> bool {
        self.tabs[self.active].go_to(position)
    }

    /// Record a visit in the current tab, even when the tab is kept open.
    pub fn navigate_current(&mut self, target: PortalTabTarget) -> bool {
        self.tabs[self.active].navigate(target)
    }

    pub fn replace_current(&mut self, target: PortalTabTarget) {
        self.tabs[self.active].replace_current(target);
    }

    pub fn contains(&self, target: &PortalTabTarget) -> bool {
        self.tabs.iter().any(|tab| tab.target() == target)
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, &PortalTab, bool)> {
        self.tabs
            .iter()
            .enumerate()
            .map(|(index, tab)| (index, tab, index == self.active))
    }

    pub fn has_local_path(&self, path: &Path) -> bool {
        self.tabs
            .iter()
            .any(|tab| matches!(tab.target(), PortalTabTarget::LocalPath(other) if other == path))
    }

    pub fn has_storage_account(&self, id: &AzureStorageAccountId) -> bool {
        self.tabs.iter().any(|tab| match tab.target() {
            PortalTabTarget::StorageAccount(other)
            | PortalTabTarget::ScopeView {
                scope: PortalResourceScope::StorageAccount(other),
                ..
            }
            | PortalTabTarget::ResourceOutput {
                resource:
                    PortalResourceOutputTarget::StorageAccount(other)
                    | PortalResourceOutputTarget::Scope(PortalResourceScope::StorageAccount(other)),
                ..
            } => other == id,
            PortalTabTarget::StorageContainer(container) => &container.storage_account_id == id,
            _ => false,
        })
    }

    /// Start a separate Home preview, even if a Home tab is already open.
    pub fn open_new_tab(&mut self) {
        let tab = self.new_tab(PortalTabTarget::Page(PortalPage::Home), true);
        self.insert_and_select(self.tabs.len(), tab);
    }

    pub fn new_tab_below(&mut self, id: PortalTabId) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let mut tab = self.new_tab(PortalTabTarget::Page(PortalPage::Home), true);
        tab.pane = self.tabs[index].pane;
        tab.group = self.tabs[index].group;
        self.insert_and_select((index + 1).max(self.pinned_count()), tab);
    }

    pub fn duplicate(&mut self, id: PortalTabId) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let mut tab = self.tabs[index].clone();
        tab.id = self.next_tab_id();
        tab.preview = false;
        tab.pinned = false;
        self.insert_and_select((index + 1).max(self.pinned_count()), tab);
    }

    /// Navigate within the current preview or start a new preview from a kept tab.
    pub fn open(&mut self, target: PortalTabTarget) -> bool {
        if self.tabs[self.active].preview {
            return self.navigate_current(target);
        }
        if self.active() == &target {
            return false;
        }
        let mut tab = self.new_tab(self.active().clone(), true);
        tab.navigate(target);
        self.insert_and_select(self.tabs.len(), tab);
        true
    }

    pub fn open_kept(&mut self, target: PortalTabTarget) {
        if let Some(index) = self.find_target(&target) {
            self.tabs[index].preview = false;
            self.select(index);
        } else {
            let tab = self.new_tab(target, false);
            self.insert_and_select(self.tabs.len(), tab);
        }
    }

    /// Open a separate preview tab while preserving the current selection and split.
    pub fn open_background(&mut self, target: PortalTabTarget) -> PortalTabId {
        let tab = self.new_tab(target, true);
        let id = tab.id;
        self.tabs.push(tab);
        id
    }

    pub fn select(&mut self, index: usize) {
        self.selected.clear();
        self.select_index(index);
    }

    fn select_index(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get(index) {
            if let Some(pane) = self.panes.get_mut(&tab.pane) {
                pane.active = tab.id;
            }
            self.active = index;
            if let Some(group) = tab.group.and_then(|id| self.group_mut(id)) {
                group.collapsed = false;
            }
        }
    }

    pub fn keep_open(&mut self, index: usize) {
        if let Some(tab) = self.tabs.get_mut(index) {
            tab.preview = false;
        }
    }

    pub fn close(&mut self, index: usize) -> Option<PortalTabTarget> {
        if index >= self.tabs.len() {
            return None;
        }
        let previous = self.active_id();
        let removed = self.tabs.remove(index);
        let pane = removed.pane;
        let target = removed.target().clone();
        if let Some(pane_state) = self.panes.get_mut(&pane)
            && pane_state.active == removed.id
            && let Some(adjacent) = self.tabs[index..]
                .iter()
                .find(|tab| tab.pane == pane)
                .or_else(|| self.tabs[..index].iter().rev().find(|tab| tab.pane == pane))
        {
            pane_state.active = adjacent.id;
        }
        if self.closed.len() == MAX_CLOSED_TABS {
            self.closed.remove(0);
        }
        self.closed.push((index, removed));
        self.remove_singleton_groups();
        self.repair_panes();
        if self.tabs.is_empty() {
            self.open_new_tab();
        } else if let Some(active) = self.index_of(previous) {
            self.active = active;
        } else {
            let neighboring_pane = self.tabs[index.min(self.tabs.len() - 1)].pane;
            let next = self
                .pane_active(pane)
                .or_else(|| self.pane_active(neighboring_pane))
                .and_then(|tab| self.index_of(tab))
                .unwrap_or_default();
            self.select_index(next);
        }
        Some(target)
    }

    pub fn close_others(&mut self, id: PortalTabId) -> Vec<PortalTabTarget> {
        let Some(pane) = self.tab(id).map(|tab| tab.pane) else {
            return Vec::new();
        };
        self.close_matching(|_, tab| tab.pane == pane && tab.id != id && !tab.pinned)
    }

    pub fn close_below(&mut self, id: PortalTabId) -> Vec<PortalTabTarget> {
        let Some(index) = self.index_of(id) else {
            return Vec::new();
        };
        let pane = self.tabs[index].pane;
        self.close_matching(|position, tab| tab.pane == pane && position > index && !tab.pinned)
    }

    pub fn close_group(&mut self, id: PortalTabGroupId) -> Vec<PortalTabTarget> {
        self.close_matching(|_, tab| tab.group == Some(id))
    }

    pub fn can_reopen_closed(&self) -> bool {
        !self.closed.is_empty()
    }

    pub fn reopen_closed(&mut self) -> bool {
        let Some((position, mut tab)) = self.closed.pop() else {
            return false;
        };
        if !self.panes.contains_key(&tab.pane) {
            tab.pane = self.active_pane();
        }
        tab.group = tab
            .group
            .filter(|&group| self.group_pane(group) == Some(tab.pane));
        let position = if tab.pinned {
            position.min(self.pinned_count())
        } else if let Some(range) = tab.group.and_then(|id| self.group_range(id)) {
            position.clamp(range.start, range.end)
        } else {
            tab.group = None;
            self.standalone_insertion(position.clamp(self.pinned_count(), self.tabs.len()))
        };
        self.insert_and_select(position, tab);
        true
    }

    pub fn set_pinned(&mut self, id: PortalTabId, pinned: bool) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        if self.tabs[index].pinned != pinned {
            let position = self.pinned_count();
            self.tabs[index].pinned = pinned;
            self.tabs[index].preview &= !pinned;
            self.move_tab_at(index, position, None);
        }
    }

    /// Move to the first or last position in the tab's current group or rail section.
    pub fn move_to_edge(&mut self, id: PortalTabId, first: bool) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        let tab = &self.tabs[index];
        let range = if tab.pinned {
            0..self.pinned_count()
        } else {
            tab.group
                .and_then(|id| self.group_range(id))
                .unwrap_or_else(|| self.pinned_count()..self.tabs.len())
        };
        self.move_tab_at(
            index,
            if first { range.start } else { range.end },
            tab.group,
        );
    }

    pub fn move_to_ungrouped_end(&mut self, id: PortalTabId) {
        if let Some(index) = self.index_of(id)
            && !self.tabs[index].pinned
        {
            self.move_tab_at(index, self.tabs.len(), None);
        }
    }

    /// Move a group as a block beside another tab or group.
    pub fn move_group(&mut self, source: PortalTabGroupId, target: PortalTabId, before: bool) {
        let (Some(range), Some(destination)) = (self.group_range(source), self.index_of(target))
        else {
            return;
        };
        let position = if let Some(target_range) = self.tabs[destination]
            .group
            .and_then(|id| self.group_range(id))
        {
            if before {
                target_range.start
            } else {
                target_range.end
            }
        } else {
            (destination + usize::from(!before)).max(self.pinned_count())
        };
        self.move_group_at(range, position);
    }

    pub fn move_group_to_end(&mut self, id: PortalTabGroupId) {
        if let Some(range) = self.group_range(id) {
            self.move_group_at(range, self.tabs.len());
        }
    }

    /// Place a tab by an edge of another tab. A group member stays in its group
    /// when reordered against a sibling. Drops from elsewhere flank the whole group.
    pub fn move_tab(&mut self, source: PortalTabId, target: PortalTabId, before: bool) {
        if source == target {
            return;
        }
        let (Some(index), Some(destination)) = (self.index_of(source), self.index_of(target))
        else {
            return;
        };
        let target = &self.tabs[destination];
        if self.tabs[index].pinned != target.pinned || self.tabs[index].pane != target.pane {
            return;
        }
        let group = target
            .group
            .filter(|&group| self.tabs[index].group == Some(group));
        let position = if group.is_none()
            && let Some(range) = target.group.and_then(|id| self.group_range(id))
        {
            if before { range.start } else { range.end }
        } else {
            destination + usize::from(!before)
        };
        self.move_tab_at(index, position, group);
    }

    /// Place a tab with another tab, creating a group when needed.
    pub fn group_tabs(&mut self, source: PortalTabId, target: PortalTabId) {
        if source == target {
            return;
        }
        let (Some(index), Some(destination)) = (self.index_of(source), self.index_of(target))
        else {
            return;
        };
        if self.tabs[index].pinned
            || self.tabs[destination].pinned
            || self.tabs[index].pane != self.tabs[destination].pane
        {
            return;
        }
        if let Some(group) = self.tabs[destination].group {
            self.move_tab_to_group(source, group);
        } else {
            let group = self.create_group();
            self.tabs[destination].group = Some(group);
            self.move_tab_at(index, destination + 1, Some(group));
        }
    }

    /// Start a group with this tab and a Home tab. Every group starts with two tabs.
    pub fn create_group_for(&mut self, id: PortalTabId) {
        if self
            .index_of(id)
            .is_none_or(|index| self.tabs[index].pinned)
        {
            return;
        }
        self.remove_from_group(id);
        let Some(index) = self.index_of(id) else {
            return;
        };
        let group = self.create_group();
        self.tabs[index].group = Some(group);
        self.open_in_group(group);
        self.select(index);
    }

    pub fn open_in_group(&mut self, id: PortalTabGroupId) {
        let Some(range) = self.group_range(id) else {
            return;
        };
        let mut tab = self.new_tab(PortalTabTarget::Page(PortalPage::Home), true);
        tab.pane = self.tabs[range.start].pane;
        tab.group = Some(id);
        self.insert_and_select(range.end, tab);
    }

    pub fn move_tab_to_group(&mut self, tab: PortalTabId, group: PortalTabGroupId) {
        let (Some(index), Some(range)) = (self.index_of(tab), self.group_range(group)) else {
            return;
        };
        if !self.tabs[index].pinned
            && self.tabs[index].group != Some(group)
            && self.tabs[index].pane == self.tabs[range.start].pane
        {
            self.move_tab_at(index, range.end, Some(group));
        }
    }

    pub fn remove_from_group(&mut self, id: PortalTabId) {
        let Some(index) = self.index_of(id) else {
            return;
        };
        if let Some(range) = self.tabs[index]
            .group
            .and_then(|group| self.group_range(group))
        {
            self.move_tab_at(index, range.end, None);
        }
    }

    pub fn ungroup(&mut self, id: PortalTabGroupId) {
        if let Some(range) = self.group_range(id) {
            for tab in &mut self.tabs[range] {
                tab.group = None;
            }
            self.groups.retain(|group| group.id != id);
        }
    }

    pub fn toggle_group(&mut self, id: PortalTabGroupId) {
        if let Some(group) = self.group_mut(id) {
            group.collapsed = !group.collapsed;
        }
    }

    pub fn set_group_name(&mut self, id: PortalTabGroupId, name: String) {
        if let Some(group) = self.group_mut(id) {
            group.name = name;
        }
    }

    pub fn set_group_color(&mut self, id: PortalTabGroupId, color: Color32) {
        if let Some(group) = self.group_mut(id) {
            group.color = color;
        }
    }

    fn find_target(&self, target: &PortalTabTarget) -> Option<usize> {
        if self.active() == target {
            Some(self.active)
        } else {
            self.tabs.iter().position(|tab| tab.target() == target)
        }
    }

    fn close_matching(
        &mut self,
        matches: impl Fn(usize, &PortalTab) -> bool,
    ) -> Vec<PortalTabTarget> {
        let indices: Vec<_> = self
            .tabs
            .iter()
            .enumerate()
            .rev()
            .filter_map(|(index, tab)| matches(index, tab).then_some(index))
            .collect();
        indices
            .into_iter()
            .filter_map(|index| self.close(index))
            .collect()
    }

    fn insert_and_select(&mut self, index: usize, tab: PortalTab) {
        self.panes
            .entry(tab.pane)
            .or_insert_with(|| PortalPane::new(tab.id));
        self.tabs.insert(index, tab);
        self.select(index);
    }

    fn next_tab_id(&mut self) -> PortalTabId {
        let id = PortalTabId(self.next_tab_id);
        self.next_tab_id += 1;
        id
    }

    fn new_tab(&mut self, target: PortalTabTarget, preview: bool) -> PortalTab {
        let mut tab = PortalTab::new(self.next_tab_id(), target, preview);
        tab.pane = self.active_pane();
        tab
    }

    fn create_group(&mut self) -> PortalTabGroupId {
        let id = PortalTabGroupId(self.next_group_id);
        self.next_group_id += 1;
        self.groups.push(PortalTabGroup::new(id));
        id
    }

    fn group_mut(&mut self, id: PortalTabGroupId) -> Option<&mut PortalTabGroup> {
        self.groups.iter_mut().find(|group| group.id == id)
    }

    fn group_range(&self, id: PortalTabGroupId) -> Option<Range<usize>> {
        let start = self.tabs.iter().position(|tab| tab.group == Some(id))?;
        let end = self.tabs.iter().rposition(|tab| tab.group == Some(id))? + 1;
        Some(start..end)
    }

    fn pinned_count(&self) -> usize {
        self.tabs.iter().take_while(|tab| tab.pinned).count()
    }

    fn standalone_insertion(&self, position: usize) -> usize {
        self.tabs
            .get(position)
            .and_then(|tab| tab.group)
            .and_then(|group| self.group_range(group))
            .filter(|range| position > range.start)
            .map_or(position, |range| range.end)
    }

    fn move_tab_at(&mut self, index: usize, position: usize, group: Option<PortalTabGroupId>) {
        let active = self.active_id();
        let mut tab = self.tabs.remove(index);
        tab.group = group;
        self.tabs
            .insert(position - usize::from(index < position), tab);
        if let Some(index) = self.index_of(active) {
            self.active = index;
        }
        self.remove_singleton_groups();
    }

    fn move_group_at(&mut self, range: Range<usize>, position: usize) {
        let active = self.active_id();
        if position < range.start {
            self.tabs[position..range.end].rotate_right(range.len());
        } else if position > range.end {
            self.tabs[range.start..position].rotate_left(range.len());
        }
        if let Some(index) = self.index_of(active) {
            self.active = index;
        }
    }

    fn remove_singleton_groups(&mut self) {
        self.groups.retain(|group| {
            self.tabs
                .iter()
                .filter(|tab| tab.group == Some(group.id))
                .nth(1)
                .is_some()
        });
        for tab in &mut self.tabs {
            if tab
                .group
                .is_some_and(|id| !self.groups.iter().any(|group| group.id == id))
            {
                tab.group = None;
            }
        }
    }

    fn repair_panes(&mut self) {
        self.selected
            .retain(|id| self.tabs.iter().any(|tab| tab.id == *id));
        self.panes.retain(|&id, pane| {
            let mut tabs = self.tabs.iter().filter(|tab| tab.pane == id);
            let Some(first) = tabs.next() else {
                return false;
            };
            if first.id != pane.active && !tabs.any(|tab| tab.id == pane.active) {
                pane.active = first.id;
            }
            true
        });
        let layout = mem::replace(
            &mut self.layout,
            PortalPaneLayout::Pane(PortalPaneId::default()),
        );
        if let Some(layout) = layout.retain(|id| self.panes.contains_key(&id)) {
            self.layout = layout;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_resource_scope::PortalResourceScope;
    use crate::portal_scope_view::PortalScopeView;
    use cloud_terrastodon_app::OutputFormat;
    use cloud_terrastodon_azure::AzureResourceGroupId;

    #[test]
    fn tab_cycling_wraps_in_the_active_panes_rail_order() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let left = tabs.active_pane();
        let logs = tabs.open_background(PortalTabTarget::Page(PortalPage::Logs));
        let resources = tabs.open_background(PortalTabTarget::Page(PortalPage::Resources));
        let about = tabs.open_background(PortalTabTarget::Page(PortalPage::About));
        tabs.split_tabs(&[logs], left, left, PortalSplitSide::Right);
        let right = tabs.active_pane();
        assert_eq!(tabs.adjacent_tab(false), None);
        assert_eq!(tabs.adjacent_tab(true), None);

        tabs.select(tabs.index_of(home).unwrap());
        assert_eq!(tabs.adjacent_tab(false), Some(resources));
        assert_eq!(tabs.adjacent_tab(true), Some(about));

        tabs.select(tabs.index_of(about).unwrap());
        assert_eq!(tabs.adjacent_tab(false), Some(home));
        assert_eq!(tabs.adjacent_tab(true), Some(resources));
        assert_eq!(tabs.pane_active(right), Some(logs));
    }

    #[test]
    fn background_open_keeps_the_active_split_and_creates_a_preview() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let logs = PortalTabTarget::Page(PortalPage::Logs);
        tabs.open_kept(logs.clone());
        let active = tabs.active_id();
        let left = tabs.active_pane();
        tabs.split_tabs(&[active], left, left, PortalSplitSide::Right);
        let right = tabs.active_pane();
        let (visits, cursor) = tabs.history();
        let visits = visits.to_vec();

        tabs.open_background(logs.clone());

        assert_eq!(tabs.active_id(), active);
        assert_eq!(tabs.pane_active(left), Some(home));
        assert_eq!(tabs.pane_active(right), Some(active));
        assert_eq!(tabs.history(), (visits.as_slice(), cursor));
        let (_, new_tab, selected) = tabs.iter().last().unwrap();
        assert_eq!(new_tab.target(), &logs);
        assert_eq!(new_tab.pane, right);
        assert_ne!(new_tab.id, active);
        assert!(new_tab.preview);
        assert!(!selected);
    }

    #[test]
    fn split_selection_and_close_preserve_the_other_pane_and_history() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.navigate_current(PortalTabTarget::Page(PortalPage::Resources));
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();
        let left = tabs.active_pane();
        tabs.split_tabs(&[logs], left, left, PortalSplitSide::Right);
        let right = tabs.active_pane();
        assert_eq!(tabs.pane_active(left), Some(home));
        assert_eq!(tabs.pane_active(right), Some(logs));

        tabs.select(tabs.index_of(home).unwrap());
        assert!(tabs.back());
        assert_eq!(tabs.active(), &PortalTabTarget::Page(PortalPage::Home));
        assert_eq!(tabs.pane_active(left), Some(home));
        assert_eq!(tabs.pane_active(right), Some(logs));

        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let about = tabs.active_id();
        assert_eq!(tabs.pane_active(left), Some(about));
        assert_eq!(tabs.pane_active(right), Some(logs));
        tabs.move_to_edge(about, true);
        assert_eq!(tabs.pane_active(left), Some(about));
        assert_eq!(tabs.pane_active(right), Some(logs));

        tabs.close(tabs.index_of(about).unwrap());
        assert!(tabs.is_split());
        assert_eq!(tabs.pane_active(left), Some(home));
        tabs.close(tabs.index_of(home).unwrap());
        assert!(!tabs.is_split());
        assert_eq!(tabs.active_id(), logs);
    }

    #[test]
    fn selected_tabs_can_move_to_new_and_existing_panes_with_independent_rails() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let left = tabs.active_pane();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Resources));
        let resources = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();
        tabs.select_from_rail(resources, Modifiers::SHIFT);
        let moving = tabs.dragged_tabs(resources);
        assert_eq!(moving, [resources, logs]);
        tabs.split_tabs(&moving, left, left, PortalSplitSide::Right);
        let right = tabs.active_pane();
        assert!(tabs.pane_expanded(right));
        tabs.set_pane_expanded(right, false);
        assert!(tabs.pane_expanded(left));
        assert!(!tabs.pane_expanded(right));
        assert_eq!(tabs.pane_active(left), Some(home));
        assert_eq!(
            tabs.iter_in_pane(right)
                .map(|(_, tab, _)| tab.id)
                .collect::<Vec<_>>(),
            moving
        );

        tabs.split_tabs(&[logs], right, right, PortalSplitSide::Bottom);
        let below = tabs.active_pane();
        assert!(!tabs.pane_expanded(below));
        tabs.set_pane_expanded(below, true);
        assert!(!tabs.pane_expanded(right));
        assert_eq!(tabs.pane_active(left), Some(home));
        assert_eq!(tabs.pane_active(right), Some(resources));
        assert_eq!(tabs.pane_active(below), Some(logs));
        tabs.move_tabs_to_pane(&moving, left);
        assert!(!tabs.is_split());
        assert_eq!(tabs.active_pane(), left);
        assert_eq!(tabs.dragged_tabs(resources), moving);
    }

    #[test]
    fn moving_a_middle_group_member_preserves_the_remaining_group() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let original = tabs.active_pane();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Resources));
        let resources = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let about = tabs.active_id();
        tabs.group_tabs(resources, home);
        let group = tabs.tab(home).unwrap().group.unwrap();
        tabs.move_tab_to_group(about, group);

        tabs.split_tabs(&[resources], original, original, PortalSplitSide::Right);
        assert_eq!(tabs.group_members(group), [home, about]);
        assert_eq!(tabs.group_range(group).unwrap().len(), 2);
        assert!(tabs.tab(resources).unwrap().group.is_none());
        assert_eq!(tabs.close_group(group).len(), 2);
        assert_eq!(tabs.active_id(), resources);
        assert!(!tabs.is_split());
    }

    #[test]
    fn grouping_and_reordering_preserve_the_selected_tab_and_dissolve_singletons() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Resources));
        let resources = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();

        tabs.group_tabs(home, logs);
        let group = tabs.tabs[1].group.unwrap();
        assert_eq!(tabs.tabs[2].id, home);
        assert_eq!(tabs.tabs[2].group, Some(group));
        assert_eq!(tabs.active_id(), logs);

        tabs.move_tab_to_group(resources, group);
        tabs.move_tab(resources, logs, true);
        assert_eq!(tabs.tabs[0].id, resources);
        assert_eq!(tabs.tabs[1].id, logs);
        assert_eq!(tabs.tabs[0].group, Some(group));

        tabs.move_to_ungrouped_end(logs);
        assert_eq!(tabs.tabs[2].group, None);
        assert_eq!(tabs.active_id(), logs);
        tabs.move_tab(home, logs, false);
        assert!(tabs.group(group).is_none());
        assert_eq!(tabs.tabs[0].id, resources);
        assert_eq!(tabs.tabs[0].group, None);
    }

    #[test]
    fn moving_a_group_keeps_both_groups_contiguous_and_after_pins() {
        let mut tabs = PortalTabs::default();
        let pinned = tabs.active_id();
        tabs.set_pinned(pinned, true);
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Resources));
        let resources = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::ResourceGroups));
        let resource_groups = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Subscriptions));
        let subscriptions = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::ManagementGroups));
        let management_groups = tabs.active_id();
        tabs.group_tabs(resource_groups, resources);
        tabs.group_tabs(management_groups, subscriptions);
        let management = tabs.tabs[3].group.unwrap();

        tabs.move_group(management, resource_groups, true);
        assert_eq!(
            tabs.tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [
                pinned,
                subscriptions,
                management_groups,
                resources,
                resource_groups
            ]
        );
        assert_eq!(tabs.active_id(), management_groups);

        tabs.move_group_to_end(management);
        assert_eq!(
            tabs.tabs.iter().map(|tab| tab.id).collect::<Vec<_>>(),
            [
                pinned,
                resources,
                resource_groups,
                subscriptions,
                management_groups
            ]
        );
        tabs.move_group(management, pinned, true);
        assert_eq!(tabs.tabs[0].id, pinned);
        assert_eq!(tabs.tabs[1].id, subscriptions);
    }

    #[test]
    fn pinning_keeps_the_tab_open_and_moves_it_out_of_a_group() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();
        tabs.group_tabs(home, logs);
        tabs.select(1);
        tabs.set_pinned(home, true);
        assert_eq!(tabs.tabs[0].id, home);
        assert_eq!(tabs.active_id(), home);
        assert!(tabs.tabs[0].pinned);
        assert!(tabs.groups().next().is_none());

        tabs.open(PortalTabTarget::Page(PortalPage::About));
        assert_eq!(
            tabs.tabs[0].target(),
            &PortalTabTarget::Page(PortalPage::Home)
        );
        assert_eq!(tabs.tabs.len(), 3);
    }

    #[test]
    fn reopening_after_batch_close_restores_tab_ids_and_back_history() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.set_pinned(home, true);
        let logs = PortalTabTarget::Page(PortalPage::Logs);
        let resources = PortalTabTarget::Page(PortalPage::Resources);
        let subscriptions = PortalTabTarget::Page(PortalPage::Subscriptions);
        tabs.open_kept(logs.clone());
        let logs_id = tabs.active_id();
        tabs.open(resources.clone());
        let resources_id = tabs.active_id();
        tabs.open_kept(subscriptions.clone());
        let subscriptions_id = tabs.active_id();

        assert_eq!(
            tabs.close_below(logs_id),
            [subscriptions, resources.clone()]
        );
        assert!(tabs.reopen_closed());
        assert_eq!(tabs.active_id(), resources_id);
        assert!(tabs.back());
        assert_eq!(tabs.active(), &logs);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &resources);
        assert!(tabs.reopen_closed());
        assert_eq!(tabs.active_id(), subscriptions_id);
        assert_eq!(tabs.close_others(subscriptions_id), [resources, logs]);
        assert_eq!(tabs.tabs[0].id, home);
    }

    #[test]
    fn two_home_tabs_can_navigate_to_subscriptions_independently() {
        let mut tabs = PortalTabs::default();
        let home = PortalTabTarget::Page(PortalPage::Home);
        let subscriptions = PortalTabTarget::Page(PortalPage::Subscriptions);
        let first_tab = tabs.active_id();
        assert!(tabs.open(subscriptions.clone()));

        tabs.open_new_tab();
        let second_tab = tabs.active_id();
        assert!(tabs.open(subscriptions.clone()));

        assert_eq!(tabs.tabs.len(), 2);
        assert_eq!(tabs.active_id(), second_tab);
        assert_eq!(
            tabs.history(),
            (&[home.clone(), subscriptions.clone()][..], 1)
        );
        assert!(tabs.back());
        assert_eq!(tabs.active(), &home);
        assert_eq!(
            tabs.tab(first_tab).map(PortalTab::target),
            Some(&subscriptions)
        );
    }

    #[test]
    fn navigation_replaces_preview_and_leaves_kept_tabs_open() {
        use PortalPage::ResourceGroups;
        use PortalPage::Resources;
        use PortalPage::Subscriptions;
        use PortalTabTarget::Page;

        let mut tabs = PortalTabs::default();
        tabs.open(Page(Resources));
        tabs.open(Page(Subscriptions));
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &Page(Subscriptions));

        tabs.keep_open(0);
        assert!(!tabs.open(Page(Subscriptions)));
        tabs.open(Page(ResourceGroups));
        assert_eq!(tabs.tabs.len(), 2);
        assert!(tabs.tabs[1].preview);

        tabs.open(Page(Subscriptions));
        assert_eq!(tabs.active, 1);
        assert_eq!(
            tabs.history(),
            (
                &[
                    Page(Subscriptions),
                    Page(ResourceGroups),
                    Page(Subscriptions)
                ][..],
                2
            )
        );
        tabs.select(0);
        tabs.open(Page(Resources));
        assert_eq!(tabs.tabs.len(), 3);
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(Subscriptions)));
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(Subscriptions)));
        assert_eq!(tabs.active(), &Page(Resources));
        assert_eq!(tabs.close(0), Some(Page(Resources)));
        assert_eq!(tabs.active(), &Page(PortalPage::Home));
    }

    #[test]
    fn home_can_close_and_is_recreated_when_the_last_tab_closes() {
        let mut tabs = PortalTabs::default();
        let home = PortalTabTarget::Page(PortalPage::Home);
        let logs = PortalTabTarget::Page(PortalPage::Logs);
        assert!(tabs.tabs[0].preview);

        tabs.open_kept(logs.clone());
        assert_eq!(tabs.close(0), Some(home.clone()));
        assert_eq!(tabs.active(), &logs);
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.close(0), Some(logs));
        assert!(tabs.tabs[0].preview);
        assert_eq!(tabs.history(), (std::slice::from_ref(&home), 0));
        assert_eq!(tabs.close(0), Some(home));
        assert!(!tabs.back());
    }

    #[test]
    fn new_tab_opens_home_with_its_own_history() {
        let mut tabs = PortalTabs::default();
        let home = PortalTabTarget::Page(PortalPage::Home);
        let resources = PortalTabTarget::Page(PortalPage::Resources);
        tabs.open(resources.clone());
        tabs.open_new_tab();
        tabs.open_new_tab();

        assert_eq!(tabs.tabs.len(), 3);
        assert!(tabs.tabs[2].preview);
        assert_eq!(tabs.history(), (std::slice::from_ref(&home), 0));
        tabs.select(0);
        assert_eq!(tabs.history(), (&[home, resources][..], 1));
    }

    #[test]
    fn history_replays_in_the_current_tab_and_new_navigation_drops_forward_visits() {
        use PortalPage::Logs;
        use PortalPage::ResourceGroups;
        use PortalPage::Resources;
        use PortalPage::StorageAccounts;
        use PortalTabTarget::Page;

        let mut tabs = PortalTabs::default();
        tabs.open_kept(Page(Resources));
        tabs.open(Page(StorageAccounts));
        tabs.open(Page(ResourceGroups));
        assert_eq!(
            tabs.history(),
            (
                &[Page(Resources), Page(StorageAccounts), Page(ResourceGroups)][..],
                2
            )
        );

        assert!(tabs.back());
        assert!(tabs.back());
        assert!(!tabs.back());
        assert_eq!(tabs.active, 2);
        assert_eq!(tabs.active(), &Page(Resources));
        assert!(tabs.go_to(2));
        assert_eq!(tabs.active(), &Page(ResourceGroups));
        assert!(tabs.back());
        assert!(tabs.forward());
        assert!(tabs.back());

        tabs.open(Page(Logs));
        assert_eq!(
            tabs.history(),
            (&[Page(Resources), Page(StorageAccounts), Page(Logs)][..], 2)
        );
        assert!(!tabs.forward());
        assert!(!tabs.go_to(4));
    }

    #[test]
    fn switching_scope_blades_replaces_only_the_displayed_visit() -> eyre::Result<()> {
        let scope = PortalResourceScope::ResourceGroup(AzureResourceGroupId::try_new(
            "00000000-0000-0000-0000-000000000001",
            "synthetic-group",
        )?);
        let overview = scope.overview();
        let activity = PortalTabTarget::ScopeView {
            scope: scope.clone(),
            view: PortalScopeView::ActivityLog,
        };
        let access = PortalTabTarget::ScopeView {
            scope,
            view: PortalScopeView::AccessControl,
        };
        let inventory = PortalTabTarget::Page(PortalPage::ResourceGroups);
        let home = PortalTabTarget::Page(PortalPage::Home);
        let mut tabs = PortalTabs::default();
        tabs.open(inventory.clone());
        tabs.open(overview.clone());
        tabs.keep_open(0);

        tabs.replace_current(activity);
        tabs.replace_current(access.clone());
        assert!(tabs.back());
        assert_eq!(tabs.active(), &inventory);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &access);

        let next = PortalTabTarget::Page(PortalPage::Logs);
        tabs.navigate_current(next.clone());
        assert!(tabs.back());
        tabs.replace_current(overview.clone());
        assert_eq!(
            tabs.history(),
            (&[home, inventory, overview, next.clone()][..], 2)
        );
        assert_eq!(tabs.tabs.len(), 1);
        assert!(!tabs.tabs[0].preview);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &next);
        Ok(())
    }

    #[test]
    fn storage_account_output_uses_preview_navigation() -> eyre::Result<()> {
        use PortalTabTarget::Page;
        use PortalTabTarget::ResourceOutput;
        use PortalTabTarget::StorageAccount;

        let id = concat!(
            "/subscriptions/00000000-0000-0000-0000-000000000001",
            "/resourceGroups/synthetic-group",
            "/providers/Microsoft.Storage/storageAccounts/syntheticstore",
        )
        .parse::<AzureStorageAccountId>()?;
        let mut tabs = PortalTabs::default();
        let detail = StorageAccount(id.clone());
        let output = ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Json,
        };

        tabs.open(Page(PortalPage::StorageAccounts));
        tabs.open(detail.clone());
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &detail);
        assert!(tabs.tabs[0].preview);

        tabs.open(output.clone());
        assert_eq!(tabs.active(), &output);
        assert_eq!(tabs.tabs.len(), 1);
        assert!(tabs.tabs[0].preview);
        assert!(tabs.back());
        assert_eq!(tabs.active(), &detail);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &output);

        // The overview breadcrumb replaces the output preview in the same tab.
        tabs.open(detail.clone());
        assert_eq!(tabs.tabs.len(), 1);
        assert_eq!(tabs.active(), &detail);

        tabs.keep_open(0);
        tabs.open(output.clone());
        assert_eq!(tabs.tabs.len(), 2);
        assert!(tabs.tabs[1].preview);
        assert_eq!(tabs.history(), (&[detail.clone(), output.clone()][..], 1));
        tabs.open(detail.clone());
        assert_eq!(tabs.active, 1);
        assert_eq!(tabs.active(), &detail);
        tabs.open(output.clone());
        assert_eq!(tabs.active, 1);
        assert_eq!(tabs.tabs.len(), 2);

        let text_output = ResourceOutput {
            resource: PortalResourceOutputTarget::StorageAccount(id.clone()),
            format: OutputFormat::Text,
        };
        assert!(tabs.navigate_current(text_output.clone()));
        assert!(!tabs.navigate_current(text_output.clone()));
        assert!(tabs.back());
        assert_eq!(tabs.active(), &output);
        assert!(tabs.forward());
        assert_eq!(tabs.active(), &text_output);
        assert!(tabs.back());

        assert_eq!(tabs.close(0), Some(detail));
        assert!(tabs.has_storage_account(&id));
        assert_eq!(tabs.close(0), Some(output));
        assert!(!tabs.has_storage_account(&id));
        Ok(())
    }
}
