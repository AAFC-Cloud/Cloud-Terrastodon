use crate::config::Config;
use emath::Pos2;
use emath::Rect;
use emath::Vec2;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct EguiConfig {
    pub starting_points_window: Rect,
    pub open_dirs: HashMap<PathBuf, Rect>,
}

impl Default for EguiConfig {
    fn default() -> Self {
        Self {
            starting_points_window: Rect::from_min_size(Pos2::new(10., 10.), Vec2::new(500., 500.)),
            open_dirs: HashMap::new(),
        }
    }
}

#[async_trait::async_trait]
impl Config for EguiConfig {
    const FILE_SLUG: &'static str = "egui_ui_state";
}
