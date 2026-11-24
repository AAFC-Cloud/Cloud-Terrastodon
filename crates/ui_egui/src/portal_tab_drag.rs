use crate::portal_pane_id::PortalPaneId;
use crate::portal_tab::PortalTab;
use crate::portal_tab_action::PortalTabAction;
use crate::portal_tab_group::PortalTabGroup;
use crate::portal_tab_group_id::PortalTabGroupId;
use crate::portal_tab_id::PortalTabId;
use crate::portal_tabs::PortalTabs;
use eframe::egui::Color32;
use eframe::egui::DragAndDrop;
use eframe::egui::PointerButton;
use eframe::egui::Rect;
use eframe::egui::Response;
use eframe::egui::Stroke;
use eframe::egui::StrokeKind;
use eframe::egui::Ui;

/// The tab, selection, or contiguous group being repositioned in the workspace.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum PortalTabDrag {
    Tab(PortalTabId),
    PinnedTab(PortalTabId),
    Tabs(PortalTabId),
    Group(PortalTabGroupId),
}

impl PortalTabDrag {
    pub fn tab_id(self) -> Option<PortalTabId> {
        match self {
            Self::Tab(tab) | Self::PinnedTab(tab) | Self::Tabs(tab) => Some(tab),
            Self::Group(_) => None,
        }
    }

    pub fn tabs(self, tabs: &PortalTabs) -> Vec<PortalTabId> {
        match self {
            Self::Tab(id) | Self::PinnedTab(id) => vec![id],
            Self::Tabs(anchor) => tabs.dragged_tabs(anchor),
            Self::Group(group) => tabs.group_members(group),
        }
    }

    /// Find the rail where the drag started, even for a selection spanning panes.
    pub fn source_pane(self, tabs: &PortalTabs) -> Option<PortalPaneId> {
        match self {
            Self::Tab(id) | Self::PinnedTab(id) | Self::Tabs(id) => {
                tabs.tab(id).map(|tab| tab.pane)
            }
            Self::Group(group) => tabs.group_pane(group),
        }
    }

    fn transfer(self, tabs: &PortalTabs, target: PortalPaneId) -> Option<PortalTabAction> {
        let crosses_panes = match self {
            Self::Tab(id) | Self::PinnedTab(id) => {
                tabs.tab(id).is_some_and(|tab| tab.pane != target)
            }
            Self::Tabs(anchor) => tabs
                .dragged_tabs(anchor)
                .into_iter()
                .any(|id| tabs.tab(id).is_some_and(|tab| tab.pane != target)),
            Self::Group(group) => tabs.group_pane(group).is_some_and(|pane| pane != target),
        };
        crosses_panes.then_some(PortalTabAction::MoveToPane(self, target))
    }

    pub(crate) fn drop_on_tab(
        ui: &Ui,
        response: &Response,
        target: &PortalTab,
        tabs: &PortalTabs,
    ) -> Option<PortalTabAction> {
        let source = *response.dnd_hover_payload::<Self>()?;
        if let Some(transfer) = source.transfer(tabs, target.pane) {
            return highlight_drop(ui, response, transfer);
        }
        let invalid_target = match source {
            Self::Tab(id) => target.pinned || target.id == id,
            Self::PinnedTab(id) => !target.pinned || target.id == id,
            Self::Tabs(anchor) => tabs.dragged_tabs(anchor).iter().any(|&id| {
                id == target.id || tabs.tab(id).is_none_or(|tab| tab.pinned != target.pinned)
            }),
            Self::Group(id) => target.pinned || target.group == Some(id),
        };
        if invalid_target {
            return None;
        }
        let pointer = ui.input(|input| input.pointer.interact_pos())?;
        let stroke = Stroke::new(2.0, Color32::from_rgb(0, 120, 212));
        let action = if target.pinned {
            let before = pointer.x < response.rect.center().x;
            let x = if before {
                response.rect.left()
            } else {
                response.rect.right()
            };
            ui.painter().vline(x, response.rect.y_range(), stroke);
            Self::move_tab_action(source, target.id, before)
        } else {
            let from_center = pointer.y - response.rect.center().y;
            match source {
                Self::Tab(id) if from_center.abs() < response.rect.height() / 4.0 => {
                    ui.painter()
                        .rect_stroke(response.rect, 3.0, stroke, StrokeKind::Inside);
                    PortalTabAction::GroupWith(id, target.id)
                }
                _ => {
                    let before = from_center < 0.0;
                    show_drop_edge(ui, response, before, stroke);
                    Self::move_tab_action(source, target.id, before)
                }
            }
        };
        response.dnd_release_payload::<Self>().map(|_| action)
    }

