use bevy::prelude::*;
use cloud_terrastodon_core_config::Config;

pub struct FoldersPlugin;

impl Plugin for FoldersPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<Folder>();
        app.add_systems(Startup, setup);
    }
}

#[derive(Component, Reflect, Debug)]
pub struct Folder;

fn setup(mut commands: Commands) {
    let scan_dirs = &Config::get_active_config().scan_dirs;
    for dir in scan_dirs.iter() {
        commands.spawn((
            Name::new(format!("Folder - {}", dir.display())),
            Folder,
        ));
    }
}
