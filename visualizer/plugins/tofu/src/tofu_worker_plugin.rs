use bevy::prelude::*;
use bevy::utils::HashSet as BevyHashSet;
use cloud_terrastodon_core_config::Config;
use crossbeam_channel::bounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::collections::HashSet;
use std::path::PathBuf;
use std::thread;

pub struct TofuWorkerPlugin;
impl Plugin for TofuWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_worker_thread, initial_fetch).chain());
        app.add_systems(Update, receive_results);
        app.add_event::<TofuEvent>();
    }
}

#[derive(Debug)]
enum GameboundMessage {
    ListFolders(HashSet<PathBuf>),
}

#[derive(Debug)]
enum ThreadboundMessage {
    ListFolders,
}

#[derive(Resource)]
pub struct TofuBridge {
    sender: Sender<ThreadboundMessage>,
    receiver: Receiver<GameboundMessage>,
}

#[derive(Event)]
pub enum TofuEvent {
    ListFolders(BevyHashSet<PathBuf>),
}

fn create_worker_thread(mut commands: Commands) {
    let (game_tx, game_rx) = bounded::<_>(10);
    let (thread_tx, thread_rx) = bounded::<_>(10);
    let bridge = TofuBridge {
        sender: thread_tx,
        receiver: game_rx,
    };
    commands.insert_resource(bridge);

    let game_tx_clone = game_tx.clone();
    info!("Spawning worker thread");
    let _handle = thread::Builder::new()
        .name("Tofu Worker Thread".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_time()
                .build()
                .unwrap();
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
                        ThreadboundMessage::ListFolders => {
                            info!("List Folders");
                            let folders = Config::get_active_config().scan_dirs.clone();
                            let resp = GameboundMessage::ListFolders(folders);
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

fn initial_fetch(bridge: ResMut<TofuBridge>) {
    for msg in [ThreadboundMessage::ListFolders] {
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_results(bridge: ResMut<TofuBridge>, mut cli_events: EventWriter<TofuEvent>) {
    for msg in bridge.receiver.try_iter() {
        let to_send: TofuEvent = match msg {
            GameboundMessage::ListFolders(folders) => {
                TofuEvent::ListFolders(folders.iter().cloned().collect())
            }
        };
        cli_events.send(to_send);
    }
}
