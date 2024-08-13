use bevy::color::palettes::css::BLACK;
use bevy::input::common_conditions::input_toggle_active;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_svg::prelude::*;
use camera_plugin::MyCameraPlugin;

fn main() {
    let mut app = App::new();

    app.insert_resource(Msaa::Sample4);

    app.add_plugins(DefaultPlugins);
    app.add_plugins(SvgPlugin);
    app.add_plugins(MyCameraPlugin);

    // must be after the default plugins
    app.add_plugins(
        WorldInspectorPlugin::default().run_if(input_toggle_active(false, KeyCode::Backquote)),
    );

    app.add_systems(Startup, setup);

    app.run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let names = Vec::new();
    let scale = 100. / 18.;
    for (i, rg_name) in names.enumerate() {
        commands
            .spawn((Name::new("Resource Group"), SpatialBundle::default()))
            .with_children(|parent| {
                parent.spawn((
                    Name::new("Icon"),
                    Svg2dBundle {
                        svg: asset_server.load("textures/ResourceGroups.svg"),
                        transform: Transform::from_translation(
                            ((Vec2::new(-18., 18.) / 2.) * scale).extend(1.),
                        )
                        .with_scale(Vec2::splat(scale).extend(1.)),
                        // origin: Origin::Center, // Origin::TopLeft is the default
                        origin: Origin::TopLeft,
                        ..default()
                    },
                ));
                let circle_transform =
                    Transform::from_scale((Vec2::splat(18. + 4.) * scale).extend(1.));
                parent.spawn((
                    Name::new("Circle"),
                    MaterialMesh2dBundle {
                        mesh: meshes.add(Circle::default()).into(),
                        transform: circle_transform,
                        material: materials.add(Color::from(BLACK)),
                        ..default()
                    },
                ));
                parent.spawn((
                    Name::new("Text"),
                    Text2dBundle {
                        text: Text::from_section(
                            rg_name,
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
