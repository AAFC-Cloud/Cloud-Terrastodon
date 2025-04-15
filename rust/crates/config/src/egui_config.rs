use emath::Pos2;
use emath::Rect;
use emath::Vec2;
use serde::Deserialize;
use serde::Serialize;

use crate::iconfig::IConfig;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EguiConfig {
    pub starting_points_window: Rect,
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            starting_points_window: Rect::from_min_size(Pos2::new(10., 10.), Vec2::new(500., 500.)),
        }
    }
}

#[async_trait::async_trait]
impl IConfig for EguiConfig {
    const FILE_SLUG: &'static str = "egui_ui_state";
}
