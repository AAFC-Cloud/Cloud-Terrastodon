use std::ops::Deref;
use std::path::PathBuf;

use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::color::palettes::css::PURPLE;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_core_azure::prelude::Scope;
use cloud_terrastodon_core_tofu::prelude::TofuAzureRMResourceKind;
use cloud_terrastodon_core_tofu::prelude::TofuBlock;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock as InnerTofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuResourceReference;
use cloud_terrastodon_visualizer_azure_plugin::prelude::AzureScope;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_follower_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_leader_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;
use cloud_terrastodon_visualizer_physics_plugin::prelude::CustomLinearDamping;

use crate::folder_plugin::Folder;
use crate::tofu_worker_plugin::TofuEvent;
pub struct TofuImportBlocksPlugin;

impl Plugin for TofuImportBlocksPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<TofuImportBlock>();
        app.register_type::<TofuImportBlockRenderInfo>();
        app.init_resource::<TofuImportBlockRenderInfo>();
        app.add_systems(Startup, setup);
        app.add_systems(Update, spawn_folders);
        app.observe(join_on_follower_added(
            |block: &TofuImportBlock, folder: &Folder| block.dir_path == folder.path,
        ));
        app.observe(join_on_follower_added(
            |block: &TofuImportBlock, scope: &AzureScope| block.id == scope.scope.expanded_form(),
        ));
        app.observe(join_on_leader_added(
            |scope: &AzureScope, block: &TofuImportBlock| block.id == scope.scope.expanded_form(),
        ));
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Default)]
pub struct TofuImportBlock {
    pub dir_path: PathBuf,
    pub file_path: PathBuf,
    #[reflect(ignore)]
    pub block: InnerTofuImportBlock,
    pub line_col: (usize, usize),
}
impl Deref for TofuImportBlock {
    type Target = InnerTofuImportBlock;

    fn deref(&self) -> &Self::Target {
        &self.block
    }
}
impl Default for TofuImportBlock {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("example dir"),
            dir_path: PathBuf::from("example dir/example file"),
            line_col: (0, 0),
            block: InnerTofuImportBlock {
                provider: TofuProviderReference::Inherited,
                id: "example id".to_string(),
                to: TofuResourceReference::Other {
                    provider: "example".to_string(),
                    kind: "thing".to_string(),
                    name: "foo".to_string(),
                },
            },
        }
    }
}

#[derive(Debug, Resource, Default, Reflect)]
#[reflect(Resource)]
struct TofuImportBlockRenderInfo {
    pub icon_transform: Transform,
    pub icon: Handle<Svg>,
    pub inner_icon_transform: Transform,
    pub material: Handle<ColorMaterial>,
    pub mesh: Mesh2dHandle,
    pub padding: f32,
    pub shape_width: f32,
    pub text_transform: Transform,
    pub shape_transform: Transform,
    #[reflect(ignore)]
    pub collider: Collider,
}

fn setup(
    mut render_info: ResMut<TofuImportBlockRenderInfo>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    info!("Setting up folder render info");
    // shape
    render_info.shape_width = 100.;
    render_info.shape_transform = Transform::default();
    render_info.mesh = meshes
        .add(Rectangle::new(
            render_info.shape_width,
            render_info.shape_width,
        ))
        .into();
    render_info.collider = Collider::rectangle(render_info.shape_width, render_info.shape_width);

    // icon
    render_info.padding = 4.;
    render_info.icon = asset_server.load("textures/fluent_emoji/page_facing_up_color.svg");
    {
        let raw_icon_width = 32;
        let icon_scale =
            (1. / raw_icon_width as f32) * (render_info.shape_width - render_info.padding);
        let icon_scale = Vec2::splat(icon_scale).extend(1.);
        let icon_translation =
            (Vec2::new(-icon_scale.x, icon_scale.y) * raw_icon_width as f32 / 2.).extend(1.);
        render_info.icon_transform =
            Transform::from_translation(icon_translation).with_scale(icon_scale);
    }

    // inner icon
    render_info.inner_icon_transform = {
        let raw_icon_width = 18;
        let desired_icon_width = 50.;
        let icon_scale = (1. / raw_icon_width as f32) * desired_icon_width;
        let icon_scale = Vec2::splat(icon_scale).extend(1.);
        let icon_translation = Vec3::new(-desired_icon_width / 2., desired_icon_width / 2., 2.);
        Transform::from_translation(icon_translation).with_scale(icon_scale)
    };

    // material
    render_info.material = materials.add(Color::from(PURPLE));

    // text
    {
        let text_margin = 4.;
        let text_translation = Vec3::new(render_info.shape_width + text_margin, 0., 5.);
        render_info.text_transform = Transform::from_translation(text_translation);
    }
}

fn spawn_folders(
    mut commands: Commands,
    render_info: Res<TofuImportBlockRenderInfo>,
    mut events: EventReader<TofuEvent>,
    asset_server: Res<AssetServer>,
) {
    for msg in events.read() {
        let TofuEvent::Refresh(data) = msg else {
            continue;
        };
        let mut i = 0;
        for (dir, blocks) in data {
            for reference in blocks {
                let TofuBlock::Import(import_block) = &reference.block else {
                    continue;
                };
                commands
                    .spawn((
                        Name::new(format!(
                            "TofuImportBlock - {}",
                            import_block.to.expression_str()
                        )),
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                0.,
                                i as f32 * 150.,
                                0.,
                            )),
                            ..default()
                        },
                        TofuImportBlock {
                            file_path: reference.path.to_owned(),
                            dir_path: dir.to_owned(),
                            block: import_block.to_owned(),
                            line_col: reference.line_col,
                        },
                        RigidBody::Dynamic,
                        CustomLinearDamping::default(),
                        render_info.collider.clone(),
                        KeepUpright,
                        OrganizableSecondary,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Name::new("Shape"),
                            MaterialMesh2dBundle {
                                mesh: render_info.mesh.clone(),
                                transform: render_info.shape_transform,
                                material: render_info.material.clone(),
                                ..default()
                            },
                        ));

                        parent.spawn((
                            Name::new("Icon"),
                            Svg2dBundle {
                                svg: render_info.icon.clone(),
                                transform: render_info.icon_transform,
                                origin: Origin::TopLeft,
                                ..default()
                            },
                        ));

                        let inner_icon_path = match &import_block.to {
                            TofuResourceReference::AzureRM {
                                kind: TofuAzureRMResourceKind::ResourceGroup,
                                ..
                            } => "textures/azure/ResourceGroups.svg",
                            x => {
                                warn!("No icon available for import block of resource kind {x:?}");
                                ""
                            }
                        };
                        if !inner_icon_path.is_empty() {
                            parent.spawn((
                                Name::new("Inner Icon"),
                                Svg2dBundle {
                                    svg: asset_server.load(inner_icon_path),
                                    transform: render_info.inner_icon_transform,
                                    origin: Origin::TopLeft,
                                    ..default()
                                },
                            ));
                        }

                        parent.spawn((
                            Name::new("Text"),
                            Text2dBundle {
                                text: Text::from_section(
                                    import_block.to.name_label(),
                                    TextStyle {
                                        font_size: 60.,
                                        ..default()
                                    },
                                )
                                .with_justify(JustifyText::Left),
                                text_anchor: Anchor::CenterLeft,
                                transform: render_info.text_transform,
                                ..default()
                            },
                            OnlyShowWhenHovered,
                        ));
                    });
                i += 1;
            }
        }
    }
}
