use avian2d::prelude::Collider;
use avian2d::prelude::RigidBody;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use cloud_terrastodon_visualizer_layout_plugin::prelude::BiasTowardsOrigin;
use cloud_terrastodon_visualizer_layout_plugin::prelude::KeepUpright;
use cloud_terrastodon_visualizer_physics_plugin::prelude::CustomLinearDamping;

pub struct NodeSpawningPlugin;
impl Plugin for NodeSpawningPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<IconHandle>();
    }
}

#[derive(Reflect, Debug, Default, Clone)]
pub enum IconHandle {
    #[default]
    Unset,
    Image(Handle<Image>),
    Svg(Handle<Svg>),
}
impl From<Handle<Svg>> for IconHandle {
    fn from(value: Handle<Svg>) -> Self {
        IconHandle::Svg(value)
    }
}
impl From<Handle<Image>> for IconHandle {
    fn from(value: Handle<Image>) -> Self {
        IconHandle::Image(value)
    }
}

pub trait GraphNodeIconData {
    fn icon_width(&self) -> i32;
    fn circle_radius(&self) -> f32;
    fn circle_icon_padding(&self) -> f32;
    fn circle_text_margin(&self) -> f32;
    fn circle_icon(&self) -> IconHandle;
    fn circle_mesh(&self) -> Mesh2dHandle;
    fn circle_material(&self) -> Handle<ColorMaterial>;
}

#[derive(Default)]
pub struct SpawnGraphNodeEvent<TTop: Bundle, TCircle: Bundle, TIcon: Bundle, TText: Bundle> {
    pub name: Name,
    pub text: String,
    pub translation: Vec3,
    pub top_extras: TTop,
    pub circle_extras: TCircle,
    pub icon_extras: TIcon,
    pub text_extras: TText,
}

pub fn spawn_graph_node<TTop: Bundle, TCircle: Bundle, TIcon: Bundle, TText: Bundle>(
    event: SpawnGraphNodeEvent<TTop, TCircle, TIcon, TText>,
    icon_data: &impl GraphNodeIconData,
    commands: &mut Commands,
) {
    commands
        .spawn((
            event.name,
            SpatialBundle {
                transform: Transform::from_translation(event.translation),
                ..default()
            },
            RigidBody::Dynamic,
            CustomLinearDamping::default(),
            Collider::circle(icon_data.circle_radius()),
            BiasTowardsOrigin,
            KeepUpright,
            event.top_extras,
        ))
        .with_children(|parent| {
            // circle
            let circle_scale = Vec2::splat(icon_data.circle_radius()).extend(1.);
            let circle_transform = Transform::from_scale(circle_scale);
            parent.spawn((
                Name::new("Circle"),
                MaterialMesh2dBundle {
                    mesh: icon_data.circle_mesh().clone(),
                    transform: circle_transform,
                    material: icon_data.circle_material().clone(),
                    ..default()
                },
                event.circle_extras,
            ));

            // icon
            let icon_scale = Vec2::splat(
                (1. / icon_data.icon_width() as f32)
                    * ((icon_data.circle_radius() * 2.) - icon_data.circle_icon_padding()),
            )
            .extend(1.);
            let icon_translation =
                (Vec2::new(-icon_scale.x, icon_scale.y) * icon_data.icon_width() as f32 / 2.)
                    .extend(1.);
            let icon_transform =
                Transform::from_translation(icon_translation).with_scale(icon_scale);
            let mut icon = parent.spawn((Name::new("Icon"), event.icon_extras));
            match icon_data.circle_icon() {
                IconHandle::Unset => todo!(),
                IconHandle::Image(handle) => icon.insert(SpriteBundle {
                    sprite: Sprite {
                        anchor: Anchor::TopLeft,
                        ..default()
                    },
                    texture: handle,
                    transform: icon_transform,
                    ..default()
                }),
                IconHandle::Svg(handle) => icon.insert(Svg2dBundle {
                    svg: handle,
                    transform: icon_transform,
                    origin: Origin::TopLeft,
                    ..default()
                }),
            };

            // text
            let text_translation = Vec3::new(
                icon_data.circle_radius() + icon_data.circle_text_margin(),
                0.,
                5.,
            );
            parent.spawn((
                Name::new("Text"),
                Text2dBundle {
                    text: Text::from_section(
                        event.text,
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
                event.text_extras,
            ));
        });
}
