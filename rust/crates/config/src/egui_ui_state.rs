use serde::Deserialize;
use serde::Serialize;

use crate::iconfig::IConfig;

#[derive(Debug, Serialize, Deserialize)]
pub struct EguiUiState {
    pub window_size: (u32, u32),
    pub window_pos: (f64, f64),
}

impl Default for EguiUiState {
    fn default() -> Self {
        Self {
            window_size: (800, 600),
            window_pos: (10.0, 10.0),
        }
    }
}

#[async_trait::async_trait]
impl IConfig for EguiUiState {
    const FILE_SLUG: &'static str = "egui_ui_state";
}
