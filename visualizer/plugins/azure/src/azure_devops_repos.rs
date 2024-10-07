use crate::az_cli::AzureCliResponse;
use crate::prelude::AzureDevopsProjectComponent;
use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::color::palettes::css::ORANGE;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsRepo;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_thing_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
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

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct AzureDevopsRepoIconData {
    pub icon_width: i32,
    pub circle_radius: f32,
    pub circle_icon_padding: f32,
    pub circle_text_margin: f32,
    pub circle_icon: Handle<Image>,
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
    handles.circle_icon = asset_server.load("textures/azure_devops/repos.png");
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
            commands
                .spawn((
                    Name::new(format!("Azure DevOps Repo - {}", repo.name)),
                    SpatialBundle {
                        transform: Transform::from_translation(Vec3::new(0., i as f32 * 150., 0.)),
                        ..default()
                    },
                    AzureDevopsRepoComponent {
                        inner: repo.to_owned(),
                    },
                    RigidBody::Dynamic,
                    CustomLinearDamping::default(),
                    Collider::circle(icon_data.circle_radius),
                    KeepUpright,
                    OrganizableSecondary,
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
                        SpriteBundle {
                            sprite: Sprite {
                                anchor: Anchor::TopLeft,
                                ..default()
                            },
                            texture: icon_data.circle_icon.clone(),
                            transform: icon_transform,
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
                                repo.name.to_owned(),
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
