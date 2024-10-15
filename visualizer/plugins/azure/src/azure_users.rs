use crate::az_cli::AzureCliResponse;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::tailwind::CYAN_400;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure::prelude::User;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use std::ops::Deref;

pub struct AzureUsersPlugin;
impl Plugin for AzureUsersPlugin {
    fn build(&self, app: &mut App) {
        info!("Building AzureUsersPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureUserIconData>();
        app.init_resource::<AzureUserIconData>();
    }
}

#[derive_graph_node_icon_data]
struct AzureUserIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: IconHandle,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub struct AzureUserComponent {
    pub inner: User,
}
impl Deref for AzureUserComponent {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn setup(
    mut handles: ResMut<AzureUserIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    handles.circle_icon = asset_server.load::<Svg>("textures/azure/User.svg").into();
    handles.circle_material = materials.add(Color::from(CYAN_400));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 75.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<AzureUserIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListAzureUsers(users) = msg else {
            continue;
        };
        for (i, user) in users.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("Azure User - {}", user.display_name)),
                    text: user.display_name.to_owned(),
                    translation: Vec3::new(0., i as f32 * 150., 0.),
                    top_extras: (
                        AzureUserComponent {
                            inner: user.to_owned(),
                        },
                        OrganizableSecondary,
                        CollisionLayers::new(PhysLayer::Node, PhysLayer::Cursor),
                    ),
                    text_extras: (OnlyShowWhenHovered,),
                    circle_extras: (),
                    icon_extras: (),
                },
                icon_data.as_ref(),
                &mut commands,
            );
        }
    }
}
