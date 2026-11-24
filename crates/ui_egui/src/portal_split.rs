use crate::portal_split_side::PortalSplitSide;
use eframe::egui::Pos2;
use eframe::egui::Rect;

const DIVIDER_WIDTH: f32 = 6.0;

/// Geometry of an adjustable vertical or horizontal workspace divider.
#[derive(Clone, Copy)]
pub(crate) struct PortalSplit {
    horizontal: bool,
    fraction: f32,
}

impl PortalSplit {
    pub fn new(side: PortalSplitSide) -> Self {
        Self {
            horizontal: matches!(side, PortalSplitSide::Left | PortalSplitSide::Right),
            fraction: 0.5,
        }
    }

    pub fn horizontal(self) -> bool {
        self.horizontal
    }

    pub fn resize(&mut self, bounds: Rect, pointer: Pos2) {
        let (offset, size) = if self.horizontal {
            (pointer.x - bounds.left(), bounds.width())
        } else {
            (pointer.y - bounds.top(), bounds.height())
        };
        let gap = DIVIDER_WIDTH.min(size);
        if size > gap {
            self.fraction = ((offset - gap * 0.5) / (size - gap)).clamp(0.2, 0.8);
        }
    }

    pub fn layout(self, bounds: Rect) -> (Rect, Rect, Rect) {
        let mut first = bounds;
        let mut divider = bounds;
        let mut second = bounds;
        if self.horizontal {
            let gap = DIVIDER_WIDTH.min(bounds.width());
            let x = bounds.left() + (bounds.width() - gap) * self.fraction;
            first.set_right(x);
            divider.set_left(x);
            divider.set_right(x + gap);
            second.set_left(x + gap);
        } else {
            let gap = DIVIDER_WIDTH.min(bounds.height());
            let y = bounds.top() + (bounds.height() - gap) * self.fraction;
            first.set_bottom(y);
            divider.set_top(y);
            divider.set_bottom(y + gap);
            second.set_top(y + gap);
        }
        (first, divider, second)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::Vec2;

    #[test]
    fn drop_preview_and_divider_track_the_destination_edge() {
        let bounds = Rect::from_min_size(Pos2::new(100.0, 50.0), Vec2::new(800.0, 500.0));
        let pointer = Pos2::new(bounds.center().x, bounds.bottom() - 1.0);
        let side = PortalSplitSide::nearest(bounds, pointer);
        let mut split = PortalSplit::new(side);
        let (top, divider, bottom) = split.layout(bounds);
        assert_eq!(top.top(), bounds.top());
        assert_eq!(top.bottom(), divider.top());
        assert_eq!(bottom.top(), divider.bottom());
        assert_eq!(bottom.bottom(), bounds.bottom());

        split.resize(bounds, pointer);
        let (_, divider, bottom) = split.layout(bounds);
        assert!(divider.top() > bounds.center().y);
        assert!(bottom.height() >= bounds.height() * 0.19);
    }
}
