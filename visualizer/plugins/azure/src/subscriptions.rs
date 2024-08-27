use std::ops::Deref;

use crate::az_cli::AzureCliEvent;
use crate::bias_towards_origin::BiasTowardsOrigin;
use crate::scope::AzureScope;
use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::AzureCloudKind;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure::prelude::SubscriptionId;
use cloud_terrastodon_core_azure::prelude::SubscriptionState;
use cloud_terrastodon_core_azure::prelude::SubscriptionUser;
use cloud_terrastodon_core_azure::prelude::TenantId;
use bevy::color::palettes::css::YELLOW;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;

pub struct SubscriptionsPlugin;
impl Plugin for SubscriptionsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building SubscriptionsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<SubscriptionIconData>();
        app.init_resource::<SubscriptionIconData>();
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct SubscriptionIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: Handle<Svg>,
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
                cloud_name: AzureCloudKind::AzureCloud,
                id: SubscriptionId::new(Uuid::nil()),
                is_default: false,
                name: "FakeSubscription".to_owned(),
                state: SubscriptionState::Enabled,
                tenant_id: TenantId::new(Uuid::nil()),
                user: SubscriptionUser {
                    name: "Fake".to_owned(),
                    kind: "Fake user in default constructor".to_owned(),
                },
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
    handles.circle_icon = asset_server.load("textures/Subscription.svg");
    handles.circle_material = materials.add(Color::from(YELLOW));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 4.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 75.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliEvent>,
    mut commands: Commands,
    icon_data: Res<SubscriptionIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliEvent::ListSubscriptions(subscriptions) = msg else {
            continue;
        };
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} subscriptions", subscriptions.len());
        for (i, sub) in subscriptions.iter().enumerate() {
            commands
                .spawn((
                    Name::new(format!("Subscription - {}", sub.name)),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            -1000.,
                            i as f32 * 250.,
                            0.,
                        )),
                        ..default()
                    },
                    AzureSubscription {
                        subscription: sub.to_owned(),
                    },
                    AzureScope {
                        scope: sub.id.as_scope(),
                    },
                    CustomLinearDamping::default(),
                    RigidBody::Dynamic,
                    Collider::circle(icon_data.circle_radius),
                    BiasTowardsOrigin,
                ))
                .with_children(|parent| {
                    let circle_scale = Vec2::splat(icon_data.circle_radius).extend(1.);
                    let circle_transform = Transform::from_scale(circle_scale);
                    parent.spawn((
                        Name::new("Circle"),
                        MaterialMesh2dBundle {
                            mesh: icon_data.circle_mesh.clone(),
                            transform: circle_transform,
                            material: icon_data.circle_material.clone(),
                            ..default()
                        },
                    ));

                    let icon_scale = Vec2::splat(
                        (1. / icon_data.icon_width as f32)
                            * ((icon_data.circle_radius * 2.) - icon_data.circle_icon_padding),
                    )
                    .extend(1.);
                    let icon_translation =
                        (Vec2::new(-icon_scale.x, icon_scale.y) * icon_data.icon_width as f32 / 2.)
                            .extend(1.);
                    let icon_transform =
                        Transform::from_translation(icon_translation).with_scale(icon_scale);
                    parent.spawn((
                        Name::new("Icon"),
                        Svg2dBundle {
                            svg: icon_data.circle_icon.clone(),
                            transform: icon_transform,
                            origin: Origin::TopLeft,
                            ..default()
                        },
                    ));

                    let text_translation = Vec3::new(
                        icon_data.circle_radius + icon_data.circle_text_margin,
                        0.,
                        5.,
                    );
                    parent.spawn((
                        Name::new("Text"),
                        Text2dBundle {
                            text: Text::from_section(
                                sub.name.to_owned(),
                                TextStyle {
                                    font_size: 60.,
                                    ..default()
                                },
                            )
                            .with_justify(JustifyText::Left),
                            text_anchor: Anchor::CenterLeft,
                            transform: Transform::from_translation(text_translation),
                            ..default()
                        },
                    ));
                });
        }
    }
}
