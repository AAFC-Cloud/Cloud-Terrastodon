use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureDevopsRepoComponent;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::css::BLUE;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevOpsProject;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_leader_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizablePrimary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use std::ops::Deref;

pub struct AzureDevopsProjectsPlugin;
impl Plugin for AzureDevopsProjectsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building AzureDevopsProjectsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureDevopsProjectIconData>();
        app.init_resource::<AzureDevopsProjectIconData>();
        app.observe(join_on_leader_added(
            |project: &AzureDevopsProjectComponent, repo: &AzureDevopsRepoComponent| {
                repo.project.id == project.id
            },
        ));
    }
}

#[derive_graph_node_icon_data]
struct AzureDevopsProjectIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: IconHandle,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub struct AzureDevopsProjectComponent {
    pub inner: AzureDevOpsProject,
}
impl Deref for AzureDevopsProjectComponent {
    type Target = AzureDevOpsProject;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn setup(
    mut handles: ResMut<AzureDevopsProjectIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up azure devops project icon data");
    handles.circle_icon = asset_server
        .load::<Svg>("textures/azure_devops/10261-icon-service-Azure-DevOps.svg")
        .into();
    handles.circle_material = materials.add(Color::from(BLUE));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 75.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<AzureDevopsProjectIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListAzureDevopsProjects(projects) = msg else {
            continue;
        };
        debug!("Received {} azure devops projects", projects.len());
        for (i, project) in projects.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("Azure DevOps Project - {}", project.name)),
                    text: project.name.to_string(),
                    translation: Vec3::new(0., i as f32 * 150., 0.),
                    top_extras: (
                        AzureDevopsProjectComponent {
                            inner: project.to_owned(),
                        },
                        OrganizablePrimary,
                        CollisionLayers::new(PhysLayer::Node, PhysLayer::Cursor),
                    ),
                    text_extras: (),
                    circle_extras: (),
                    icon_extras: (),
                },
                icon_data.as_ref(),
                &mut commands,
            );
        }
    }
}
