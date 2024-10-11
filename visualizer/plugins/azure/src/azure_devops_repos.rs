use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureDevopsProjectComponent;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::css::ORANGE;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsRepo;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_thing_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use std::ops::Deref;
pub struct AzureDevopsReposPlugin;
impl Plugin for AzureDevopsReposPlugin {
    fn build(&self, app: &mut App) {
        info!("Building AzureDevopsReposPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureDevopsRepoIconData>();
        app.init_resource::<AzureDevopsRepoIconData>();
        app.observe(join_on_thing_added(
            |repo: &AzureDevopsRepoComponent, project: &AzureDevopsProjectComponent| {
                repo.project.id == project.id
            },
        ));
    }
}

#[derive_graph_node_icon_data]
struct AzureDevopsRepoIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: IconHandle,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub struct AzureDevopsRepoComponent {
    pub inner: AzureDevopsRepo,
}
impl Deref for AzureDevopsRepoComponent {
    type Target = AzureDevopsRepo;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

fn setup(
    mut handles: ResMut<AzureDevopsRepoIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up azure devops Repo icon data");
    handles.circle_icon = asset_server
        .load::<Image>("textures/azure_devops/repos.png")
        .into();
    handles.circle_material = materials.add(Color::from(ORANGE));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 96;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 50.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<AzureDevopsRepoIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListAzureDevopsRepos(repos) = msg else {
            continue;
        };
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} azure devops Repos", repos.len());
        for (i, repo) in repos.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("Azure DevOps Repo - {}", repo.name)),
                    text: repo.name.to_owned(),
                    translation: Vec3::new(0., i as f32 * 150., 0.),
                    top_extras: (
                        AzureDevopsRepoComponent {
                            inner: repo.to_owned(),
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
