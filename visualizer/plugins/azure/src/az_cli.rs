use bevy::prelude::*;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::ResourceGroup;
use cloud_terrastodon_core_azure::prelude::Subscription;
use crossbeam_channel::bounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::thread;

pub struct AzureCliPlugin;
impl Plugin for AzureCliPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_worker_thread, initial_fetch).chain());
        app.add_systems(Update, receive_results);
        app.add_event::<AzureCliEvent>();
    }
}

#[derive(Debug)]
enum GameboundMessage {
    ListResourceGroups(Vec<ResourceGroup>),
    ListSubscriptions(Vec<Subscription>),
}

#[derive(Debug)]
enum ThreadboundMessage {
    ListResourceGroups,
    ListSubscriptions,
}

#[derive(Resource)]
pub struct AzureCliBridge {
    sender: Sender<ThreadboundMessage>,
    receiver: Receiver<GameboundMessage>,
}

#[derive(Event)]
pub enum AzureCliEvent {
    ListResourceGroups(Vec<ResourceGroup>),
    ListSubscriptions(Vec<Subscription>),
}

fn create_worker_thread(mut commands: Commands) {
    let (game_tx, game_rx) = bounded::<_>(10);
    let (thread_tx, thread_rx) = bounded::<_>(10);
    let bridge = AzureCliBridge {
        sender: thread_tx,
        receiver: game_rx,
    };
    commands.insert_resource(bridge);

    let game_tx_clone = game_tx.clone();
    info!("Spawning worker thread");
    let _handle = thread::Builder::new()
        .name("Azure Worker Thread".to_string())
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
                        ThreadboundMessage::ListResourceGroups => {
                            info!("Fetching resource groups");
                            let resource_groups = fetch_all_resource_groups().await.unwrap();
                            let resp = GameboundMessage::ListResourceGroups(resource_groups);
                            if let Err(e) = game_tx.send(resp) {
                                error!("Gamebound channel failure, exiting: {}", e);
                                break;
                            }
                        }
                        ThreadboundMessage::ListSubscriptions => {
                            info!("Fetching subscriptions");
                            let subscriptions = fetch_all_subscriptions().await.unwrap();
                            let resp = GameboundMessage::ListSubscriptions(subscriptions);
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

fn initial_fetch(bridge: ResMut<AzureCliBridge>) {
    for msg in [
        ThreadboundMessage::ListResourceGroups,
        ThreadboundMessage::ListSubscriptions,
    ] {
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_results(bridge: ResMut<AzureCliBridge>, mut cli_events: EventWriter<AzureCliEvent>) {
    for msg in bridge.receiver.try_iter() {
        let to_send: AzureCliEvent = match msg {
            GameboundMessage::ListResourceGroups(resource_groups) => {
                AzureCliEvent::ListResourceGroups(resource_groups)
            }
            GameboundMessage::ListSubscriptions(subscriptions) => {
                AzureCliEvent::ListSubscriptions(subscriptions)
            }
        };
        cli_events.send(to_send);
    }
}
