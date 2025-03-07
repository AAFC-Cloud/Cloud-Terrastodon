use eyre::Result;
use bevy::prelude::*;
use bevy::utils::HashMap;
use cloud_terrastodon_core_command::prelude::CommandBuilder;
use cloud_terrastodon_core_command::prelude::CommandKind;
use cloud_terrastodon_core_command::prelude::OutputBehaviour;
use cloud_terrastodon_core_config::Config;
use cloud_terrastodon_core_tofu::prelude::list_blocks_for_dir;
use cloud_terrastodon_core_tofu::prelude::CodeReference;
use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::path::PathBuf;
use std::thread;

pub struct TofuWorkerPlugin;
impl Plugin for TofuWorkerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_worker_thread, initial_fetch).chain());
        app.add_systems(Update, receive_results);
        app.add_systems(Update, receive_events);
        app.add_event::<TofuEvent>();
    }
}

#[derive(Debug)]
enum GameboundMessage {
    Refresh(HashMap<PathBuf, Vec<CodeReference>>),
}

#[derive(Debug)]
enum ThreadboundMessage {
    Refresh,
    Open(PathBuf, usize),
}

#[derive(Resource)]
pub struct TofuBridge {
    sender: Sender<ThreadboundMessage>,
    receiver: Receiver<GameboundMessage>,
}

#[derive(Event)]
pub enum TofuEvent {
    Refresh(HashMap<PathBuf, Vec<CodeReference>>),
    Open(PathBuf, usize),
}

fn create_worker_thread(mut commands: Commands) {
    let (game_tx, game_rx) = unbounded::<_>();
    let (thread_tx, thread_rx) = unbounded::<_>();
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
                                let folders = &Config::get_active_config().scan_dirs.clone();

                                // build the mapping
                                let mut data = HashMap::new();
                                for folder in folders {
                                    if !folder.exists() {
                                        continue;
                                    }
                                    let blocks = list_blocks_for_dir(folder).await?;
                                    data.insert(folder.to_owned(), blocks);
                                }
                                if !data.is_empty() {
                                    let resp = GameboundMessage::Refresh(data);
                                    game_tx.send(resp)?;
                                }
                            }
                            ThreadboundMessage::Open(path, line) => {
                                CommandBuilder::new(CommandKind::VSCode)
                                    .should_announce(true)
                                    .args([
                                        "--goto",
                                        format!("{}:{}", path.display(), line).as_str(),
                                    ])
                                    .use_output_behaviour(OutputBehaviour::Display)
                                    .run_raw()
                                    .await?;
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
    {
        let msg = ThreadboundMessage::Refresh;
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_results(bridge: ResMut<TofuBridge>, mut cli_events: EventWriter<TofuEvent>) {
    for msg in bridge.receiver.try_iter() {
        let to_send: TofuEvent = match msg {
            GameboundMessage::Refresh(data) => TofuEvent::Refresh(data.clone()),
        };
        cli_events.send(to_send);
    }
}

fn receive_events(mut events: EventReader<TofuEvent>, bridge: ResMut<TofuBridge>) {
    for event in events.read() {
        match event {
            TofuEvent::Refresh(_) => {}
            TofuEvent::Open(path, line) => {
                let msg = ThreadboundMessage::Open(path.clone(), *line);
                if let Err(e) = bridge.sender.send(msg) {
                    error!("Threadbound channel failure: {}", e);
                }
            }
        }
    }
}
