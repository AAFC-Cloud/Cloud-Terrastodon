use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_svg::prelude::*;
use camera_plugin::MyCameraPlugin;
use resource_groups_plugin::ResourceGroupsPlugin;

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Sample4);

    app.add_plugins(DefaultPlugins);
    app.add_plugins(SvgPlugin);
    app.add_plugins(MyCameraPlugin);
    app.add_plugins(ResourceGroupsPlugin);

    // must be after the default plugins
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)),
    );

    app.run();
}
