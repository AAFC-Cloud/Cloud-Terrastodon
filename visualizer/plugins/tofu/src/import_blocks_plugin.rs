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
use cloud_terrastodon_core_tofu::prelude::TofuBlock;
use cloud_terrastodon_core_tofu::prelude::TofuImportBlock as InnerTofuImportBlock;
use cloud_terrastodon_core_tofu::prelude::TofuProviderReference;
use cloud_terrastodon_core_tofu::prelude::TofuResourceReference;
use cloud_terrastodon_visualizer_azure_plugin::prelude::AzureScope;
use cloud_terrastodon_visualizer_cursor_plugin::prelude::OnlyShowWhenHovered;
use cloud_terrastodon_visualizer_damping_plugin::CustomLinearDamping;
use cloud_terrastodon_visualizer_layout_plugin::prelude::join_on_thing_added;
use cloud_terrastodon_visualizer_layout_plugin::prelude::BiasTowardsOrigin;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_layout_plugin::prelude::OrganizableSecondary;

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
        app.observe(join_on_thing_added(
            |block: &TofuImportBlock, folder: &Folder| block.source == folder.path,
        ));
        app.observe(join_on_thing_added(
            |block: &TofuImportBlock, scope: &AzureScope| block.id == scope.scope.expanded_form(),
        ));
        app.observe(join_on_thing_added(
            |scope: &AzureScope, block: &TofuImportBlock| block.id == scope.scope.expanded_form(),
        ));
    }
}

#[derive(Component, Reflect, Debug)]
#[reflect(Default)]
pub struct TofuImportBlock {
    pub source: PathBuf,
    #[reflect(ignore)]
    pub block: InnerTofuImportBlock,
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
            source: PathBuf::from("example source"),
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
) {
    for msg in events.read() {
        let TofuEvent::Refresh(data) = msg;
        let mut i = 0;
        for (dir, blocks) in data {
            for block in blocks {
                let TofuBlock::Import(import_block) = block else {
                    continue;
                };
                let display = import_block.to.expression_str();
                commands
                    .spawn((
                        Name::new(format!("TofuImportBlock - {}", display)),
                        SpatialBundle {
                            transform: Transform::from_translation(Vec3::new(
                                0.,
                                i as f32 * 150.,
                                0.,
                            )),
                            ..default()
                        },
                        TofuImportBlock {
                            source: dir.to_owned(),
                            block: import_block.to_owned(),
                        },
                        RigidBody::Dynamic,
                        CustomLinearDamping::default(),
                        render_info.collider.clone(),
                        BiasTowardsOrigin,
                        KeepUpright,
                        OrganizableSecondary,
                    ))
                    .with_children(|parent| {
                        parent.spawn((
                            Name::new("Shape"),
                            MaterialMesh2dBundle {
                                mesh: render_info.mesh.clone(),
                                transform: render_info.shape_transform.clone(),
                                material: render_info.material.clone(),
                                ..default()
                            },
                        ));

                        parent.spawn((
                            Name::new("Icon"),
                            Svg2dBundle {
                                svg: render_info.icon.clone(),
                                transform: render_info.icon_transform.clone(),
                                origin: Origin::TopLeft,
                                ..default()
                            },
                        ));

                        parent.spawn((
                            Name::new("Text"),
                            Text2dBundle {
                                text: Text::from_section(
                                    display,
                                    TextStyle {
                                        font_size: 60.,
                                        ..default()
                                    },
                                )
                                .with_justify(JustifyText::Left),
                                text_anchor: Anchor::CenterLeft,
                                transform: render_info.text_transform.clone(),
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