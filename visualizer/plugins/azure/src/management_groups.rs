use crate::prelude::AzureCliResponse;
use crate::scope::AzureScope;
use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::color::palettes::css::MAGENTA;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_core_azure::prelude::uuid::Uuid;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_azure::prelude::ManagementGroup;
use cloud_terrastodon_core_azure::prelude::ManagementGroupId;
use cloud_terrastodon_core_azure::prelude::TenantId;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_thing_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::BiasTowardsOrigin;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizablePrimary;
use std::ops::Deref;

pub struct ManagementGroupsPlugin;
impl Plugin for ManagementGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ManagementGroupsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<ManagementGroupIconData>();
        app.init_resource::<ManagementGroupIconData>();
        app.observe(join_on_thing_added(
            |new: &AzureManagementGroup, exists: &AzureManagementGroup| new.parent_id.as_ref() == Some(&exists.id) || Some(&new.id) == exists.parent_id.as_ref(),
        ));
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct ManagementGroupIconData {
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
    handles.circle_icon = asset_server.load("textures/azure/ManagementGroup.svg");
    handles.circle_material = materials.add(Color::from(MAGENTA));
    handles.circle_mesh = meshes.add(Circle { radius: 1. }).into();
    handles.icon_width = 18;
    handles.circle_icon_padding = 4.;
    handles.circle_text_margin = 4.;
    handles.circle_radius = 90.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliResponse>,
    mut commands: Commands,
    icon_data: Res<ManagementGroupIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliResponse::ListManagementGroups(management_group) = msg else {
            continue;
        };
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} ManagementGroups", management_group.len());
        for (i, management_group) in management_group.iter().enumerate() {
            commands
                .spawn((
                    Name::new(format!("ManagementGroup - {}", management_group.name())),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(
                            -1000.,
                            i as f32 * 250.,
                            0.,
                        )),
                        ..default()
                    },
                    AzureManagementGroup {
                        inner: management_group.to_owned(),
                    },
                    AzureScope {
                        scope: management_group.id.as_scope(),
                    },
                    CustomLinearDamping::default(),
                    RigidBody::Dynamic,
                    Collider::circle(icon_data.circle_radius),
                    BiasTowardsOrigin,
                    KeepUpright,
                    OrganizablePrimary,
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
                                management_group.display_name.to_owned(),
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
