use crate::app::MyApp;
use eframe::egui;
use eframe::egui::Ui;
use egui_tiles::Behavior;
use egui_tiles::SimplificationOptions;
use egui_tiles::Tabs;
use egui_tiles::TileId;
use egui_tiles::Tiles;
use egui_tiles::Tree;
use egui_tiles::UiResponse;
use std::fmt::Debug;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Pane {
    StartingPoints,
    SelectedItems,
    WorkDirs,
    FileDrag,
}

pub struct TreeBehavior {
    app_ptr: Option<*mut MyApp>,
    simplification_options: SimplificationOptions,
    tab_bar_height: f32,
    gap_width: f32,
}

impl Default for TreeBehavior {
    fn default() -> Self {
        Self {
            app_ptr: None,
            simplification_options: SimplificationOptions {
                prune_empty_tabs: true,
                prune_empty_containers: true,
                prune_single_child_tabs: false,
                prune_single_child_containers: true,
                all_panes_must_have_tabs: true,
                join_nested_linear_containers: true,
            },
            tab_bar_height: 26.0,
            gap_width: 2.0,
        }
    }
}

impl TreeBehavior {
    /// Set the raw pointer to the app without creating extra borrows.
    pub fn set_app(&mut self, app_ptr: *mut MyApp) {
        self.app_ptr = Some(app_ptr);
    }
    pub fn clear_app(&mut self) {
        self.app_ptr = None;
    }

    fn app(&mut self) -> &mut MyApp {
        unsafe {
            &mut *self
                .app_ptr
                .expect("TreeBehavior: app not set before UI render")
        }
    }
}

impl Behavior<Pane> for TreeBehavior {
    fn pane_ui(&mut self, ui: &mut Ui, _tile_id: TileId, view: &mut Pane) -> UiResponse {
        match view {
            Pane::StartingPoints => {
                crate::windows::starting_points_window::starting_points_ui(self.app(), ui);
                UiResponse::None
            }
            Pane::SelectedItems => {
                crate::windows::selected_items_window::selected_items_ui(self.app(), ui);
                UiResponse::None
            }
            Pane::WorkDirs => {
                let mut dnd_resp = None;
                crate::windows::work_dirs_window::work_dirs_ui(self.app(), ui, &mut dnd_resp);
                if let Some(crate::windows::work_dirs_window::DNDResponse {
                    source_index,
                    destination_index,
                }) = dnd_resp
                {
                    let item = self
                        .app()
                        .work_dirs_config
                        .work_dirs
                        .remove_index(source_index)
                        .unwrap();
                    self.app()
                        .work_dirs_config
                        .work_dirs
                        .shift_insert(destination_index, item);
                }
                UiResponse::None
            }
            Pane::FileDrag => {
                crate::file_drag_and_drop::file_drag_ui(self.app(), ui);
                UiResponse::None
            }
        }
    }

    fn tab_title_for_pane(&mut self, pane: &Pane) -> egui::WidgetText {
        match pane {
            Pane::StartingPoints => "Starting Points".into(),
            Pane::SelectedItems => "Selected Items".into(),
            Pane::WorkDirs => "Work Dirs".into(),
            Pane::FileDrag => "File Drop".into(),
        }
    }

    fn top_bar_right_ui(
        &mut self,
        _tiles: &Tiles<Pane>,
        ui: &mut Ui,
        _tile_id: TileId,
        _tabs: &Tabs,
        _scroll_offset: &mut f32,
    ) {
        if ui.button("➕").clicked() {
            // no-op: adding new panes can be done later if needed
        }
    }

    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        self.tab_bar_height
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        self.gap_width
    }

    fn simplification_options(&self) -> SimplificationOptions {
        self.simplification_options
    }

    fn is_tab_closable(&self, _tiles: &Tiles<Pane>, _tile_id: TileId) -> bool {
        true
    }
}

pub fn create_tree() -> Tree<Pane> {
    let mut next = || Pane::StartingPoints; // just placeholder generator not used
    let mut tiles = Tiles::default();

    let starting = tiles.insert_pane(Pane::StartingPoints);
    let selected = tiles.insert_pane(Pane::SelectedItems);
    let workdirs = tiles.insert_pane(Pane::WorkDirs);
    let filedrop = tiles.insert_pane(Pane::FileDrag);

    let children = vec![starting, selected, workdirs, filedrop];
    let root = tiles.insert_tab_tile(children);

    Tree::new("main_tiles", root, tiles)
}
