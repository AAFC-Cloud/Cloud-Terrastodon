use std::ops::Deref;

use crate::az_cli::AzureCliEvent;
use crate::scope::AzureScope;
use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use azure::prelude::uuid::Uuid;
use azure::prelude::ResourceGroup;
use azure::prelude::ResourceGroupId;
use azure::prelude::Scope;
use azure::prelude::SubscriptionId;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;

pub struct ResourceGroupsPlugin;
impl Plugin for ResourceGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ResourceGroupsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureResourceGroup>();
        app.register_type::<ResourceGroupIconData>();
        app.init_resource::<ResourceGroupIconData>();
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct ResourceGroupIconData {
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
    handles.circle_icon = asset_server.load("textures/ResourceGroups.svg");
    handles.circle_material = materials.add(Color::from(BLACK));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 32.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 50.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliEvent>,
    mut commands: Commands,
    icon_data: Res<ResourceGroupIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliEvent::ListResourceGroups(resource_groups) = msg else {
            continue;
        };
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} resource groups", resource_groups.len());
        for (i, rg) in resource_groups.iter().enumerate() {
            commands
                .spawn((
                    Name::new(format!("Resource Group - {}", rg.name)),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0., i as f32 * 150., 0.)),
                        ..default()
                    },
                    AzureResourceGroup {
                        resource_group: rg.to_owned(),
                    },
                    AzureScope {
                        scope: rg.id.as_scope(),
                    },
                    RigidBody::Dynamic,
                    CustomLinearDamping::default(),
                    Collider::circle(icon_data.circle_radius),
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
                                rg.name.to_owned(),
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
                        OnlyShowWhenHovered,
                    ));
                });
        }
    }
}