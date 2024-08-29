mod folder_plugin;
mod tofu_worker_plugin;

use bevy::prelude::*;
use folder_plugin::FoldersPlugin;
use tofu_worker_plugin::TofuWorkerPlugin;
pub struct TofuPlugin;

impl Plugin for TofuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FoldersPlugin);
        app.add_plugins(TofuWorkerPlugin);
    }
}
