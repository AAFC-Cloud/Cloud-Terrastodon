use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;

use crate::az_cli::AzureCliBridge;
use crate::az_cli::AzureCliEvent;
pub struct ResourceGroupsPlugin;

impl Plugin for ResourceGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ResourceGroupsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<ResourceGroupIconData>();
        app.init_resource::<ResourceGroupIconData>();
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct ResourceGroupIconData {
    pub scale: f32,
    pub circle_icon: Handle<Svg>,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
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
    handles.circle_mesh = meshes.add(Circle::default()).into();
    handles.scale = 100. / 18.;
}

fn receive_results(
    mut cli_events: EventReader<AzureCliEvent>,
    mut commands: Commands,
    icon_data: Res<ResourceGroupIconData>,
) {
    for msg in cli_events.read() {
        let AzureCliEvent::ListResourceGroups(resource_groups) = msg;
        info!("icon data: {icon_data:#?}");
        info!("Received {} resource groups", resource_groups.len());
        for (i, rg) in resource_groups.into_iter().enumerate() {
            commands
                .spawn((
                    Name::new(format!("Resource Group - {}", rg.name)),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0., i as f32 * 150., 0.)),
                        ..default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Icon"),
                        Svg2dBundle {
                            svg: icon_data.circle_icon.clone(),
                            transform: Transform::from_translation(
                                ((Vec2::new(-18., 18.) / 2.) * icon_data.scale).extend(1.),
                            )
                            .with_scale(Vec2::splat(icon_data.scale).extend(1.)),
                            // origin: Origin::Center, // Origin::TopLeft is the default
                            origin: Origin::TopLeft,
                            ..default()
                        },
                    ));
                    let circle_transform =
                        Transform::from_scale((Vec2::splat(18. + 4.) * icon_data.scale).extend(1.));
                    parent.spawn((
                        Name::new("Circle"),
                        MaterialMesh2dBundle {
                            mesh: icon_data.circle_mesh.clone(),
                            transform: circle_transform,
                            material: icon_data.circle_material.clone(),
                            ..default()
                        },
                    ));
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
                            transform: Transform::from_translation(Vec3::new(
                                circle_transform.scale.x / 2. + 5.,
                                0.,
                                0.,
                            )),
                            ..default()
                        },
                    ));
                });
        }
    }
}
	