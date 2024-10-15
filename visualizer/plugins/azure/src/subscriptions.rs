use crate::prelude::AzureCliResponse;
use crate::prelude::AzureManagementGroup;
use crate::resource_groups::AzureResourceGroup;
use crate::scope::AzureScope;
use avian2d::prelude::CollisionLayers;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Svg;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::ManagementGroupId;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
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

pub struct SubscriptionsPlugin;
impl Plugin for SubscriptionsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building SubscriptionsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<SubscriptionIconData>();
        app.init_resource::<SubscriptionIconData>();
        app.observe(join_on_leader_added(
            |sub: &AzureSubscription, rg: &AzureResourceGroup| sub.id == rg.subscription_id,
        ));
        app.observe(join_on_follower_added(
            |sub: &AzureSubscription, mg: &AzureManagementGroup| {
                sub.parent_management_group_id == mg.id
            },
        ));
    }
}

#[derive_graph_node_icon_data]
struct SubscriptionIconData {
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
pub struct AzureSubscription {
    #[reflect(ignore)]
    pub subscription: Subscription,
}
impl Deref for AzureSubscription {
    type Target = Subscription;

    fn deref(&self) -> &Self::Target {
        &self.subscription
    }
}
impl Default for AzureSubscription {
    fn default() -> Self {
        Self {
            subscription: Subscription {
                id: SubscriptionId::new(Uuid::nil()),
                name: "FakeSubscription".to_owned(),
                tenant_id: TenantId::new(Uuid::nil()),
                parent_management_group_id: ManagementGroupId::from_name("FakeManagementGroup"),
            },
        }
    }
}

fn setup(
    mut handles: ResMut<SubscriptionIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up resource group icon data");
    // The Subscription.svg file has been modified to remove a path element that causes problems
    // https://github.com/Weasy666/bevy_svg/issues/42
    handles.circle_icon = asset_server
        .load::<Svg>("textures/azure/Subscription.svg")
        .into();
    handles.circle_material = materials.add(Color::from(YELLOW));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 4.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 75.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<SubscriptionIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListSubscriptions(subscriptions) = msg else {
            continue;
        };
        debug!("Received {} subscriptions", subscriptions.len());
        for (i, sub) in subscriptions.iter().enumerate() {
            spawn_graph_node(
                SpawnGraphNodeEvent {
                    name: Name::new(format!("Subscription - {}", sub.name)),
                    text: sub.name.to_owned(),
                    translation: Vec3::new(-1000., i as f32 * 250., 0.),
                    top_extras: (
                        AzureSubscription {
                            subscription: sub.to_owned(),
                        },
                        AzureScope {
                            scope: sub.id.as_scope(),
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
