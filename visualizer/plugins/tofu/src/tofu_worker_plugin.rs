use bevy::prelude::*;
use bevy::utils::HashMap;
use cloud_terrastodon_core_config::Config;
use cloud_terrastodon_core_tofu::prelude::as_single_body;
use cloud_terrastodon_core_tofu::prelude::IntoTofuBlocks;
use cloud_terrastodon_core_tofu::prelude::TofuBlock;
use crossbeam_channel::bounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use anyhow::Result;
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
    Refresh(HashMap<PathBuf, Vec<TofuBlock>>),
}

#[derive(Debug)]
enum ThreadboundMessage {
    Refresh,
}

#[derive(Resource)]
pub struct TofuBridge {
    sender: Sender<ThreadboundMessage>,
    receiver: Receiver<GameboundMessage>,
}

#[derive(Event)]
pub enum TofuEvent {
    Refresh(HashMap<PathBuf, Vec<TofuBlock>>),
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
                    let result: Result<()> = try {
                        let msg = thread_rx.recv()?;
                        debug!("Received {msg:?}");
                        match msg {
                            ThreadboundMessage::Refresh => {
                                info!("Refresh");
                                // get the dirs to scan
                                let folders = &Config::get_active_config().scan_dirs;

                                // build the mapping
                                let mut data = HashMap::new();
                                for folder in folders {
                                    let body = as_single_body(folder).await?;
                                    let blocks = body.try_into_tofu_blocks()?;
                                    data.insert(folder.to_owned(), blocks);
                                }
                                let resp = GameboundMessage::Refresh(data);
                                game_tx.send(resp)?;
                            }
                        }
                    };
                    if let Err(e) = result {
                        error!("Worker error: {}", e);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            });
        })
        .unwrap();
}

fn initial_fetch(bridge: ResMut<TofuBridge>) {
    for msg in [ThreadboundMessage::Refresh] {
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_results(bridge: ResMut<TofuBridge>, mut cli_events: EventWriter<TofuEvent>) {
    for msg in bridge.receiver.try_iter() {
        let to_send: TofuEvent = match msg {
            GameboundMessage::Refresh(data) => {
                TofuEvent::Refresh(data.clone())
            }
        };
        cli_events.send(to_send);
    }
}
