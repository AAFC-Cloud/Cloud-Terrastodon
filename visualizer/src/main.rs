use bevy::input::common_conditions::input_toggle_active;
use bevy::log::LogPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_svg::prelude::*;
use cloud_terrastodon_visualizer_azure_plugin::prelude::AzurePlugin;
use cloud_terrastodon_visualizer_camera_plugin::MyCameraPlugin;
use cloud_terrastodon_visualizer_cursor_plugin::CursorPlugin;
use cloud_terrastodon_visualizer_graph_nodes_plugin::GraphNodesPlugin;
use cloud_terrastodon_visualizer_layout_plugin::LayoutPlugin;
use cloud_terrastodon_visualizer_physics_plugin::PhysicsPlugin;
use cloud_terrastodon_visualizer_tofu_plugin::TofuPlugin;
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
            cloud_terrastodon_visualizer_cursor_plugin::cursor_hover_plugin=error
            cloud_terrastodon_visualizer_layout_plugin::bias_towards_origin=info
            avian2d::collision::narrow_phase=error
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
    app.add_plugins(CursorPlugin);
    app.add_plugins(TofuPlugin);
    app.add_plugins(LayoutPlugin);
    app.add_plugins(PhysicsPlugin);
    app.add_plugins(GraphNodesPlugin);
   
    // must be after the default plugins
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)),
    );

    app.run();
}
