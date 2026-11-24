use eframe::egui::Pos2;
use eframe::egui::Rect;

/// The side of the destination pane that receives a dragged tab.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PortalSplitSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl PortalSplitSide {
    pub fn nearest(rect: Rect, pointer: Pos2) -> Self {
        // Compare distances relative to the pane's dimensions. Dropping in the
        // center of a wide pane still offers a side-by-side split.
        let x = (pointer.x - rect.left()) / rect.width();
        let y = (pointer.y - rect.top()) / rect.height();
        [
            (1.0 - x, Self::Right),
            (x, Self::Left),
            (y, Self::Top),
            (1.0 - y, Self::Bottom),
        ]
        .into_iter()
        .min_by(|left, right| left.0.total_cmp(&right.0))
        .map_or(Self::Right, |(_, side)| side)
    }
}
