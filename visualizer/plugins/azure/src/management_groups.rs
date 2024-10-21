use crate::prelude::AzureCliResponse;
use crate::prelude::AzureSubscription;
use crate::scope::AzureScope;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::tailwind::GRAY_700;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::ManagementGroup;
use cloud_terrastodon_core_azure::prelude::ManagementGroupId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::TenantId;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_follower_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_leader_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizablePrimary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use std::ops::Deref;

pub struct ManagementGroupsPlugin;
impl Plugin for ManagementGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ManagementGroupsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<ManagementGroupIconData>();
        app.init_resource::<ManagementGroupIconData>();
        app.observe(join_on_leader_added(
            |added_mg: &AzureManagementGroup, exists_mg: &AzureManagementGroup| {
                Some(&added_mg.id) == exists_mg.parent_id.as_ref()
            },
        ));
        app.observe(join_on_follower_added(
            |added_mg: &AzureManagementGroup, exists_mg: &AzureManagementGroup| {
                added_mg.parent_id.as_ref() == Some(&exists_mg.id)
            },
        ));
        app.observe(join_on_leader_added(
            |added_mg: &AzureManagementGroup, exists_sub: &AzureSubscription| {
                added_mg.id == exists_sub.parent_management_group_id
            },
        ));
    }
}

#[derive_graph_node_icon_data]
struct ManagementGroupIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: IconHandle,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Reflect, Component)]
#[reflect(Default)]
pub struct AzureManagementGroup {
    #[reflect(ignore)]
    pub inner: ManagementGroup,
}
impl Deref for AzureManagementGroup {
    type Target = ManagementGroup;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl Default for AzureManagementGroup {
    fn default() -> Self {
        Self {
            inner: ManagementGroup {
                id: ManagementGroupId::from_name("FakeManagementGroup"),
                display_name: "FakeManagementGroup".to_owned(),
                tenant_id: TenantId::new(Uuid::nil()),
                parent_id: None,
            },
        }
    }
}

fn setup(
    mut handles: ResMut<ManagementGroupIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up resource group icon data");
    // The ManagementGroup.svg file has been modified to remove a path element that causes problems
    // https://github.com/Weasy666/bevy_svg/issues/42
    handles.circle_icon = asset_server
        .load::<Svg>("textures/azure/ManagementGroup.svg")
        .into();
    handles.circle_material = materials.add(Color::from(GRAY_700));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 24.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 90.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<ManagementGroupIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListAzureManagementGroups(management_group) = msg else {
            continue;
        };
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} ManagementGroups", management_group.len());
        for (i, management_group) in management_group.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("ManagementGroup - {}", management_group.name())),
                    text: management_group.display_name.to_owned(),
                    translation: Vec3::new(-1000., i as f32 * 250., 0.),
                    top_extras: (
                        AzureManagementGroup {
                            inner: management_group.to_owned(),
                        },
                        AzureScope {
                            scope: management_group.id.as_scope(),
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
