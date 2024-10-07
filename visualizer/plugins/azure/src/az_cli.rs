use bevy::prelude::*;
use cloud_terrastodon_core_azure::prelude::fetch_all_resource_groups;
use cloud_terrastodon_core_azure::prelude::fetch_all_subscriptions;
use cloud_terrastodon_core_azure::prelude::ResourceGroup;
use cloud_terrastodon_core_azure::prelude::Subscription;
use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_projects;
use cloud_terrastodon_core_azure_devops::prelude::fetch_all_azure_devops_repos_for_project;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsProject;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsProjectId;
use cloud_terrastodon_core_azure_devops::prelude::AzureDevopsRepo;
use crossbeam_channel::unbounded;
use crossbeam_channel::Receiver;
use crossbeam_channel::Sender;
use std::error::Error;
use std::thread;

pub struct AzureCliPlugin;
impl Plugin for AzureCliPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (create_worker_thread, initial_fetch).chain());
        app.add_systems(Update, receive_responses);
        app.add_systems(Update, receive_requests);
        app.add_systems(Update, refresh_repos);
        app.add_event::<AzureCliRequest>();
        app.add_event::<AzureCliResponse>();
    }
}

#[derive(Resource)]
pub struct AzureCliBridge {
    sender: Sender<AzureCliRequest>,
    receiver: Receiver<AzureCliResponse>,
}

#[derive(Debug, Event, Clone)]
pub enum AzureCliRequest {
    ListResourceGroups,
    ListSubscriptions,
    ListAzureDevopsProjects,
    ListAzureDevopsRepos(AzureDevopsProjectId),
}

#[derive(Debug, Event, Clone)]
pub enum AzureCliResponse {
    ListResourceGroups(Vec<ResourceGroup>),
    ListSubscriptions(Vec<Subscription>),
    ListAzureDevopsProjects(Vec<AzureDevopsProject>),
    ListAzureDevopsRepos(Vec<AzureDevopsRepo>),
}

fn create_worker_thread(mut commands: Commands) {
    let (game_tx, game_rx) = unbounded::<_>();
    let (thread_tx, thread_rx) = unbounded::<_>();
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
                    let response: Result<AzureCliResponse, Box<dyn Error>> = try {
                        match msg {
                            AzureCliRequest::ListResourceGroups => {
                                info!("Fetching resource groups");
                                let resource_groups = fetch_all_resource_groups().await?;
                                AzureCliResponse::ListResourceGroups(resource_groups)
                            }
                            AzureCliRequest::ListSubscriptions => {
                                info!("Fetching subscriptions");
                                let subscriptions = fetch_all_subscriptions().await?;
                                AzureCliResponse::ListSubscriptions(subscriptions)
                            }
                            AzureCliRequest::ListAzureDevopsProjects => {
                                info!("Fetching AzureDevopsProjects");
                                let projects = fetch_all_azure_devops_projects().await?;
                                AzureCliResponse::ListAzureDevopsProjects(projects)
                            }
                            AzureCliRequest::ListAzureDevopsRepos(project_id) => {
                                info!("Fetching AzureDevopsRepos");
                                let repos = fetch_all_azure_devops_repos_for_project(&project_id).await?;
                                info!("Found {} repos", repos.len());
                                AzureCliResponse::ListAzureDevopsRepos(repos)
                            }
                        }
                    };
                    let response = match response {
                        Ok(data) => data,
                        Err(e) => {
                            error!("Threadbound message processing failed: {e}");
                            break;
                        }
                    };
                    if let Err(e) = game_tx.send(response) {
                        error!("Gamebound channel failure, exiting: {}", e);
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(50));
                }
            });
        })
        .unwrap();
}

fn initial_fetch(bridge: ResMut<AzureCliBridge>) {
    for msg in [
        AzureCliRequest::ListResourceGroups,
        AzureCliRequest::ListSubscriptions,
        AzureCliRequest::ListAzureDevopsProjects,
    ] {
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_requests(bridge: ResMut<AzureCliBridge>, mut cli_requests: EventReader<AzureCliRequest>) {
    for msg in cli_requests.read() {
        debug!("Sending bridge message: {:?}", msg);
        if let Err(e) = bridge.sender.send(msg.to_owned()) {
            error!("Threadbound channel failure: {}", e);
        }
    }
}

fn receive_responses(bridge: ResMut<AzureCliBridge>, mut cli_responses: EventWriter<AzureCliResponse>) {
    for msg in bridge.receiver.try_iter() {
        cli_responses.send(msg);
    }
}

fn refresh_repos(mut cli_responses: EventReader<AzureCliResponse>, mut cli_requests: EventWriter<AzureCliRequest>) {
    // debug!("refresh repos firing");
    for msg in cli_responses.read() {
        debug!("refresh repos checking {msg:?}");
        if let AzureCliResponse::ListAzureDevopsProjects(projects) = msg {
            debug!("found list of {} projects to get repos for", projects.len());
            for project in projects.iter().take(30) {
                debug!("Requesting repos refresh for Azure DevOps project {}", project.name);
                cli_requests.send(AzureCliRequest::ListAzureDevopsRepos(project.id.to_owned()));
            }
        }
    }
}