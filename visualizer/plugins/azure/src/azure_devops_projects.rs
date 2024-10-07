use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureDevopsRepoComponent;
use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::color::palettes::css::BLUE;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsProject;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_thing_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::BiasTowardsOrigin;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizablePrimary;
use std::ops::Deref;

pub struct AzureDevopsProjectsPlugin;
impl Plugin for AzureDevopsProjectsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building AzureDevopsProjectsPlugin");
        app.add_systems(Startup, setup);
        app.add_systems(Update, receive_results);
        app.register_type::<AzureDevopsProjectIconData>();
        app.init_resource::<AzureDevopsProjectIconData>();
        app.observe(join_on_thing_added(
            |project: &AzureDevopsProjectComponent, repo: &AzureDevopsRepoComponent| {
                repo.project.id == project.id
            },
        ));
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct AzureDevopsProjectIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: Handle<Svg>,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

#[derive(Debug, Component)]
pub struct AzureDevopsProjectComponent {
    pub inner: AzureDevopsProject,
}
impl Deref for AzureDevopsProjectComponent {
    type Target = AzureDevopsProject;

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
    handles.circle_icon =
        asset_server.load("textures/azure_devops/10261-icon-service-Azure-DevOps.svg");
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
        debug!("icon data: {icon_data:#?}");
        debug!("Received {} azure devops projects", projects.len());
        for (i, project) in projects.iter().enumerate() {
            commands
                .spawn((
                    Name::new(format!("Azure DevOps Project - {}", project.name)),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0., i as f32 * 150., 0.)),
                        ..default()
                    },
                    AzureDevopsProjectComponent {
                        inner: project.to_owned(),
                    },
                    RigidBody::Dynamic,
                    CustomLinearDamping::default(),
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
                                project.name.to_owned(),
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
