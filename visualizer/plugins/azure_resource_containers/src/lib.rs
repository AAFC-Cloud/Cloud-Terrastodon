mod az_cli;
mod resource_groups;

use az_cli::AzureCliPlugin;
use bevy::prelude::*;
use resource_groups::ResourceGroupsPlugin;
pub struct AzureResourceContainersPlugin;
impl Plugin for AzureResourceContainersPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ResourceGroupsPlugin);
        app.add_plugins(AzureCliPlugin);
    }
}