    pub(crate) fn drop_on_group(
        ui: &Ui,
        response: &Response,
        target: &PortalTabGroup,
        first_member: PortalTabId,
        tabs: &PortalTabs,
    ) -> Option<PortalTabAction> {
        let source = *response.dnd_hover_payload::<Self>()?;
        if let Some(transfer) = source.transfer(tabs, tabs.group_pane(target.id)?) {
            return highlight_drop(ui, response, transfer);
        }
        if matches!(source, Self::PinnedTab(_) | Self::Tabs(_)) || source == Self::Group(target.id)
        {
            return None;
        }
        let stroke = Stroke::new(2.0, target.color);
        let pointer = ui.input(|input| input.pointer.interact_pos())?;
        let from_center = pointer.y - response.rect.center().y;
        let action = match source {
            Self::Tab(id) if from_center.abs() < response.rect.height() / 4.0 => {
                ui.painter()
                    .rect_stroke(response.rect, 3.0, stroke, StrokeKind::Inside);
                PortalTabAction::MoveToGroup(id, target.id)
            }
            _ => {
                let before = from_center < 0.0;
                show_drop_edge(ui, response, before, stroke);
                Self::move_tab_action(source, first_member, before)
            }
        };
        response.dnd_release_payload::<Self>().map(|_| action)
    }

    fn end_action(self, tabs: &PortalTabs, pane: PortalPaneId) -> Option<PortalTabAction> {
        if let Some(transfer) = self.transfer(tabs, pane) {
            return Some(transfer);
        }
        match self {
            Self::Tab(tab) => Some(PortalTabAction::MoveOutsideGroups(tab)),
            Self::Group(group) => Some(PortalTabAction::MoveGroupToEnd(group)),
            Self::Tabs(anchor)
                if tabs
                    .dragged_tabs(anchor)
                    .iter()
                    .all(|&id| tabs.tab(id).is_some_and(|tab| !tab.pinned)) =>
            {
                Some(PortalTabAction::MoveSelectedToEnd(anchor))
            }
            Self::PinnedTab(_) | Self::Tabs(_) => None,
        }
    }

    pub(crate) fn drop_at_end(
        ui: &Ui,
        viewport: Rect,
        end_y: f32,
        tabs: &PortalTabs,
        pane: PortalPaneId,
    ) -> Option<PortalTabAction> {
        let source = DragAndDrop::payload::<Self>(ui.ctx())?;
        let mut trailing_space = viewport.intersect(ui.clip_rect());
        trailing_space.min.y = trailing_space.min.y.max(end_y);
        if !trailing_space.is_positive() || !ui.rect_contains_pointer(trailing_space) {
            return None;
        }
        let action = source.end_action(tabs, pane)?;
        ui.painter().hline(
            trailing_space.x_range(),
            trailing_space.top(),
            Stroke::new(2.0, Color32::from_rgb(0, 120, 212)),
        );
        if ui.input(|input| input.pointer.button_released(PointerButton::Primary)) {
            DragAndDrop::take_payload::<Self>(ui.ctx()).map(|_| action)
        } else {
            None
        }
    }

    fn move_tab_action(source: Self, target: PortalTabId, before: bool) -> PortalTabAction {
        match source {
            Self::Tab(tab) | Self::PinnedTab(tab) if before => {
                PortalTabAction::MoveBefore(tab, target)
            }
            Self::Tab(tab) | Self::PinnedTab(tab) => PortalTabAction::MoveAfter(tab, target),
            Self::Tabs(anchor) => PortalTabAction::MoveSelectedBeside {
                anchor,
                target,
                before,
            },
            Self::Group(group) => PortalTabAction::MoveGroup(group, target, before),
        }
    }
}

fn highlight_drop(
    ui: &Ui,
    response: &Response,
    action: PortalTabAction,
) -> Option<PortalTabAction> {
    ui.painter().rect_stroke(
        response.rect,
        3.0,
        Stroke::new(2.0, Color32::from_rgb(0, 120, 212)),
        StrokeKind::Inside,
    );
    response
        .dnd_release_payload::<PortalTabDrag>()
        .map(|_| action)
}

