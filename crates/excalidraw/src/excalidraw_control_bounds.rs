use crate::interactive_scene_fields::SceneFields;
use crate::interactive_scene_fields::number;

/// Axis-aligned control geometry. Rotated controls are deliberately rejected
/// rather than silently interpreting their unrotated bounding boxes.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ExcalidrawControlBounds {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ExcalidrawControlBounds {
    pub fn read(fields: &SceneFields) -> Option<Self> {
        if number(fields, "angle").unwrap_or(0.0).abs() > 0.000_001 {
            return None;
        }
        Self::unrotated(fields)
    }

    /// Used only to detect unsupported rotated text near a control, never to
    /// accept that text as an answer or select a log target.
    pub fn unrotated(fields: &SceneFields) -> Option<Self> {
        let bounds = Self {
            x: number(fields, "x")?,
            y: number(fields, "y")?,
            width: number(fields, "width")?,
            height: number(fields, "height")?,
        };
        ([bounds.x, bounds.y, bounds.width, bounds.height]
            .iter()
            .all(|value| value.is_finite())
            && bounds.width >= 0.0
            && bounds.height >= 0.0)
            .then_some(bounds)
    }

    pub fn contains(self, child: Self) -> bool {
        child.x >= self.x
            && child.y >= self.y
            && child.x + child.width <= self.x + self.width
            && child.y + child.height <= self.y + self.height
    }

    pub fn contains_center(self, child: Self) -> bool {
        let x = child.x + child.width / 2.0;
        let y = child.y + child.height / 2.0;
        x >= self.x && y >= self.y && x <= self.x + self.width && y <= self.y + self.height
    }
}
