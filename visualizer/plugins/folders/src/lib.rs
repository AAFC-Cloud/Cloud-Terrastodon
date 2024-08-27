use bevy::prelude::*;

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
    // get
}