fn show_drop_edge(ui: &Ui, response: &Response, before: bool, stroke: Stroke) {
    let rect = response.rect;
    ui.painter().hline(
        rect.x_range(),
        if before { rect.top() } else { rect.bottom() },
        stroke,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_page::PortalPage;
    use crate::portal_split_side::PortalSplitSide;
    use crate::portal_tab_target::PortalTabTarget;
    use eframe::egui::Button;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::Modifiers;
    use eframe::egui::Pos2;
    use eframe::egui::RawInput;
    use eframe::egui::Sense;

    fn drag_to(
        source: PortalTabDrag,
        position_in_target: impl FnOnce(Rect) -> Pos2,
        drop: impl Fn(&Ui, &Response) -> Option<PortalTabAction>,
    ) -> Option<PortalTabAction> {
        let context = Context::default();
        let frame = |events| {
            let mut source_rect = Rect::ZERO;
            let mut target_rect = Rect::ZERO;
            let mut action = None;
            let input = RawInput {
                events,
                ..Default::default()
            };
            let mut output = context.run_ui(input, |ui| {
                let source_button = ui.add_sized(
                    [32.0, 32.0],
                    Button::new("Drag").sense(Sense::click_and_drag()),
                );
                source_rect = source_button.rect;
                source_button.dnd_set_drag_payload(source);
                let target = ui.add_sized([32.0, 32.0], Button::new("Here"));
                target_rect = target.rect;
                action = drop(ui, &target);
            });
            output.textures_delta.clear();
            (source_rect, target_rect, action)
        };
        let (source_rect, target_rect, _) = frame(Vec::new());
        let origin = source_rect.center();
        let destination = position_in_target(target_rect);
        let mouse = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(vec![Event::PointerMoved(origin), mouse(origin, true)]);
        frame(vec![Event::PointerMoved(destination)]);
        frame(vec![mouse(destination, false)]).2
    }

    #[test]
    fn pinned_icons_use_horizontal_drag_positions() {
        let mut tabs = PortalTabs::default();
        let first = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let last = tabs.active_id();
        tabs.set_pinned(first, true);
        tabs.set_pinned(last, true);
        let target = tabs.iter().next().unwrap().1;

        let action = drag_to(
            PortalTabDrag::PinnedTab(last),
            |rect| Pos2::new(rect.left() + 3.0, rect.center().y),
            |ui, response| PortalTabDrag::drop_on_tab(ui, response, target, &tabs),
        );
        let Some(PortalTabAction::MoveBefore(source, target)) = action else {
            panic!("left side of pinned tab did not accept a reorder");
        };
        tabs.move_tab(source, target, true);
        assert_eq!(
            tabs.iter().map(|(_, tab, _)| tab.id).collect::<Vec<_>>(),
            [last, first]
        );
    }

    #[test]
    fn dragging_cannot_move_tabs_between_pinned_and_regular_sections() {
        let mut tabs = PortalTabs::default();
        let pinned_id = tabs.active_id();
        tabs.set_pinned(pinned_id, true);
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let regular_id = tabs.active_id();
        let mut entries = tabs.iter().map(|(_, tab, _)| tab);
        let pinned = entries.next().unwrap();
        let regular = entries.next().unwrap();

        for (source, target) in [
            (PortalTabDrag::Tab(regular_id), pinned),
            (PortalTabDrag::PinnedTab(pinned_id), regular),
        ] {
            assert!(
                drag_to(
                    source,
                    |rect| rect.center(),
                    |ui, response| { PortalTabDrag::drop_on_tab(ui, response, target, &tabs) }
                )
                .is_none()
            );
        }
        assert!(
            drag_to(
                PortalTabDrag::PinnedTab(pinned_id),
                |rect| rect.center(),
                |ui, response| PortalTabDrag::drop_at_end(
                    ui,
                    response.rect,
                    response.rect.top(),
                    &tabs,
                    tabs.active_pane()
                ),
            )
            .is_none()
        );
    }

    #[test]
    fn dragging_from_a_pinned_icon_transfers_the_entire_selection() {
        let mut tabs = PortalTabs::default();
        let pinned = tabs.active_id();
        tabs.set_pinned(pinned, true);
        tabs.open_new_tab();
        let destination_tab = tabs.active_id();
        tabs.split_tabs(
            &[destination_tab],
            tabs.active_pane(),
            tabs.active_pane(),
            PortalSplitSide::Right,
        );
        let destination = tabs.active_pane();

        tabs.select(tabs.index_of(pinned).unwrap());
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let regular = tabs.active_id();
        tabs.select_from_rail(pinned, Modifiers::CTRL);
        let target = tabs.tab(destination_tab).unwrap();

        let drop = drag_to(
            PortalTabDrag::Tabs(pinned),
            |rect| rect.center(),
            |ui, response| PortalTabDrag::drop_on_tab(ui, response, target, &tabs),
        );
        let Some(PortalTabAction::MoveToPane(drag, pane)) = drop else {
            panic!("tab in the other pane did not accept the selection");
        };
        assert_eq!(pane, destination);
        assert_eq!(drag.tabs(&tabs), [pinned, regular]);
        tabs.move_tabs_to_pane(&drag.tabs(&tabs), destination);
        assert!(tabs.tab(pinned).unwrap().pinned);
        assert!(!tabs.tab(regular).unwrap().pinned);
        assert_eq!(tabs.tab(pinned).unwrap().pane, destination);
        assert_eq!(tabs.tab(regular).unwrap().pane, destination);
    }

    #[test]
    fn dropping_on_a_regular_tab_distinguishes_the_center_from_the_edge() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let about = tabs.active_id();
        let target = tabs.iter().next().unwrap().1;
        let source = PortalTabDrag::Tab(about);
        let drop =
            |ui: &Ui, response: &Response| PortalTabDrag::drop_on_tab(ui, response, target, &tabs);

        let edge = drag_to(
            source,
            |rect| Pos2::new(rect.center().x, rect.top() + 3.0),
            drop,
        );
        assert!(matches!(edge, Some(PortalTabAction::MoveBefore(a, b)) if (a, b) == (about, home)));
        let middle = drag_to(source, |rect| rect.center(), drop);
        let Some(PortalTabAction::GroupWith(source, target)) = middle else {
            panic!("the center of the tab did not accept grouping");
        };
        tabs.group_tabs(source, target);
        assert_eq!(tabs.groups().count(), 1);
        assert!(tabs.iter().all(|(_, tab, _)| tab.group.is_some()));
    }

    #[test]
    fn dragging_to_the_ungrouped_area_dissolves_a_two_tab_group() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::About));
        let about = tabs.active_id();
        tabs.group_tabs(about, home);

        let action = drag_to(
            PortalTabDrag::Tab(about),
            |rect| rect.center(),
            |ui, response| {
                PortalTabDrag::drop_at_end(
                    ui,
                    response.rect,
                    response.rect.top(),
                    &tabs,
                    tabs.active_pane(),
                )
            },
        );
        let Some(PortalTabAction::MoveOutsideGroups(tab)) = action else {
            panic!("the ungrouped drop area did not accept a tab");
        };
        tabs.move_to_ungrouped_end(tab);
        assert!(tabs.groups().next().is_none());
        assert!(tabs.iter().all(|(_, tab, _)| tab.group.is_none()));
    }

    #[test]
    fn splitting_a_cross_pane_selection_copies_the_drag_anchor_rail() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let left = tabs.active_pane();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();
        tabs.split_tabs(&[logs], left, left, PortalSplitSide::Right);
        let right = tabs.active_pane();
        tabs.set_pane_expanded(right, false);
        tabs.select_from_rail(home, Modifiers::CTRL);
        tabs.select_from_rail(logs, Modifiers::CTRL);

        let drag = PortalTabDrag::Tabs(logs);
        let moving = drag.tabs(&tabs);
        assert_eq!(moving, [home, logs]);
        let source = drag.source_pane(&tabs).unwrap();
        tabs.split_tabs(&moving, source, left, PortalSplitSide::Bottom);

        assert!(tabs.pane_expanded(left));
        assert!(!tabs.pane_expanded(tabs.active_pane()));
    }

    #[test]
    fn dropping_in_blank_rail_space_moves_a_tab_to_the_end_of_the_destination_pane() {
        let mut tabs = PortalTabs::default();
        let home = tabs.active_id();
        let destination = tabs.active_pane();
        tabs.open_kept(PortalTabTarget::Page(PortalPage::Logs));
        let logs = tabs.active_id();
        tabs.split_tabs(&[logs], destination, destination, PortalSplitSide::Right);

        let action = drag_to(
            PortalTabDrag::Tab(logs),
            |rect| Pos2::new(rect.center().x, rect.bottom() + 120.0),
            |ui, response| {
                let mut viewport = response.rect;
                viewport.max.y += 160.0;
                PortalTabDrag::drop_at_end(ui, viewport, response.rect.top(), &tabs, destination)
            },
        );
        let Some(PortalTabAction::MoveToPane(drag, pane)) = action else {
            panic!("blank rail space did not accept the tab from the other pane");
        };
        assert_eq!(pane, destination);
        tabs.move_tabs_to_pane(&drag.tabs(&tabs), pane);
        let order: Vec<_> = tabs.iter_in_pane(pane).map(|(_, tab, _)| tab.id).collect();
        assert_eq!(order, [home, logs]);
    }
}
