use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureScope;
use crate::subscriptions::AzureSubscription;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::ResourceGroup;
use cloud_terrastodon_core_azure::prelude::ResourceGroupId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_graph_nodes_derive::derive_graph_node_icon_data;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::spawn_graph_node;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::GraphNodeIconData;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::IconHandle;
use cloud_terrastodon_visualizer_graph_nodes_plugin::prelude::SpawnGraphNodeEvent;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_follower_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::PhysLayer;
use std::ops::Deref;

pub struct ResourceGroupsPlugin;
impl Plugin for ResourceGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ResourceGroupsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureResourceGroup>();
        app.register_type::<ResourceGroupIconData>();
        app.init_resource::<ResourceGroupIconData>();
        app.observe(join_on_follower_added(
            |rg: &AzureResourceGroup, sub: &AzureSubscription| rg.subscription_id == sub.id,
        ));
    }
}

#[derive_graph_node_icon_data]
struct ResourceGroupIconData {
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
pub struct AzureResourceGroup {
    #[reflect(ignore)]
    pub resource_group: ResourceGroup,
}
impl Deref for AzureResourceGroup {
    type Target = ResourceGroup;

    fn deref(&self) -> &Self::Target {
        &self.resource_group
    }
}
impl Default for AzureResourceGroup {
    fn default() -> Self {
        let name = "FakeResourceGroup";
        let subscription_id = SubscriptionId::new(Uuid::nil());
        let id = ResourceGroupId::new(&subscription_id, name.to_string());
        Self {
            resource_group: ResourceGroup {
                id,
                subscription_id,
                location: "canadacentral".to_owned(),
                managed_by: None,
                name: name.to_owned(),
                properties: Default::default(),
                tags: Default::default(),
            },
        }
    }
}

fn setup(
    mut handles: ResMut<ResourceGroupIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up resource group icon data");
    handles.circle_icon = asset_server
        .load::<Svg>("textures/azure/ResourceGroups.svg")
        .into();
    handles.circle_material = materials.add(Color::from(BLACK));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 50.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<ResourceGroupIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListResourceGroups(resource_groups) = msg else {
            continue;
        };
        debug!("Received {} resource groups", resource_groups.len());
        for (i, rg) in resource_groups.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("Resource Group - {}", rg.name)),
                    text: rg.name.to_owned(),
                    translation: Vec3::new(0., i as f32 * 150., 0.),
                    top_extras: (
                        AzureResourceGroup {
                            resource_group: rg.to_owned(),
                        },
                        AzureScope {
                            scope: rg.id.as_scope(),
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
