use crate::app::MyApp;
use crate::portal_typography;
use cloud_terrastodon_auth::CloudTerrastodonAccountStore;
use cloud_terrastodon_auth_types::AzureBearerAuthenticationSelection;
use cloud_terrastodon_azure_devops::AzureDevOpsOrganizationUrl;
use cloud_terrastodon_command::app_work::AppWorkTracker;
use cloud_terrastodon_config::Config;
use cloud_terrastodon_execution::CloudTerrastodonExecutor;
use cloud_terrastodon_pathing::AppDir;
use eframe::NativeOptions;
#[cfg(windows)]
use eframe::egui::IconData;
use eframe::egui::ViewportBuilder;
use eframe::egui::ViewportCommand;
use eframe::egui::ViewportId;
use eframe::egui::Visuals;
use eyre::WrapErr;
use eyre::eyre;
use std::panic::AssertUnwindSafe;
use std::panic::catch_unwind;
use std::panic::resume_unwind;
use std::sync::mpsc::sync_channel;
use teamy_tracing::DiagnosticReport;
use teamy_tracing::capture::LogBuffer;
use tokio::runtime;
use tokio::task::block_in_place;
use tracing::info;

pub async fn run_app(
    app_info: String,
    azure_devops_organization: Option<AzureDevOpsOrganizationUrl>,
    authentication: eyre::Result<Option<AzureBearerAuthenticationSelection>>,
    account_store: CloudTerrastodonAccountStore,
    log_capture: LogBuffer,
    executor: &CloudTerrastodonExecutor,
) -> eyre::Result<()> {
    let (_shutdown, run) = executor.with_async_child_scope(move |executor| async move {
        let viewport = ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([640.0, 480.0]);
        // Eframe installs its default icon again on the first frame unless an
        // explicit icon is supplied. Windows uses the executable resource below.
        #[cfg(windows)]
        let viewport = viewport.with_icon(IconData::default());
        let native_options = NativeOptions {
            viewport,
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
                        "Cloud Terrastodon",
                        native_options,
                        Box::new(|cc: &eframe::CreationContext<'_>| {
                            #[cfg(windows)]
                            if let Err(error) = crate::bundled_window_icon::apply(cc) {
                                // Other SDK hosts may own different executable resources.
                                let report = DiagnosticReport::new(error);
                                tracing::debug!(
                                    error = report.as_error(),
                                    "No bundled window icon is available"
                                );
                            }
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
                            portal_typography::install(&cc.egui_ctx);
                            cc.egui_ctx.set_visuals(Visuals::light());
                            let app = runtime::Handle::current().block_on(async move {
                                MyApp::new(
                                    azure_devops_organization,
                                    authentication,
                                    account_store,
                                    log_capture,
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
                        let egui_saved = egui_config
                            .save()
                            .await
                            .wrap_err("Failed to save final GUI settings");
                        let work_dirs_saved = work_dirs_config
                            .save()
                            .await
                            .wrap_err("Failed to save final work-directory settings");
                        // The returned error owns primary-error presentation.
                        // Log only another failure that would otherwise be lost.
                        match (egui_saved, work_dirs_saved) {
                            (Err(primary), Err(secondary)) => {
                                let report = DiagnosticReport::new(secondary);
                                tracing::error!(
                                    error = report.as_error(),
                                    "Work-directory settings also failed to save"
                                );
                                return Err(primary);
                            }
                            (Err(error), Ok(())) | (Ok(()), Err(error)) => return Err(error),
                            (Ok(()), Ok(())) => {}
                        }
                    }
                    eyre::Ok(())
                })
                .await;
            let drained = match (drained, saved) {
                (Err(primary), Err(secondary)) => {
                    let report = DiagnosticReport::new(secondary);
                    tracing::error!(
                        error = report.as_error(),
                        "Settings also failed to save while background work failed"
                    );
                    Err(primary)
                }
                (Err(error), Ok(())) | (Ok(()), Err(error)) => Err(error),
                (Ok(()), Ok(())) => Ok(()),
            };
            match (result, drained) {
                (Err(panic), drain) => {
                    if let Err(drain_error) = drain {
                        let report = DiagnosticReport::new(drain_error);
                        tracing::error!(
                            error = report.as_error(),
                            "Background work failed while draining after a GUI panic"
                        );
                    }
                    resume_unwind(panic);
                }
                (Ok(Err(error)), drain) => {
                    if let Err(drain_error) = drain {
                        let report = DiagnosticReport::new(drain_error);
                        tracing::error!(
                            error = report.as_error(),
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
