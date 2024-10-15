use avian2d::prelude::Gravity;
use avian2d::prelude::PhysicsDebugPlugin;
use avian2d::prelude::PhysicsGizmos;
use avian2d::PhysicsPlugins;
use bevy::prelude::*;
use damping::DampingPlugin;
use pause::PhysicsPausePlugin;

mod layers;
mod damping;
mod pause;

pub struct PhysicsPlugin;
impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        // Configure Avian
        #[cfg(debug_assertions)]
        {
            app.add_plugins(PhysicsDebugPlugin::default());
            {
                let mut store = app.world_mut().resource_mut::<GizmoConfigStore>();
                let config = store.config_mut::<PhysicsGizmos>().0;
                config.enabled = false;
            }
            app.add_systems(
                Update,
                |inputs: Res<ButtonInput<KeyCode>>, mut store: ResMut<GizmoConfigStore>| {
                    store.config_mut::<PhysicsGizmos>().0.enabled ^=
                        inputs.just_pressed(KeyCode::Backquote);
                },
            );
        }

        app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0));
        app.insert_resource(Gravity(Vec2::ZERO));

        // Add our plugins
        app.add_plugins(DampingPlugin);
        app.add_plugins(PhysicsPausePlugin);
    }
}

pub mod prelude {
    pub use crate::layers::*;
    pub use crate::damping::*;
    pub use crate::*;
}
