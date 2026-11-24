use crate::portal_pane_id::PortalPaneId;
use eframe::egui::Color32;
use eframe::egui::CursorIcon;
use eframe::egui::PointerButton;
use eframe::egui::Pos2;
use eframe::egui::Rect;
use eframe::egui::Sense;
use eframe::egui::Ui;
use eframe::egui::Vec2;

/// The position of a split divider within its containing pane layout.
#[derive(Clone, Copy)]
pub(crate) struct PortalSplitHandle {
    pub marker: PortalPaneId,
    pub bounds: Rect,
    pub rect: Rect,
    /// A horizontal resize has a vertical divider.
    pub horizontal: bool,
}

impl PortalSplitHandle {
    fn junction(self, other: Self) -> Option<Pos2> {
        if self.horizontal == other.horizontal || !self.rect.intersects(other.rect) {
            return None;
        }
        let (vertical, horizontal) = if self.horizontal {
            (self, other)
        } else {
            (other, self)
        };
        Some(Pos2::new(
            vertical.rect.center().x,
            horizontal.rect.center().y,
        ))
    }
}

/// Register corner handles after the ordinary dividers so they take precedence at the joint.
pub(crate) fn show_split_junctions(
    ui: &mut Ui,
    handles: &[PortalSplitHandle],
    mut resize: impl FnMut(PortalPaneId, Rect, Pos2),
) {
    for (index, &parent) in handles.iter().enumerate() {
        let descendants = &handles[index + 1..];
        for &child in descendants {
            let Some(center) = parent.junction(child) else {
                continue;
            };
            // Widen the hit area along the parent divider without covering pane controls.
            let grip = Rect::from_center_size(center, Vec2::splat(16.0)).intersect(parent.rect);
            let id = ui.make_persistent_id(("split-junction", parent.marker, child.marker));
            let response = ui
                .interact(grip, id, Sense::drag())
                .on_hover_and_drag_cursor(CursorIcon::ResizeNwSe);
            if response.hovered() || response.dragged() {
                ui.painter()
                    .rect_filled(grip, 0.0, Color32::from_rgb(0, 120, 212));
            }

            let drag_id = id.with("resize-offsets");
            if response.drag_started_by(PointerButton::Primary)
                && let Some(origin) = ui.input(|input| input.pointer.press_origin())
            {
                // Keep the starting offsets and membership through the gesture. If two
                // dividers meet at a crossing, both resize with the shared divider.
                let offsets = std::iter::once(parent)
                    .chain(descendants.iter().copied().filter(|&handle| {
                        parent
                            .junction(handle)
                            .is_some_and(|point| point.distance(center) <= 0.5)
                    }))
                    .map(|handle| (handle.marker, handle.rect.center() - origin))
                    .collect::<Vec<_>>();
                ui.data_mut(|data| data.insert_temp(drag_id, offsets));
            }

            if (response.dragged_by(PointerButton::Primary) && response.drag_delta() != Vec2::ZERO
                || response.drag_stopped_by(PointerButton::Primary))
                && let Some(pointer) = ui.input(|input| input.pointer.interact_pos())
                && let Some(offsets) =
                    ui.data(|data| data.get_temp::<Vec<(PortalPaneId, Vec2)>>(drag_id))
            {
                for (marker, offset) in offsets {
                    if let Some(handle) = handles.iter().find(|handle| handle.marker == marker) {
                        resize(marker, handle.bounds, pointer + offset);
                    }
                }
                ui.ctx().request_repaint();
            }
            if response.drag_stopped() {
                ui.data_mut(|data| data.remove::<Vec<(PortalPaneId, Vec2)>>(drag_id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::portal_pane_layout::PortalPaneLayout;
    use crate::portal_split_side::PortalSplitSide;
    use eframe::egui::Context;
    use eframe::egui::Event;
    use eframe::egui::Modifiers;
    use eframe::egui::RawInput;

    #[test]
    fn dragging_a_crossing_resizes_both_axes_and_keeps_the_aligned_dividers_together() {
        let bounds = Rect::from_min_size(Pos2::ZERO, Vec2::new(700.0, 500.0));
        let context = Context::default();
        let mut layout = PortalPaneLayout::Pane(PortalPaneId(0));
        assert!(layout.split(PortalPaneId(0), PortalPaneId(1), PortalSplitSide::Bottom));
        assert!(layout.split(PortalPaneId(0), PortalPaneId(2), PortalSplitSide::Right));
        assert!(layout.split(PortalPaneId(1), PortalPaneId(3), PortalSplitSide::Right));

        let frame = |layout: &mut PortalPaneLayout, events| {
            let mut handles = Vec::new();
            layout.split_handles(bounds, &mut handles);
            let mut output = context.run_ui(
                RawInput {
                    screen_rect: Some(bounds),
                    events,
                    ..Default::default()
                },
                |ui| {
                    for handle in &handles {
                        ui.interact(handle.rect, ui.id().with(handle.marker), Sense::drag())
                            .on_hover_cursor(if handle.horizontal {
                                CursorIcon::ResizeHorizontal
                            } else {
                                CursorIcon::ResizeVertical
                            });
                    }
                    show_split_junctions(ui, &handles, |marker, pane, center| {
                        layout.divider_mut(marker).unwrap().resize(pane, center);
                    });
                },
            );
            output.textures_delta.clear();
            output.platform_output.cursor_icon
        };
        frame(&mut layout, Vec::new());
        let origin = bounds.center() + Vec2::new(4.0, 0.0);
        assert_eq!(
            frame(&mut layout, vec![Event::PointerMoved(origin)]),
            CursorIcon::ResizeNwSe
        );
        let button = |pos, pressed| Event::PointerButton {
            pos,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        };
        frame(&mut layout, vec![button(origin, true)]);
        let movement = Vec2::new(55.0, 37.0);
        assert_eq!(
            frame(&mut layout, vec![Event::PointerMoved(origin + movement)]),
            CursorIcon::ResizeNwSe
        );
        frame(&mut layout, vec![button(origin + movement, false)]);

        let mut handles = Vec::new();
        layout.split_handles(bounds, &mut handles);
        for handle in handles {
            let center = handle.rect.center();
            let moved = bounds.center() + movement;
            assert!(if handle.horizontal {
                (center.x - moved.x).abs() < 0.01
            } else {
                (center.y - moved.y).abs() < 0.01
            });
        }
        let away_from_join = Pos2::new(25.0, bounds.center().y + movement.y);
        assert_eq!(
            frame(&mut layout, vec![Event::PointerMoved(away_from_join)]),
            CursorIcon::ResizeVertical
        );
    }
}
