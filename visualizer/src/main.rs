use avian2d::prelude::Gravity;
use avian2d::PhysicsPlugins;
use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_svg::prelude::*;
use cloud_terrastodon_visualizer_azure_plugin::AzurePlugin;
use cloud_terrastodon_visualizer_camera_plugin::MyCameraPlugin;
use itertools::Itertools;

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Sample4);

    let log_plugin = LogPlugin {
        level: bevy::log::Level::INFO,
        filter: "
            wgpu=error
            cloud_terrastodon=debug
            cloud_terrastodon_visualizer=debug
        "
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.starts_with("//"))
        .filter(|line| !line.is_empty())
        .join(",")
        .trim()
        .into(),
        ..default()
    };
    app.add_plugins(DefaultPlugins.set(log_plugin));
    app.add_plugins(SvgPlugin);
    app.add_plugins(MyCameraPlugin);
    app.add_plugins(AzurePlugin);
    app.add_plugins(PhysicsPlugins::default().with_length_unit(100.0));
    app.insert_resource(Gravity(Vec2::ZERO));

    // must be after the default plugins
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)),
    );

    app.run();
}
