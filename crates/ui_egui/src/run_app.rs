use crate::app::MyApp;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_pathing::AppDir;
use eframe::NativeOptions;
use eframe::egui::ViewportCommand;
use eframe::egui::ViewportId;
use eyre::eyre;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::panic::resume_unwind;
use std::sync::mpsc::sync_channel;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn run_app(
    app_info: String,
    azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    authentication: AzureBearerAuthenticationSelection,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let (_shutdown, run) = executor.with_async_child_scope(move |executor| async move {
        let native_options = NativeOptions {
            persist_window: true,
            persistence_path: Some(AppDir::Config.join("egui_window_state.ron")),
            run_and_return: true,
            ..Default::default()
        };

        let work_tracker = AppWorkTracker::new();
        let (shutdown_settings, settings_receiver) = sync_channel(1);

        {
            let work_tracker2 = work_tracker.clone();
            let app_executor = executor.clone();
            let result = catch_unwind(AssertUnwindSafe(|| {
                block_in_place(move || {
                    // Keep the wakeup registered until the native event loop
                    // exits, including while no UI frame is otherwise due.
                    let mut _shutdown_listener = None;
                    eframe::run_native(
                        "MyApp",
                        native_options,
                        Box::new(|cc: &eframe::CreationContext<'_>| {
                            let gui = cc.egui_ctx.clone();
                            _shutdown_listener =
                                Some(app_executor.cancellation().register_listener(move |_| {
                                    // Sending a viewport command also requests its
                                    // repaint, waking an idle eframe event loop.
                                    gui.send_viewport_cmd_to(
                                        ViewportId::ROOT,
                                        ViewportCommand::Close,
                                    );
                                }));
                            // This gives us image support:
                            egui_extras::install_image_loaders(&cc.egui_ctx);
                            let app = runtime::Handle::current().block_on(async move {
                                MyApp::new(
                                    azure_devops_organization,
                                    authentication,
                                    work_tracker2,
                                    shutdown_settings,
                                    app_info,
                                    &app_executor,
                                )
                                .await
                            })?;
                            Ok(Box::new(app))
                        }),
                    )
                })
            }));
            executor.cancellation().request_cancel("GUI window closed");
            let drained = work_tracker.finish(executor.cancellation()).await;
            // A queued periodic save can be cancelled before it starts. The
            // native app transfers its final settings on exit, and these writes
            // run after earlier jobs so an older snapshot cannot overwrite them.
            let saved = executor
                .cancellation()
                .finish_required("save final GUI settings", async |_cleanup| {
                    if let Ok((egui_config, work_dirs_config)) = settings_receiver.try_recv() {
                        // Attempt both saves even if one fails.
                        let egui_saved = egui_config.save().await;
                        let work_dirs_saved = work_dirs_config.save().await;
                        if let Err(error) = &work_dirs_saved {
                            tracing::error!(?error, "Failed to save final work-directory settings");
                        }
                        egui_saved?;
                        work_dirs_saved?;
                    }
                    eyre::Ok(())
                })
                .await;
            if let Err(error) = &saved {
                tracing::error!(?error, "Failed to save final GUI settings");
            }
            let drained = drained.and(saved);
            match (result, drained) {
                (Err(panic), drain) => {
                    if let Err(drain_error) = drain {
                        tracing::error!(
                            ?drain_error,
                            "Background work failed while draining after a GUI panic"
                        );
                    }
                    resume_unwind(panic);
                }
                (Ok(Err(error)), drain) => {
                    if let Err(drain_error) = drain {
                        tracing::error!(
                            ?drain_error,
                            "Background work also failed during GUI shutdown"
                        );
                    }
                    // eframe can contain a non-Send glutin error, so its
                    // diagnostic must cross eyre's Send + Sync boundary as text.
                    return Err(eyre!("Failed to run app: {error:#?}"));
                }
                (Ok(Ok(())), drain) => drain?,
            }
        }

        info!("Goodbye!");

        Ok(())
    });
    // This sibling lease retains the GUI owner through shutdown without making
    // independently created SDK jobs inherit a blanket cancellation deferral.
    executor
        .cancellation()
        .deferred(
            "close GUI and drain background work",
            async move |_lifetime| run.await,
        )
        .await
}
