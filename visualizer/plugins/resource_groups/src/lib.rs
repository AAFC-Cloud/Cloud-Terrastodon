use azure::prelude::fetch_all_resource_groups;
use azure::prelude::ResourceGroup;
use bevy::color::palettes::css::BLACK;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::sprite::MaterialMesh2dBundle;
use bevy::sprite::Mesh2dHandle;
use bevy_svg::prelude::Origin;
use bevy_svg::prelude::Svg;
use bevy_svg::prelude::Svg2dBundle;
use crossbeam_channel::bounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::thread;
pub struct ResourceGroupsPlugin;

impl Plugin for ResourceGroupsPlugin {
    fn build(&self, app: &mut App) {
        info!("Building ResourceGroupsPlugin");
        app.add_systems(Startup, (create_worker_thread, setup).chain());
        app.add_systems(Update, receive_results);
        app.init_resource::<ResourceGroupIconData>();
    }
}

#[derive(Debug)]
enum GameboundMessage {
    List(Vec<ResourceGroup>),
}

#[derive(Debug)]
enum ThreadboundMessage {
    List,
}

#[derive(Resource)]
struct Bridge {
    pub sender: Sender<ThreadboundMessage>,
    pub receiver: Receiver<GameboundMessage>,
}

fn create_worker_thread(mut commands: Commands) {
    let (game_tx, game_rx) = bounded::<_>(10);
    let (thread_tx, thread_rx) = bounded::<_>(10);
    commands.insert_resource(Bridge {
        sender: thread_tx,
        receiver: game_rx,
    });

    let game_tx_clone = game_tx.clone();
    info!("Spawning worker thread");
    thread::Builder::new()
        .name("Azure Worker Thread".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread().enable_time().build().unwrap();
            rt.block_on(async {
                let game_tx = game_tx_clone;
                loop {
                    let msg = match thread_rx.recv() {
                        Ok(msg) => msg,
                        Err(_) => {
                            error!("Threadbound channel failure, exiting");
                            break;
                        }
                    };
                    debug!("Received {msg:?}");
                    match msg {
                        ThreadboundMessage::List => {
                            info!("Fetching resource groups");
                            let resource_groups = fetch_all_resource_groups().await.unwrap();
                            let resp = GameboundMessage::List(resource_groups);
                            if let Err(e) = game_tx.send(resp) {
                                error!("Gamebound channel failure, exiting: {}", e);
                                break;
                            }
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            });
        })
        .unwrap();
}

#[derive(Debug, Resource, Default)]
struct ResourceGroupIconData {
    pub scale: f32,
    pub circle_icon: Handle<Svg>,
    pub circle_mesh: Mesh2dHandle,
    pub circle_material: Handle<ColorMaterial>,
}

fn setup(
    bridge: ResMut<Bridge>,
    mut handles: ResMut<ResourceGroupIconData>,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    handles.circle_icon = asset_server.load("textures/ResourceGroups.svg");
    handles.circle_material = materials.add(Color::from(BLACK));
    handles.circle_mesh = meshes.add(Circle::default()).into();
    handles.scale = 100. / 18.;

    let msg = ThreadboundMessage::List;
    debug!("Sending bridge message: {:?}", msg);
    if let Err(e) = bridge.sender.send(msg) {
        error!("Threadbound channel failure: {}", e);
    }
}

fn receive_results(
    bridge: ResMut<Bridge>,
    mut commands: Commands,
    icon_data: Res<ResourceGroupIconData>,
) {
    for msg in bridge.receiver.try_iter() {
        let GameboundMessage::List(resource_groups) = msg;
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
                                rg.name,
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
