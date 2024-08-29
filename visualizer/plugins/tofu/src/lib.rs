#![feature(try_blocks)]
mod folder_plugin;
mod tofu_worker_plugin;
mod import_blocks_plugin;

use bevy::prelude::*;
use folder_plugin::FoldersPlugin;
use import_blocks_plugin::TofuImportBlocksPlugin;
use tofu_worker_plugin::TofuWorkerPlugin;
pub struct TofuPlugin;

impl Plugin for TofuPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FoldersPlugin);
        app.add_plugins(TofuWorkerPlugin);
        app.add_plugins(TofuImportBlocksPlugin);
    }
}